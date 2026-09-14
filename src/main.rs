// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Ole Reinartz
//!
//! Web Search MCP Server - Main Entry Point
//!
//! This binary provides a command-line interface for the web search MCP server.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use tracing::{info, Level};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use gannet_mcp::{mcp_config, service, Config, LoggingConfig, Server};

/// Web Search MCP Server
///
/// A Model Context Protocol server for web searching and webpage fetching
#[derive(Parser, Debug)]
#[command(name = "gannet-mcp")]
#[command(version, about = "MCP server for web search and fetch operations", long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long, value_name = "FILE", global = true)]
    config: Option<PathBuf>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info", global = true)]
    log_level: Level,

    /// Log format (pretty, json, compact)
    #[arg(long, default_value = "pretty", global = true)]
    log_format: LogFormat,

    /// Include timestamps in log output
    #[arg(long, global = true)]
    include_timestamps: bool,

    /// Host to bind to (default: 127.0.0.1)
    #[arg(long)]
    host: Option<String>,

    /// Port to listen on (default: 8080)
    #[arg(long)]
    port: Option<u16>,

    /// Path to Unix socket file
    #[arg(long)]
    socket_path: Option<PathBuf>,

    /// Use STDIO transport (default: true)
    #[arg(long)]
    stdio: bool,

    /// Run in HTTP mode (Streamable HTTP transport)
    #[arg(long)]
    http: bool,

    /// MCP endpoint path for HTTP mode (default: /mcp)
    #[arg(long)]
    mcp_path: Option<String>,

    /// SSE keep-alive interval in seconds for HTTP mode
    #[arg(long)]
    http_sse_keep_alive: Option<u64>,

    /// Prefer JSON responses for HTTP mode (default: true)
    #[arg(long)]
    json_response: bool,

    /// CORS allowed origin for HTTP mode (default: *)
    #[arg(long)]
    cors_origin: Option<String>,

    /// Allowed hosts for HTTP mode Host header validation (comma-separated)
    #[arg(long)]
    allowed_hosts: Option<String>,

    /// Legacy session mode for HTTP mode
    #[arg(long)]
    legacy_session_mode: bool,

    /// Search provider (duckduckgo, serper, searxng, brightdata)
    #[arg(long)]
    provider: Option<String>,

    /// API key for search provider
    #[arg(long)]
    api_key: Option<String>,

    /// Bright Data SERP zone (e.g. "serp_api1")
    #[arg(long)]
    search_engine_id: Option<String>,

    /// Base URL for custom search API (e.g. SearXNG)
    #[arg(long)]
    base_url: Option<String>,

    /// Default number of search results (default: 10)
    #[arg(long)]
    search_limit: Option<u32>,

    /// Maximum results per search query
    #[arg(long)]
    max_results: Option<u32>,

    /// Search request timeout in seconds (default: 10)
    #[arg(long)]
    search_timeout: Option<u64>,

    /// Enable rate limiting (default: true)
    #[arg(long)]
    rate_limited: bool,

    /// Maximum requests per minute (default: 60)
    #[arg(long)]
    requests_per_minute: Option<u32>,

    /// Maximum concurrent requests (default: 10)
    #[arg(long)]
    max_concurrent: Option<u32>,

    /// Burst size for token bucket (default: 5)
    #[arg(long)]
    burst_size: Option<u32>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Manage the gannet-mcp OS service (HTTP daemon)
    Service {
        #[command(subcommand)]
        action: service::ServiceAction,
    },
    /// Emit an MCP client config snippet
    McpConfig(mcp_config::McpConfigArgs),
}

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum LogFormat {
    /// Human-readable formatted output
    Pretty,
    /// JSON formatted output
    Json,
    /// Compact formatted output
    Compact,
}

impl std::fmt::Display for LogFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogFormat::Pretty => write!(f, "pretty"),
            LogFormat::Json => write!(f, "json"),
            LogFormat::Compact => write!(f, "compact"),
        }
    }
}

