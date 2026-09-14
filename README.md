# gannet-mcp

A Model Context Protocol (MCP) server implementation in Rust for web searching and webpage fetching.

## Features

- 🔍 **Web Search**: Search the web using multiple providers
  - DuckDuckGo (default, no API key required — uses `ddgs` crate)
  - BrightData (generic SERP API)
  - Serper API
  - SearXNG (self-hosted, open-source metasearch)
  
- 🌐 **Webpage Fetching**: Fetch and parse webpages with content extraction
  - HTML parsing with scraper
  - Link and image extraction
  - Metadata extraction (title, description)
  - Content size limits and timeouts

- ⚡ **Performance**
  - Async/await architecture with Tokio
  - Connection pooling
  - Rate limiting
  - Configurable timeouts

- 🔒 **Security**
  - URL validation
  - Content size limits
  - Rate limiting
  - Safe search options

## Architecture

```
gannet-mcp/
├── Cargo.toml              # Project dependencies
├── src/
│   ├── main.rs             # Binary entry point
│   ├── lib.rs              # Library root
│   ├── service.rs            # `service` subcommand (OS service install/run/status)
│   ├── mcp_config.rs         # `mcp-config` subcommand (client JSON snippet)
│   ├── server.rs           # MCP server (rmcp 3.x #[tool]/#[tool_router])
│   ├── config.rs           # Configuration management
 │   ├── error.rs            # Error types
│   ├── models/             # Data models
│   │   ├── mod.rs
│   │   ├── search.rs       # Search request/response models
│   │   ├── content.rs      # Webpage content models
│   │   └── config.rs       # Extended config structures
│   ├── services/           # External service integrations
│   │   ├── mod.rs
│   │   ├── search_service.rs   # Search providers (DDG, BrightData, Serper, SearXNG)
│   │   └── fetch_service.rs    # Webpage fetching service
│   └── handlers/           # MCP protocol handlers (active)
│       ├── mod.rs
│       ├── search_handler.rs   # Search tool handler
│       └── fetch_handler.rs    # Fetch tool handler
├── tests/
│   ├── integration_test.rs         # STDIO JSON-RPC integration tests
│   ├── fetch_service_test.rs       # Mocked fetch tests (mockito)
│   └── search_provider_test.rs     # Search provider + config tests
├── .env.example            # Environment variables template
├── mcp.json                # IDE MCP registration (searxng)
├── .cursor/mcp.json        # Cursor IDE MCP registration
├── free_search_providers.md  # Comparison of free search providers
├── search_provider.md        # Guide to getting free API keys
└── README.md               # This file
```

## Installation

### Prerequisites

- Rust 1.70+ (with Cargo)
- OpenSSL (for TLS support)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/reinartz/gannet-mcp.git
cd gannet-mcp

# Build release version
cargo build --release

# The binary will be at target/release/gannet-mcp
```

### Installation with Cargo

```bash
cargo install --path .
```

## Configuration

The server can be configured via:

1. **Configuration file** (YAML or JSON via the `config` crate)
2. **Environment variables** (prefixed with `MCP__`)
3. **Command-line arguments**

### Configuration File

Create a `config.yaml`:

```yaml
server:
  use_stdio: true
  host: "127.0.0.1"
  port: 8080

search:
  provider: "duckduckgo"
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

### Environment Variables

See `.env.example` for all available options:

```bash
cp .env.example .env
# Edit .env with your configuration
```

### Command Line

```bash
gannet-mcp --help
```

## Usage

### STDIO Mode (Recommended for MCP)

```bash
# Run with default settings
gannet-mcp

# With custom log level
gannet-mcp --log-level debug

# Using config file
gannet-mcp --config config.yaml
```

### HTTP Mode

```bash
# Serve MCP over Streamable HTTP (default binds localhost)
gannet-mcp --http

# Custom endpoint path
gannet-mcp --http --mcp-path /api/mcp
```

### OS Service (HTTP daemon)

```bash
sudo gannet-mcp service install   # install + enable + start (systemd/launchd/SCM)
gannet-mcp service status         # no root required
sudo gannet-mcp service restart   # start | stop | uninstall likewise
```

`install` points the OS supervisor at the hidden `gannet-mcp service run`
entry point, which always serves HTTP (never STDIO).

### MCP Client Config

```bash
gannet-mcp mcp-config --client claude --stdio --print   # emit JSON snippet
```

