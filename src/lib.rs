// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Ole Reinartz
//! # gannet_mcp
//!
//! A Model Context Protocol (MCP) server for web searching and webpage fetching.
//!
//! ## Features
//!
//! - **Web Search** via multiple providers: DuckDuckGo, Serper, SearXNG
//! - **Webpage Fetching** with HTML parsing and content extraction
//! - **Link & Image Extraction** from fetched pages
//! - **Multiple transport modes**: STDIO (default) and Streamable HTTP
//! - **Rate Limiting** and configurable timeouts
//!
//! ## Usage
//!
//! Run as a STDIO-based MCP server:
//!
//! ```bash
//! gannet-mcp
//! ```
//!
//! Or as an HTTP server:
//!
//! ```bash
//! gannet-mcp --http
//! ```
//!
//! ## Library Usage
//!
//! ```rust,no_run
//! use gannet_mcp::{Server, Config};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = Config::default();
//!     let server = Server::new(config).await?;
//!     server.run_stdio().await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Search Providers
//!
//! | Provider | Config | Requirements |
//! |----------|--------|-------------|
//! | DuckDuckGo | `duckduckgo` | None (uses `ddgs` crate) |
//! | Bright Data | `brightdata` | `api_key` + `search_engine_id` (zone) |
//! | Serper | `serper` | `api_key` |
//! | SearXNG | `searxng` | `base_url` |

pub mod config;
pub mod error;
pub mod handlers;
pub mod mcp_config;
pub mod models;
pub mod server;
pub mod service;
pub mod services;

// Re-export commonly used types
pub use config::{Config, HttpConfig, LoggingConfig, RateLimitConfig, SearchConfig, ServerConfig};
pub use error::{ServerError, ServerResult};
pub use server::Server;

// Version information
#[allow(dead_code)]
const VERSION: &str = env!("CARGO_PKG_VERSION");
#[allow(dead_code)]
const NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_info() {
        assert!(!VERSION.is_empty());
        assert!(!NAME.is_empty());
    }
}