fn main() {
    if let Err(e) = run() {
        // Plain stderr: subcommand paths (service/mcp-config) never initialize
        // the tracing subscriber, so `error!` would be silently swallowed.
        eprintln!("gannet-mcp: error: {e:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Args::parse();

    match &args.command {
        // Hidden daemon entry via the OS supervisor: same as the legacy HTTP
        // path but forced to HTTP, honoring global flags (--config, --port…).
        Some(Command::Service {
            action: service::ServiceAction::Run,
        }) => {
            let mut config = load_config(&args)?;
            service::apply_run_overrides(&mut config);
            run_server(config)
        }
        Some(Command::Service { action }) => service::run_service(action),
        Some(Command::McpConfig(mc_args)) => mcp_config::run_mcp_config(mc_args),
        // Legacy path: no subcommand, STDIO by default.
        None => {
            let config = load_config(&args)?;
            run_server(config)
        }
    }
}

fn run_server(config: Config) -> Result<()> {
    // Initialize logging from config (which already has CLI overrides applied)
    init_logging(&config.logging)?;

    info!(
        "Starting Web Search MCP Server v{}",
        env!("CARGO_PKG_VERSION")
    );

    // Create and run server
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("Failed to create Tokio runtime")?;

    runtime.block_on(async {
        let use_stdio = config.server.use_stdio;

        let server = Server::new(config)
            .await
            .context("Failed to initialize server")?;

        if use_stdio {
            server.run_stdio().await?;
        } else {
            server.run_http().await?;
        }
        Ok(())
    })
}

fn init_logging(logging: &LoggingConfig) -> Result<()> {
    // Parse log level from config
    let level = match logging.level.as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("gannet_mcp={}", level)));

    let subscriber = tracing_subscriber::registry().with(env_filter);

    match logging.format.as_str() {
        "json" => {
            subscriber
                .with(fmt::layer().with_writer(std::io::stderr).json())
                .init();
        }
        "compact" => {
            subscriber
                .with(fmt::layer().with_writer(std::io::stderr).compact())
                .init();
        }
        _ => {
            subscriber
                .with(fmt::layer().with_writer(std::io::stderr).pretty())
                .init();
        }
    }

    Ok(())
}