Supported clients: `claude | cursor | zed | opencode | continue | copilot`;
`--stdio` (default) or `--http`; `--print` (default) or `--write`.

> **Security note:** HTTP mode has no authentication in v0.1.0 (bearer-token
> auth is a planned follow-up). Bind localhost (the default) and expose it
> via a reverse proxy with TLS if remote access is needed. Avoid binding
> `0.0.0.0` on an untrusted network.

### MCP Tools

The server provides two main tools:

#### `web_search`

Search the web for information.

```json
{
  "name": "web_search",
  "arguments": {
    "query": "rust programming language",
    "limit": 10,
    "language": "en"
  }
}
```

Parameters:
- `query` (required): The search query string
- `limit` (optional): Maximum results (default: 10, max: 100)
- `offset` (optional): Pagination offset
- `language` (optional): Language code (en, es, fr, etc.)
- `safe_search` (optional): off, moderate, strict
- `region` (optional): Region code for localized results

#### `web_fetch`

Fetch and parse a webpage.

```json
{
  "name": "web_fetch",
  "arguments": {
    "url": "https://example.com",
    "extract_links": true,
    "extract_images": false
  }
}
```

Parameters:
- `url` (required): URL to fetch
- `include_raw_html` (optional): Include raw HTML in response
- `extract_links` (optional): Extract links (default: true)
- `extract_images` (optional): Extract images (default: false)
- `timeout_secs` (optional): Request timeout (default: 30)
- `max_content_size` (optional): Max content size in bytes (default: 10MB)

## Search Providers

### DuckDuckGo (Default)

No configuration required. Uses the `ddgs` crate (not HTML scraping).

### Serper API

1. Get an API key from [Serper](https://serper.dev/) (2,500 free queries, no credit card)
2. Configure:

```bash
export MCP__SEARCH__PROVIDER=serper
export MCP__SEARCH__API_KEY=your_api_key
```

### SearXNG (Self-Hosted)

1. Run SearXNG via Docker: `docker run -d -p 8081:8080 searxng/searxng`
2. Configure:

```bash
export MCP__SEARCH__PROVIDER=searxng
export MCP__SEARCH__BASE_URL=http://127.0.0.1:8081
```

### BrightData

Generic Bright Data SERP API (works with any Bright Data SERP zone).

```bash
export MCP__SEARCH__PROVIDER=brightdata
export MCP__SEARCH__API_KEY=your_brightdata_api_key
export MCP__SEARCH__SEARCH_ENGINE_ID=serp_api1
```

## Documentation

- [`search_provider.md`](./search_provider.md) — Detailed guide for getting free API keys (Serper, Bright Data)
- [`free_search_providers.md`](./free_search_providers.md) — Comparison of all free search providers with free tiers

## Development

### Project Structure

The codebase follows Rust best practices:

- **Models**: Pure data structures with serialization
- **Services**: Business logic and external integrations (implement `SearchProvider` trait)
- **Handlers**: MCP protocol request handling
- **Server**: Core MCP server implementation using `rmcp` v3.x macros (`#[tool]`, `#[tool_router]`)

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Code Formatting

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check
```

### Linting

```bash
# Run clippy
cargo clippy -- -D warnings
```

## Error Handling

The server uses a custom error type `ServerError` with the following variants:

- `Network`: HTTP/network errors
- `UrlParse`: Invalid URL format
- `JsonSerialize`: JSON serialization errors
- `Config`: Configuration errors
- `SearchApi`: Search provider API errors
- `RateLimitExceeded`: Rate limit exceeded
- `InvalidRequest`: Invalid request parameters
- `NotFound`: Resource not found
- `Timeout`: Request timeout
- `McpProtocol`: MCP protocol errors

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and linting
5. Submit a pull request

## License

Dual-licensed under MIT OR Apache-2.0 — see LICENSE-MIT and LICENSE-APACHE files for details.

## Acknowledgments

Built with:
- [Tokio](https://tokio.rs/) - Async runtime
- [Reqwest](https://docs.rs/reqwest/) - HTTP client
- [Scraper](https://docs.rs/scraper/) - HTML parsing
- [rmcp](https://crates.io/crates/rmcp) - MCP protocol implementation (v3.x)
- [ddgs](https://crates.io/crates/ddgs) - DuckDuckGo search client
- [clap](https://docs.rs/clap/) - CLI argument parsing
- [config](https://docs.rs/config/) - Configuration management