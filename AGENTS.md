# AGENTS.md — gannet-mcp

## Commands

```bash
cargo build --release          # release build with LTO
cargo test                     # all tests (unit + integration)
cargo test -- --nocapture      # see log output
cargo test <test_name>         # single test
cargo clippy -- -D warnings    # lint (must pass)
cargo fmt -- --check           # formatting check
make rpm                       # build RPM + SRPM
make srpm                      # build only SRPM
make install-rpm-deps          # install RPM build deps
make clean                     # clean build artifacts
```

## Architecture

- **Binary**: `src/main.rs` — clap CLI, creates tokio runtime, calls `Server::run_stdio()` or `Server::run_http()`
- **Library**: `src/lib.rs` — re-exports `Config`, `Server`, `ServerError`
- **MCP SDK**: `rmcp` v3.1.0 with `server`, `transport-io`, `macros`, `schemars`, `transport-streamable-http-server` features
- **Two tools**: `web_search` and `web_fetch` (defined via `#[tool]` and `#[tool_router]` macros in `src/server.rs`)
- **Pattern**: `handlers/` wrap `services/`; services implement `SearchProvider` trait
- **Dead code**: `src/handler.rs` is a legacy file using the old `mcp_types` crate — kept for reference only; the active code is in `src/handlers/`
- **DuckDuckGo**: uses the `ddgs` crate (not HTML scraping)

## Search providers

| Provider | Config key | Requires |
|----------|-----------|---------|
| duckduckgo | `duckduckgo` (default) | nothing (uses `ddgs` crate) |

| serper | `serper` | `api_key` |
| searxng | `searxng` | `base_url` (default `http://127.0.0.1:8888`) |
| brightdata | `brightdata` | `api_key` + `search_engine_id` (zone) |

**Known gotcha**: DuckDuckGo (via `ddgs`) frequently returns CAPTCHA challenges (bot detection). In CI or automated environments, prefer `serper`/`searxng`/`brightdata`.

See also:
- [`search_provider.md`](./search_provider.md) — detailed guide for getting free API keys (Serper, Bright Data)
- [`free_search_providers.md`](./free_search_providers.md) — comparison of all free search providers with free tiers

## Configuration

Config loaded from (in order): CLI args → env vars → config file (YAML/JSON via the `config` crate).
Env vars use `MCP__` prefix with `__` as separator (e.g., `MCP__SEARCH__PROVIDER=brightdata`).
Copy `.env.example` → `.env` for local overrides.

### Transport modes

- **STDIO** (default): Run with `MCP__SERVER__USE_STDIO=true` (or no flags). This is the recommended mode for MCP clients.
- **Streamable HTTP**: Run with `--http` flag or `MCP__SERVER__USE_STDIO=false`. Serves MCP over HTTP at the configured path (default `/mcp`).
  - `--mcp-path <path>` overrides the endpoint path (default `/mcp`).
  - HTTP config options: `MCP__SERVER__HTTP__PATH`, `MCP__SERVER__HTTP__SSE_KEEP_ALIVE_SECS`, `MCP__SERVER__HTTP__JSON_RESPONSE`, `MCP__SERVER__HTTP__ALLOWED_HOSTS`.

## Tests

- **Unit tests**: alongside source (e.g., `src/services/fetch_service.rs`, `src/handlers/search_handler.rs`)
- **Integration tests**: in `tests/` — `integration_test.rs` spawns real binary via `assert_cmd` and speaks MCP JSON-RPC over STDIO
- **Fetch tests**: `tests/fetch_service_test.rs` uses `mockito` for HTTP mocking
- **Search provider tests**: `tests/search_provider_test.rs` — tests service creation, handler logic, config validation; live DuckDuckGo calls may fail in CI (CAPTCHA/network) — tests accept either success or network error

## RPM packaging (Fedora)

- `gannet-mcp.spec` — RPM spec file using `%cargo` macros
- `make rpm` — build RPM and SRPM into `rpmbuild/`
- `make srpm` — build only SRPM
- `make install-rpm-deps` — install required packages (`rpm-build`, `rust-srpm-macros`)
- **Prerequisites**: `sudo dnf install rpm-build rust cargo rust-srpm-macros`
- The spec expects a tarball named `gannet-mcp-<version>.tar.gz` in `rpmbuild/SOURCES/`
- The Makefile generates this tarball automatically from the current git HEAD (or from the working tree if not a git repo)

## IDE integration

- `mcp.json` and `.cursor/mcp.json` register this as an MCP server
- Default config in those files uses searxng at `http://127.0.0.1:8081` (override for other providers)