pub(crate) fn load_config(args: &Args) -> Result<Config> {
    let mut config = if let Some(ref config_path) = args.config {
        Config::from_file(config_path)?
    } else {
        Config::from_env().unwrap_or_default()
    };

    // Override with command-line arguments
    config.logging.level = args.log_level.to_string().to_lowercase();

    config.logging.format = args.log_format.to_string();

    config.logging.include_timestamps = args.include_timestamps;

    if let Some(ref host) = args.host {
        config.server.host = host.clone();
    }

    if let Some(p) = args.port {
        config.server.port = p;
    }

    if let Some(ref socket_path) = args.socket_path {
        config.server.socket_path = Some(socket_path.clone());
    }

    if args.stdio {
        config.server.use_stdio = true;
    }

    if args.http {
        config.server.use_stdio = false;
    }

    config.server.http.path = args.mcp_path.clone().unwrap_or(config.server.http.path);

    if let Some(sse) = args.http_sse_keep_alive {
        config.server.http.sse_keep_alive_secs = Some(sse);
    }

    if args.json_response {
        config.server.http.json_response = true;
    }

    if let Some(ref cors) = args.cors_origin {
        config.server.http.cors_origin = cors.clone();
    }

    if let Some(ref hosts) = args.allowed_hosts {
        config.server.http.allowed_hosts = hosts.split(',').map(|s| s.trim().to_string()).collect();
    }

    if args.legacy_session_mode {
        config.server.http.legacy_session_mode = true;
    }

    if let Some(ref provider) = args.provider {
        config.search.provider = provider.clone();
    }

    if let Some(ref api_key) = args.api_key {
        config.search.api_key = Some(api_key.clone());
    }

    if let Some(ref engine_id) = args.search_engine_id {
        config.search.search_engine_id = Some(engine_id.clone());
    }

    if let Some(ref base) = args.base_url {
        config.search.base_url = Some(base.clone());
    }

    if let Some(limit) = args.search_limit {
        config.search.default_limit = limit;
    }

    if let Some(max) = args.max_results {
        config.search.max_limit = max;
    }

    if let Some(timeout) = args.search_timeout {
        config.search.timeout_secs = timeout;
    }

    if args.rate_limited {
        config.rate_limit.enabled = true;
    }

    if let Some(rpm) = args.requests_per_minute {
        config.rate_limit.requests_per_minute = rpm;
    }

    if let Some(concurrent) = args.max_concurrent {
        config.rate_limit.max_concurrent = concurrent;
    }

    if let Some(burst) = args.burst_size {
        config.rate_limit.burst_size = burst;
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;
    use std::io::Write;

    #[test]
    fn test_args_parser_exists() {
        let mut cmd = Args::command();
        cmd.build();
        assert_eq!(cmd.get_name(), "gannet-mcp");
    }

    #[test]
    fn test_args_defaults() {
        let args = Args::try_parse_from(["gannet-mcp"]).unwrap();
        assert!(args.config.is_none());
        assert_eq!(args.log_level, Level::INFO);
        assert_eq!(args.log_format, LogFormat::Pretty);
        assert!(!args.include_timestamps);
        assert!(args.host.is_none());
        assert!(args.port.is_none());
        assert!(args.socket_path.is_none());
        assert!(!args.stdio);
        assert!(!args.http);
        assert!(args.mcp_path.is_none());
        assert!(args.http_sse_keep_alive.is_none());
        assert!(!args.json_response);
        assert!(args.cors_origin.is_none());
        assert!(args.allowed_hosts.is_none());
        assert!(!args.legacy_session_mode);
        assert!(args.provider.is_none());
        assert!(args.api_key.is_none());
        assert!(args.search_engine_id.is_none());
        assert!(args.base_url.is_none());
        assert!(args.search_limit.is_none());
        assert!(args.max_results.is_none());
        assert!(args.search_timeout.is_none());
        assert!(!args.rate_limited);
        assert!(args.requests_per_minute.is_none());
        assert!(args.max_concurrent.is_none());
        assert!(args.burst_size.is_none());
    }

    #[test]
    fn test_args_all_options() {
        let args = Args::try_parse_from([
            "gannet-mcp",
            "--config",
            "/etc/server.yaml",
            "--log-level",
            "debug",
            "--log-format",
            "json",
            "--include-timestamps",
            "--host",
            "0.0.0.0",
            "--port",
            "3000",
            "--socket-path",
            "/tmp/mcp.sock",
            "--stdio",
            "--http-sse-keep-alive",
            "60",
            "--json-response",
            "--cors-origin",
            "https://example.com",
            "--allowed-hosts",
            "localhost,example.com",
            "--legacy-session-mode",
            "--provider",
            "brightdata",
            "--api-key",
            "test-key-123",
            "--search-engine-id",
            "serp_api1",
            "--base-url",
            "http://custom.local",
            "--search-limit",
            "25",
            "--max-results",
            "50",
            "--search-timeout",
            "30",
            "--rate-limited",
            "--requests-per-minute",
            "100",
            "--max-concurrent",
            "20",
            "--burst-size",
            "10",
        ])
        .unwrap();
        assert_eq!(
            args.config.unwrap(),
            std::path::PathBuf::from("/etc/server.yaml")
        );
        assert_eq!(args.log_level, Level::DEBUG);
        assert_eq!(args.log_format, LogFormat::Json);
        assert!(args.include_timestamps);
        assert_eq!(args.host, Some("0.0.0.0".to_string()));
        assert_eq!(args.port, Some(3000));
        assert_eq!(args.socket_path, Some(PathBuf::from("/tmp/mcp.sock")));
        assert!(args.stdio);
        assert_eq!(args.http_sse_keep_alive, Some(60));
        assert!(args.json_response);
        assert_eq!(args.cors_origin, Some("https://example.com".to_string()));
        assert_eq!(
            args.allowed_hosts,
            Some("localhost,example.com".to_string())
        );
        assert!(args.legacy_session_mode);
        assert_eq!(args.provider, Some("brightdata".to_string()));
        assert_eq!(args.api_key, Some("test-key-123".to_string()));
        assert_eq!(args.search_engine_id, Some("serp_api1".to_string()));
        assert_eq!(args.base_url, Some("http://custom.local".to_string()));
        assert_eq!(args.search_limit, Some(25));
        assert_eq!(args.max_results, Some(50));
        assert_eq!(args.search_timeout, Some(30));
        assert!(args.rate_limited);
        assert_eq!(args.requests_per_minute, Some(100));
        assert_eq!(args.max_concurrent, Some(20));
        assert_eq!(args.burst_size, Some(10));
    }

    #[test]
    fn test_log_format_display_pretty() {
        let format = LogFormat::Pretty;
        assert_eq!(format.to_string(), "pretty");
    }

    #[test]
    fn test_log_format_display_json() {
        let format = LogFormat::Json;
        assert_eq!(format.to_string(), "json");
    }

    #[test]
    fn test_log_format_display_compact() {
        let format = LogFormat::Compact;
        assert_eq!(format.to_string(), "compact");
    }

    #[test]
    fn test_log_format_from_str() {
        use clap::ValueEnum;
        assert_eq!(
            LogFormat::from_str("pretty", false).unwrap(),
            LogFormat::Pretty
        );
        assert_eq!(LogFormat::from_str("json", false).unwrap(), LogFormat::Json);
        assert_eq!(
            LogFormat::from_str("compact", false).unwrap(),
            LogFormat::Compact
        );
    }

    #[test]
    fn test_load_config_with_provider_override() {
        let args = Args::try_parse_from(["gannet-mcp", "--provider", "brightdata"]).unwrap();
        let config = load_config(&args).unwrap();
        assert_eq!(config.search.provider, "brightdata");
    }

    #[test]
    fn test_load_config_with_api_key_override() {
        let args = Args::try_parse_from(["gannet-mcp", "--api-key", "my-api-key"]).unwrap();
        let config = load_config(&args).unwrap();
        assert_eq!(config.search.api_key, Some("my-api-key".to_string()));
    }

    #[test]
    fn test_load_config_with_max_results_override() {
        let args = Args::try_parse_from(["gannet-mcp", "--max-results", "50"]).unwrap();
        let config = load_config(&args).unwrap();
        assert_eq!(config.search.max_limit, 50);
    }

    #[test]
    fn test_load_config_default_max_results_no_override() {
        let args = Args::try_parse_from(["gannet-mcp"]).unwrap();
        let config = load_config(&args).unwrap();
        assert_eq!(config.search.max_limit, 100);
    }

    #[test]
    fn test_args_http_flag() {
        let args = Args::try_parse_from(["gannet-mcp", "--http"]).unwrap();
        assert!(args.http);
        let config = load_config(&args).unwrap();
        assert!(!config.server.use_stdio);
    }

    #[test]
    fn test_args_stdio_flag() {
        let args = Args::try_parse_from(["gannet-mcp", "--stdio"]).unwrap();
        assert!(args.stdio);
        let config = load_config(&args).unwrap();
        assert!(config.server.use_stdio);
    }

    #[test]
    fn test_args_mcp_path_flag() {
        let args = Args::try_parse_from(["gannet-mcp", "--mcp-path", "/custom"]).unwrap();
        assert_eq!(args.mcp_path, Some("/custom".to_string()));
        let config = load_config(&args).unwrap();
        assert_eq!(config.server.http.path, "/custom");
    }

    #[test]
    fn test_http_flag_disables_stdio() {
        let args = Args::try_parse_from(["gannet-mcp", "--http"]).unwrap();
        let config = load_config(&args).unwrap();
        assert!(!config.server.use_stdio);
    }

    #[test]
    fn test_mcp_path_flag_overrides_default() {
        let args = Args::try_parse_from(["gannet-mcp", "--mcp-path", "/v1/search"]).unwrap();
        let config = load_config(&args).unwrap();
        assert_eq!(config.server.http.path, "/v1/search");
    }

    #[test]
    fn test_http_and_mcp_path_combined() {
        let args =
            Args::try_parse_from(["gannet-mcp", "--http", "--mcp-path", "/api/mcp"]).unwrap();
        assert!(args.http);
        assert_eq!(args.mcp_path, Some("/api/mcp".to_string()));
        let config = load_config(&args).unwrap();
        assert!(!config.server.use_stdio);
        assert_eq!(config.server.http.path, "/api/mcp");
    }

    #[test]
    fn test_load_config_with_config_file() {
        let tmp = tempfile::Builder::new().suffix(".yaml").tempfile().unwrap();
        writeln!(
            tmp.as_file(),
            "search:\n  provider: serper\n  api_key: file-key\n  max_limit: 20"
        )
        .unwrap();

        let args =
            Args::try_parse_from(["gannet-mcp", "--config", tmp.path().to_str().unwrap()]).unwrap();
        let config = load_config(&args).unwrap();
        assert_eq!(config.search.provider, "serper");
        assert_eq!(config.search.api_key, Some("file-key".to_string()));
        assert_eq!(config.search.max_limit, 20);
    }

    #[test]
    fn test_load_config_cli_overrides_config_file() {
        let tmp = tempfile::Builder::new().suffix(".yaml").tempfile().unwrap();
        writeln!(
            tmp.as_file(),
            "search:\n  provider: brightdata\n  api_key: file-key\n  max_limit: 5"
        )
        .unwrap();

        let args = Args::try_parse_from([
            "gannet-mcp",
            "--config",
            tmp.path().to_str().unwrap(),
            "--provider",
            "duckduckgo",
            "--api-key",
            "cli-key",
        ])
        .unwrap();
        let config = load_config(&args).unwrap();
        assert_eq!(config.search.provider, "duckduckgo");
        assert_eq!(config.search.api_key, Some("cli-key".to_string()));
        assert_eq!(config.search.max_limit, 5);
    }

    #[test]
    fn test_load_config_json_config_file() {
        let tmp = tempfile::Builder::new().suffix(".json").tempfile().unwrap();
        writeln!(
            tmp.as_file(),
            "{{\"search\":{{\"provider\":\"brightdata\",\"api_key\":\"json-key\",\"max_limit\":30}}}}"
        )
        .unwrap();

        let args =
            Args::try_parse_from(["gannet-mcp", "--config", tmp.path().to_str().unwrap()]).unwrap();
        let config = load_config(&args).unwrap();
        assert_eq!(config.search.provider, "brightdata");
        assert_eq!(config.search.api_key, Some("json-key".to_string()));
        assert_eq!(config.search.max_limit, 30);
    }

    #[test]
    fn test_load_config_combined_overrides() {
        let tmp = tempfile::Builder::new().suffix(".yaml").tempfile().unwrap();
        writeln!(
            tmp.as_file(),
            "search:\n  provider: serper\n  api_key: file-key\n  max_limit: 5"
        )
        .unwrap();

        let args = Args::try_parse_from([
            "gannet-mcp",
            "--config",
            tmp.path().to_str().unwrap(),
            "--provider",
            "searxng",
            "--api-key",
            "override-key",
            "--max-results",
            "75",
        ])
        .unwrap();
        let config = load_config(&args).unwrap();
        assert_eq!(config.search.provider, "searxng");
        assert_eq!(config.search.api_key, Some("override-key".to_string()));
        assert_eq!(config.search.max_limit, 75);
    }

    #[test]
    fn test_args_port_flag() {
        let args = Args::try_parse_from(["gannet-mcp", "--port", "9999"]).unwrap();
        assert_eq!(args.port, Some(9999));
        let config = load_config(&args).unwrap();
        assert_eq!(config.server.port, 9999);
    }

    #[test]
    fn test_args_http_port_combined() {
        let args = Args::try_parse_from([
            "gannet-mcp",
            "--http",
            "--port",
            "7777",
            "--mcp-path",
            "/api",
        ])
        .unwrap();
        assert!(args.http);
        assert_eq!(args.port, Some(7777));
        assert_eq!(args.mcp_path, Some("/api".to_string()));
        let config = load_config(&args).unwrap();
        assert!(!config.server.use_stdio);
        assert_eq!(config.server.port, 7777);
        assert_eq!(config.server.http.path, "/api");
    }

    #[test]
    fn test_load_config_host_override() {
        let args = Args::try_parse_from(["gannet-mcp", "--host", "0.0.0.0"]).unwrap();
        let config = load_config(&args).unwrap();
        assert_eq!(config.server.host, "0.0.0.0");
    }

    #[test]
    fn test_load_config_socket_path_override() {
        let args = Args::try_parse_from(["gannet-mcp", "--socket-path", "/tmp/mcp.sock"]).unwrap();
        let config = load_config(&args).unwrap();
        assert_eq!(
            config.server.socket_path,
            Some(PathBuf::from("/tmp/mcp.sock"))
        );
    }

    #[test]
    fn test_load_config_search_engine_id_override() {
        let args =
            Args::try_parse_from(["gannet-mcp", "--search-engine-id", "engine-123"]).unwrap();
        let config = load_config(&args).unwrap();
        assert_eq!(
            config.search.search_engine_id,
            Some("engine-123".to_string())
        );
    }

    #[test]
    fn test_load_config_base_url_override() {
        let args =
            Args::try_parse_from(["gannet-mcp", "--base-url", "http://custom.local"]).unwrap();
        let config = load_config(&args).unwrap();
        assert_eq!(
            config.search.base_url,
            Some("http://custom.local".to_string())
        );
    }

    #[test]
    fn test_load_config_search_limit_override() {
        let args = Args::try_parse_from(["gannet-mcp", "--search-limit", "25"]).unwrap();
        let config = load_config(&args).unwrap();
        assert_eq!(config.search.default_limit, 25);
    }

    #[test]
    fn test_load_config_search_timeout_override() {
        let args = Args::try_parse_from(["gannet-mcp", "--search-timeout", "30"]).unwrap();
        let config = load_config(&args).unwrap();
        assert_eq!(config.search.timeout_secs, 30);
    }

    #[test]
    fn test_load_config_rate_limit_overrides() {
        let args = Args::try_parse_from([
            "gannet-mcp",
            "--requests-per-minute",
            "100",
            "--max-concurrent",
            "20",
            "--burst-size",
            "10",
        ])
        .unwrap();
        let config = load_config(&args).unwrap();
        assert_eq!(config.rate_limit.requests_per_minute, 100);
        assert_eq!(config.rate_limit.max_concurrent, 20);
        assert_eq!(config.rate_limit.burst_size, 10);
    }

    #[test]
    fn test_load_config_http_options() {
        let args = Args::try_parse_from([
            "gannet-mcp",
            "--http",
            "--mcp-path",
            "/api",
            "--http-sse-keep-alive",
            "60",
            "--cors-origin",
            "https://example.com",
        ])
        .unwrap();
        let config = load_config(&args).unwrap();
        assert!(!config.server.use_stdio);
        assert_eq!(config.server.http.path, "/api");
        assert_eq!(config.server.http.sse_keep_alive_secs, Some(60));
        assert_eq!(config.server.http.cors_origin, "https://example.com");
    }

    #[test]
    fn test_load_config_log_level_override() {
        let args = Args::try_parse_from(["gannet-mcp", "--log-level", "debug"]).unwrap();
        let config = load_config(&args).unwrap();
        assert_eq!(config.logging.level, "debug");
    }

    #[test]
    fn test_load_config_include_timestamps_override() {
        let args = Args::try_parse_from(["gannet-mcp", "--include-timestamps"]).unwrap();
        let config = load_config(&args).unwrap();
        assert!(config.logging.include_timestamps);
    }

    #[test]
    fn test_load_config_all_sections() {
        let args = Args::try_parse_from([
            "gannet-mcp",
            "--host",
            "0.0.0.0",
            "--port",
            "9090",
            "--provider",
            "serper",
            "--api-key",
            "serper-key",
            "--search-limit",
            "15",
            "--requests-per-minute",
            "30",
            "--log-level",
            "warn",
        ])
        .unwrap();
        let config = load_config(&args).unwrap();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 9090);
        assert_eq!(config.search.provider, "serper");
        assert_eq!(config.search.api_key, Some("serper-key".to_string()));
        assert_eq!(config.search.default_limit, 15);
        assert_eq!(config.rate_limit.requests_per_minute, 30);
        assert_eq!(config.logging.level, "warn");
    }

    #[test]
    fn test_load_config_allowed_hosts() {
        let args = Args::try_parse_from(["gannet-mcp", "--allowed-hosts", "localhost,example.com"])
            .unwrap();
        let config = load_config(&args).unwrap();
        assert_eq!(
            config.server.http.allowed_hosts,
            vec!["localhost".to_string(), "example.com".to_string()]
        );
    }

    #[test]
    fn test_load_config_json_response() {
        let args = Args::try_parse_from(["gannet-mcp", "--json-response"]).unwrap();
        let config = load_config(&args).unwrap();
        assert!(config.server.http.json_response);
    }

    #[test]
    fn test_load_config_legacy_session_mode() {
        let args = Args::try_parse_from(["gannet-mcp", "--legacy-session-mode"]).unwrap();
        let config = load_config(&args).unwrap();
        assert!(config.server.http.legacy_session_mode);
    }

    #[test]
    fn test_load_config_rate_limited_flag() {
        let args = Args::try_parse_from(["gannet-mcp", "--rate-limited"]).unwrap();
        let config = load_config(&args).unwrap();
        assert!(config.rate_limit.enabled);
    }

    #[test]
    fn test_service_status_parsing() {
        let args = Args::try_parse_from(["gannet-mcp", "service", "status"]).unwrap();
        assert!(matches!(
            args.command,
            Some(Command::Service {
                action: service::ServiceAction::Status
            })
        ));
    }

    #[test]
    fn test_service_run_parsing_hidden() {
        // `run` is hidden from help but must still parse.
        let args = Args::try_parse_from(["gannet-mcp", "service", "run"]).unwrap();
        assert!(matches!(
            args.command,
            Some(Command::Service {
                action: service::ServiceAction::Run
            })
        ));
    }

    #[test]
    fn test_service_run_forces_http() {
        // Unit-level: the `service run` path forces use_stdio=false.
        let args = Args::try_parse_from(["gannet-mcp", "service", "run"]).unwrap();
        let mut config = load_config(&args).unwrap();
        service::apply_run_overrides(&mut config);
        assert!(!config.server.use_stdio);
    }

    #[test]
    fn test_service_run_forces_http_over_cli_stdio() {
        // Even an explicit `--stdio` must not survive the `service run` override.
        let args = Args::try_parse_from(["gannet-mcp", "--stdio", "service", "run"]).unwrap();
        assert!(args.stdio);
        let mut config = load_config(&args).unwrap();
        assert!(config.server.use_stdio);
        service::apply_run_overrides(&mut config);
        assert!(!config.server.use_stdio);
    }

    #[test]
    fn test_mcp_config_parsing() {
        let args =
            Args::try_parse_from(["gannet-mcp", "mcp-config", "--client", "claude", "--http"])
                .unwrap();
        match args.command {
            Some(Command::McpConfig(mc)) => {
                assert!(matches!(mc.client, Some(mcp_config::McpClient::Claude)));
                assert!(mc.http);
                assert!(!mc.stdio);
            }
            other => panic!("expected mcp-config subcommand, got {other:?}"),
        }
    }

    #[test]
    fn test_global_flags_after_subcommand() {
        // `--config` / `--log-*` are global: they parse after subcommands too.
        let args = Args::try_parse_from([
            "gannet-mcp",
            "service",
            "status",
            "--log-level",
            "debug",
            "--log-format",
            "json",
        ])
        .unwrap();
        assert_eq!(args.log_level, Level::DEBUG);
        assert_eq!(args.log_format, LogFormat::Json);
        assert!(matches!(
            args.command,
            Some(Command::Service {
                action: service::ServiceAction::Status
            })
        ));
    }
}
