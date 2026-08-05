# Configuration Reference

gannet-mcp can be configured via three methods (in order of precedence):

1. **Command-line arguments** (highest priority)
2. **Environment variables** (middle priority)
3. **Configuration file** (YAML/JSON, lowest priority)
4. **Default values** (base configuration)

## Environment Variables

All environment variables use the `MCP__` prefix with `__` as separator between nested keys.

| Environment Variable | Type | Default | Description |
|---------------------|------|---------|-------------|
| `MCP__SERVER__USE_STDIO` | boolean | `true` | Use STDIO transport (false = HTTP) |
| `MCP__SERVER__HOST` | string | `127.0.0.1` | Bind address |
| `MCP__SERVER__PORT` | integer | `8080` | Bind port (HTTP mode) |
| `MCP__SERVER__HTTP__PATH` | string | `/mcp` | MCP endpoint path |
| `MCP__SERVER__HTTP__SSE_KEEP_ALIVE_SECS` | integer | `15` | SSE heartbeat interval |
| `MCP__SERVER__HTTP__JSON_RESPONSE` | boolean | `true` | Return JSON responses |
| `MCP__SERVER__HTTP__ALLOWED_HOSTS` | string | (none) | Comma-separated allowed hosts |
| `MCP__SEARCH__PROVIDER` | string | `duckduckgo` | Search provider |
| `MCP__SEARCH__API_KEY` | string | (none) | API key for Serper/BrightData |
| `MCP__SEARCH__SEARCH_ENGINE_ID` | string | (none) | Bright Data zone  |
| `MCP__SEARCH__BASE_URL` | string | `http://127.0.0.1:8888` | SearXNG base URL |
| `MCP__SEARCH__DEFAULT_LIMIT` | integer | `10` | Default search result limit |
| `MCP__SEARCH__MAX_LIMIT` | integer | `100` | Maximum search result limit |
| `MCP__SEARCH__TIMEOUT_SECS` | integer | `10` | Search request timeout |
| `MCP__RATE_LIMIT__ENABLED` | boolean | `true` | Enable rate limiting |
| `MCP__RATE_LIMIT__REQUESTS_PER_MINUTE` | integer | `60` | Max requests per minute |
| `MCP__RATE_LIMIT__MAX_CONCURRENT` | integer | `10` | Max concurrent requests |
| `MCP__LOGGING__LEVEL` | string | `info` | Log level (trace/debug/info/warn/error) |
| `MCP__LOGGING__FORMAT` | string | `pretty` | Log format (pretty/json) |

## Configuration File

Create a `config.yaml` or `config.json` file:

### YAML Example

```yaml
server:
  use_stdio: true
  host: "127.0.0.1"
  port: 8080
  http:
    path: "/mcp"
    sse_keep_alive_secs: 15
    json_response: true
    allowed_hosts: []

search:
  provider: "duckduckgo"
  api_key: ""
  search_engine_id: ""
  base_url: "http://127.0.0.1:8888"
  default_limit: 10
  max_limit: 100
  timeout_secs: 10

rate_limit:
  enabled: true
  requests_per_minute: 60
  max_concurrent: 10

logging:
  level: "info"
  format: "pretty"
```

### JSON Example

```json
{
  "server": {
    "use_stdio": true,
    "host": "127.0.0.1",
    "port": 8080,
    "http": {
      "path": "/mcp",
      "sse_keep_alive_secs": 15,
      "json_response": true,
      "allowed_hosts": []
    }
  },
  "search": {
    "provider": "duckduckgo",
    "api_key": "",
    "search_engine_id": "",
    "base_url": "http://127.0.0.1:8888",
    "default_limit": 10,
    "max_limit": 100,
    "timeout_secs": 10
  },
  "rate_limit": {
    "enabled": true,
    "requests_per_minute": 60,
    "max_concurrent": 10
  },
  "logging": {
    "level": "info",
    "format": "pretty"
  }
}
```

### Config File Path

Specify a config file via CLI:

```bash
gannet-mcp --config /path/to/config.yaml
```

The config crate searches for config files in the following order:

1. Current directory (`config.yaml`, `config.json`)
2. User home directory (`~/.config/gannet-mcp/config.yaml`)
3. Path specified by `--config` argument

## Search Provider Configuration

### DuckDuckGo

```yaml
search:
  provider: "duckduckgo"
```

No additional configuration needed.

### Serper API

```yaml
search:
  provider: "serper"
  api_key: "your_api_key"
```

### BrightData

```yaml
search:
  provider: "brightdata"
  api_key: "your_brightdata_api_key"
  search_engine_id: "serp_api1"
```

### SearXNG

```yaml
search:
  provider: "searxng"
  base_url: "http://127.0.0.1:8888"
```

## Environment Variables Template

Copy `.env.example` for all available options:

```bash
cp .env.example .env
# Edit .env with your configuration
```

## Configuration Precedence

When the same setting is specified in multiple places:

```
CLI args > Environment vars > Config file > Defaults
```

Example:

```bash
# This sets the provider to "serper", overriding both the config file and defaults
gannet-mcp --search-provider serper
```

## Validation

Invalid configurations will cause the server to exit with an error:

- Unknown search provider name
- Missing API key for providers that require one (Serper, BrightData)
- Invalid URL format for SearXNG base URL
- Non-numeric values for integer fields
- Out-of-range values (e.g., limit > 100 or < 1)
