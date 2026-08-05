# Architecture Overview

This document describes the architecture of gannet-mcp.

## High-Level Design

gannet-mcp is an MCP (Model Context Protocol) server that provides two tools to LLM clients:

1. **`web_search`** - Search the web using multiple providers
2. **`web_fetch`** - Fetch and parse webpages

The server supports two transport modes:
- **STDIO** (default): Communicates over stdin/stdout for MCP clients
- **Streamable HTTP**: Serves MCP over HTTP for web-based clients

## Project Structure

```
gannet-mcp/
├── src/
│   ├── main.rs             # Binary entry point (clap CLI, tokio runtime)
│   ├── lib.rs              # Library root (re-exports Config, Server, ServerError)
│   ├── config.rs           # Configuration management (all config structs)
│   ├── error.rs            # Error types (ServerError enum)
│   ├── server.rs           # MCP server (rmcp 3.x #[tool]/#[tool_router])
│   ├── handler.rs          # (legacy) Old mcp_types-based handler — kept for reference
│   ├── models/             # Data models
│   │   ├── mod.rs
│   │   ├── search.rs       # Search request/response models
│   │   ├── content.rs      # Webpage content models
│   │   └── config.rs       # Extended config structures
│   ├── services/           # External service integrations
│   │   ├── mod.rs
│   │   ├── search_service.rs   # Search providers (SearchProvider trait)
│   │   └── fetch_service.rs    # Webpage fetching service
│   └── handlers/           # MCP protocol handlers (active)
│       ├── mod.rs
│       ├── search_handler.rs   # Search tool handler
│       └── fetch_handler.rs    # Fetch tool handler
├── tests/
│   ├── integration_test.rs         # STDIO JSON-RPC integration tests
│   ├── fetch_service_test.rs       # Mocked fetch tests (mockito)
│   └── search_provider_test.rs     # Search provider + config tests
└── .github/                      # GitHub configuration
    ├── workflows/ci.yml          # CI/CD pipeline
    ├── ISSUE_TEMPLATE/           # Issue templates
    └── PULL_REQUEST_TEMPLATE.md  # PR template
```

## Component Layers

### 1. Binary Layer (`src/main.rs`)

The entry point that:
- Parses CLI arguments with `clap`
- Creates a Tokio async runtime
- Instantiates `Server` and calls `run_stdio()` or `run_http()`

### 2. MCP Server Layer (`src/server.rs`)

Uses `rmcp` v3.x with `#[tool]` and `#[tool_router]` macros to define:
- `web_search` tool
- `web_fetch` tool
- MCP lifecycle (initialize, tools/list, tools/call)

### 3. Handler Layer (`src/handlers/`)

Wraps services and handles MCP protocol details:
- `search_handler.rs` - Validates search parameters, calls search service
- `fetch_handler.rs` - Validates URL, calls fetch service

### 4. Service Layer (`src/services/`)

Implements business logic:
- `search_service.rs` - `SearchProvider` trait with provider implementations
  - `DuckDuckGoProvider` - Uses `ddgs` crate
  - `BingProvider` - Bing Search API
  - `SerperProvider` - Serper API
  - `SearXNGProvider` - SearXNG API
- `fetch_service.rs` - HTTP fetching, HTML parsing with `scraper`

### 5. Model Layer (`src/models/`)

Pure data structures:
- `search.rs` - SearchRequest, SearchResponse, SearchResult
- `content.rs` - FetchRequest, FetchResponse, ContentItem
- `config.rs` - Extended configuration structures

### 6. Configuration Layer (`src/config.rs`)

Configuration management using the `config` crate:
- `Config` - Top-level configuration
- `ServerConfig`, `HttpConfig`, `SearchConfig`, `RateLimitConfig`, `LoggingConfig`
- Supports YAML, JSON, environment variables, and CLI args

### 7. Error Layer (`src/error.rs`)

Custom error type `ServerError` with variants:
- `Network`, `UrlParse`, `JsonSerialize`, `Config`, `SearchApi`
- `RateLimitExceeded`, `InvalidRequest`, `NotFound`, `Timeout`, `McpProtocol`

## Data Flow

### web_search

```
MCP Client
    │
    ▼
tools/call (web_search)
    │
    ▼
SearchHandler::handle()
    │
    ├── Validate parameters (query, limit, language)
    │
    ▼
SearchService::search()
    │
    ├── Provider-specific implementation (DuckDuckGo etc.)
    │
    ▼
SearchResponse
    │
    ▼
MCP Client
```

### web_fetch

```
MCP Client
    │
    ▼
tools/call (web_fetch)
    │
    ▼
FetchHandler::handle()
    │
    ├── Validate URL (http/https)
    │
    ▼
FetchService::fetch()
    │
    ├── HTTP request with timeout
    ├── HTML parsing with scraper
    ├── Link/image extraction (if requested)
    │
    ▼
FetchResponse
    │
    ▼
MCP Client
```

## Transport Modes

### STDIO Mode

```
MCP Client ── stdin ──► [gannet-mcp] ── stdout ──► MCP Client
                        (tokio runtime)
```

- JSON-RPC 2.0 messages over stdin/stdout
- Line-delimited JSON
- Default mode, recommended for MCP clients

### HTTP Mode

```
MCP Client ── HTTP ──► [axum server] ──► [rmcp handler]
                          (Streamable HTTP)
```

- HTTP server using `axum`
- Streamable HTTP transport
- SSE support for event streaming
- Configurable endpoint path

## External Dependencies

| Dependency | Purpose |
|-----------|---------|
| `rmcp` v3.1.0 | MCP protocol implementation |
| `tokio` | Async runtime |
| `reqwest` | HTTP client |
| `scraper` | HTML parsing |
| `ddgs` | DuckDuckGo search |
| `clap` | CLI argument parsing |
| `config` | Configuration management |
| `axum` | HTTP server |
| `serde` | Serialization |
| `thiserror` | Error types |
| `schemars` | JSON Schema generation |

## Testing Strategy

- **Unit tests**: Alongside source files in `src/`
- **Service tests**: `tests/fetch_service_test.rs` uses `mockito` for HTTP mocking
- **Integration tests**: `tests/integration_test.rs` spawns real binary via `assert_cmd`
- **Provider tests**: `tests/search_provider_test.rs` tests service creation and config
