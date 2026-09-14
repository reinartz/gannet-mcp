# Usage Guide

This guide covers how to run and use the gannet-mcp server.

## Quick Start

```bash
# Run with default settings (DuckDuckGo, STDIO mode)
gannet-mcp

# Run with debug logging
gannet-mcp --log-level debug

# Use a custom config file
gannet-mcp --config config.yaml
```

## Transport Modes

### STDIO Mode (Default)

STDIO mode is the recommended transport for MCP clients. The server reads JSON-RPC messages from stdin and writes responses to stdout.

```bash
# Default STDIO mode
gannet-mcp

# Explicitly enable STDIO
MCP__SERVER__USE_STDIO=true gannet-mcp
```

### HTTP Mode

HTTP mode serves the MCP protocol over HTTP with Streamable HTTP transport.

```bash
# Enable HTTP mode
gannet-mcp --http

# Custom host and port
MCP__SERVER__HTTP__HOST=0.0.0.0 MCP__SERVER__HTTP__PORT=9000 gannet-mcp --http

# Custom endpoint path
gannet-mcp --http --mcp-path /api/mcp
```

> **Security note:** HTTP mode has no authentication in v0.1.0 (bearer-token
> auth is a planned follow-up). Bind localhost (the default) and expose it
> via a reverse proxy with TLS if remote access is needed. Avoid binding
> `0.0.0.0` on an untrusted network.

### HTTP Configuration Options

| Option | Env Var | Default | Description |
|--------|---------|---------|-------------|
| Host | `MCP__SERVER__HTTP__HOST` | `127.0.0.1` | Bind address |
| Port | `MCP__SERVER__HTTP__PORT` | `8080` | Bind port |
| Path | `MCP__SERVER__HTTP__PATH` | `/mcp` | Endpoint path |
| SSE Keep-Alive | `MCP__SERVER__HTTP__SSE_KEEP_ALIVE_SECS` | `15` | SSE heartbeat interval |
| JSON Response | `MCP__SERVER__HTTP__JSON_RESPONSE` | `true` | Return JSON instead of text/event-stream |
| Allowed Hosts | `MCP__SERVER__HTTP__ALLOWED_HOSTS` | (none) | Comma-separated list of allowed request hosts |

## Command Line Options

```
Usage: gannet-mcp [OPTIONS]

Options:
  -c, --config <CONFIG>     Path to configuration file (YAML/JSON)
  -l, --log-level <LOG_LEVEL>  Log level (trace, debug, info, warn, error)
      --http                Enable HTTP mode (Streamable HTTP transport)
      --mcp-path <PATH>     MCP endpoint path (HTTP mode only, default: /mcp)
  -h, --help                Print help
  -V, --version             Print version
```

## MCP Tools

### web_search

Search the web for information using the configured search provider.

**Parameters:**
| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| query | string | yes | - | The search query string |
| limit | integer | no | 10 | Maximum number of results (1-100) |
| offset | integer | no | 0 | Pagination offset |
| language | string | no | (none) | Language code (en, es, fr, de, etc.) |
| safe_search | string | no | moderate | Safe search setting (off, moderate, strict) |
| region | string | no | (none) | Region code for localized results |

**Example (STDIO):**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "web_search",
    "arguments": {
      "query": "Rust programming language",
      "limit": 5
    }
  }
}
```

### web_fetch

Fetch and parse a webpage with content extraction.

**Parameters:**
| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| url | string | yes | - | URL to fetch |
| include_raw_html | boolean | no | false | Include raw HTML in response |
| extract_links | boolean | no | true | Extract links from the page |
| extract_images | boolean | no | false | Extract images from the page |
| timeout_secs | integer | no | 30 | Request timeout in seconds |
| max_content_size | integer | no | 10485760 | Max content size in bytes (10MB) |

**Example (STDIO):**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "web_fetch",
    "arguments": {
      "url": "https://example.com",
      "extract_links": true,
      "extract_images": true
    }
  }
}
```

## Search Providers

### DuckDuckGo (Default)

No configuration required. Uses the `ddgs` crate.

```bash
# No config needed
gannet-mcp
```

**Note**: May return CAPTCHA challenges in automated environments.

### Serper API

```bash
export MCP__SEARCH__PROVIDER=serper
export MCP__SEARCH__API_KEY=your_api_key
```

### BrightData

```bash
export MCP__SEARCH__PROVIDER=brightdata
export MCP__SEARCH__API_KEY=your_brightdata_api_key
export MCP__SEARCH__SEARCH_ENGINE_ID=serp_api1
```

### SearXNG

```bash
export MCP__SEARCH__PROVIDER=searxng
export MCP__SEARCH__BASE_URL=http://127.0.0.1:8081
```

## Integration with MCP Clients

### Claude Desktop

Add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "gannet-mcp": {
      "command": "gannet-mcp",
      "args": []
    }
  }
}
```

### Cursor

Already configured via `.cursor/mcp.json`.

### VS Code

Add to `mcp.json` in your workspace:

```json
{
  "mcpServers": {
    "gannet-mcp": {
      "command": "gannet-mcp",
      "args": []
    }
  }
}
```
