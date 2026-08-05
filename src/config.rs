// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Ole Reinartz
//!
//! Configuration models for the server

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Server configuration
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct Config {
    /// Server settings
    #[serde(default)]
    pub server: ServerConfig,

    /// Search API settings
    #[serde(default)]
    pub search: SearchConfig,

    /// Rate limiting settings
    #[serde(default)]
    pub rate_limit: RateLimitConfig,

    /// Logging settings
    #[serde(default)]
    pub logging: LoggingConfig,
}

/// HTTP transport configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpConfig {
    /// URL path for MCP endpoint (default: /mcp)
    #[serde(default = "default_mcp_path")]
    pub path: String,
    /// SSE keep-alive interval in seconds (default: 30)
    #[serde(default = "default_sse_keep_alive")]
    pub sse_keep_alive_secs: Option<u64>,
    /// Prefer JSON responses for simple request-response (default: true)
    #[serde(default = "default_json_response")]
    pub json_response: bool,
    /// Allowed hosts for inbound Host header validation
    #[serde(default)]
    pub allowed_hosts: Vec<String>,
    /// CORS allowed origin (default: "*" = allow all). Set to specific origin like "https://example.com" for production.
    #[serde(default = "default_cors_origin")]
    pub cors_origin: String,
    /// Legacy session mode. When false, all POSTs are handled statelessly without requiring Mcp-Session-Id headers.
    /// This fixes compatibility with clients that do not track and replay the Mcp-Session-Id response header
    /// (e.g. llama-server / llama-ui-mcp), which otherwise get 422 "Unexpected message, expect initialize request"
    /// on their notifications/initialized POST. (default: false)
    #[serde(default = "default_legacy_session_mode")]
    pub legacy_session_mode: bool,
}

fn default_cors_origin() -> String {
    "*".to_string()
}

fn default_mcp_path() -> String {
    "/mcp".into()
}

fn default_sse_keep_alive() -> Option<u64> {
    Some(30)
}

fn default_json_response() -> bool {
    true
}

fn default_legacy_session_mode() -> bool {
    false
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            path: default_mcp_path(),
            sse_keep_alive_secs: default_sse_keep_alive(),
            json_response: default_json_response(),
            allowed_hosts: vec![],
            cors_origin: default_cors_origin(),
            legacy_session_mode: default_legacy_session_mode(),
        }
    }
}

/// Server configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    /// Host to bind to
    #[serde(default = "default_host")]
    pub host: String,

    /// Port to listen on
    #[serde(default = "default_port")]
    pub port: u16,

    /// Path to the socket file (for STDIO mode)
    #[serde(default)]
    pub socket_path: Option<PathBuf>,

    /// Use STDIO transport (default for MCP)
    #[serde(default = "default_stdio")]
    pub use_stdio: bool,

    /// HTTP transport configuration
    #[serde(default)]
    pub http: HttpConfig,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_stdio() -> bool {
    true
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            socket_path: None,
            use_stdio: true,
            http: HttpConfig::default(),
        }
    }
}

/// Search API configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchConfig {
    /// Search API provider (duckduckgo, serper, searxng, brightdata)
    #[serde(default = "default_search_provider")]
    pub provider: String,

    /// API key for the search provider
    #[serde(default)]
    pub api_key: Option<String>,

    /// Search engine ID (for Bright Data SERP zone, e.g. "serp_api1")
    #[serde(default)]
    pub search_engine_id: Option<String>,

    /// Base URL for custom search API (e.g., custom Bright Data endpoint or SearXNG instance)
    #[serde(default)]
    pub base_url: Option<String>,

    /// Default number of results
    #[serde(default = "default_search_limit")]
    pub default_limit: u32,

    /// Maximum results per query
    #[serde(default = "default_max_limit")]
    pub max_limit: u32,

    /// Request timeout in seconds
    #[serde(default = "default_search_timeout")]
    pub timeout_secs: u64,
}

fn default_search_provider() -> String {
    "duckduckgo".to_string()
}

fn default_search_limit() -> u32 {
    10
}

fn default_max_limit() -> u32 {
    100
}

fn default_search_timeout() -> u64 {
    10
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            provider: default_search_provider(),
            api_key: None,
            search_engine_id: None,
            base_url: None,
            default_limit: default_search_limit(),
            max_limit: default_max_limit(),
            timeout_secs: default_search_timeout(),
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Enable rate limiting
    #[serde(default = "default_rate_limit_enabled")]
    pub enabled: bool,

    /// Maximum requests per minute
    #[serde(default = "default_requests_per_minute")]
    pub requests_per_minute: u32,

    /// Maximum concurrent requests
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent: u32,

    /// Burst size for token bucket
    #[serde(default = "default_burst_size")]
    pub burst_size: u32,
}

fn default_rate_limit_enabled() -> bool {
    true
}

fn default_requests_per_minute() -> u32 {
    60
}

fn default_max_concurrent() -> u32 {
    10
}

fn default_burst_size() -> u32 {
    5
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: default_rate_limit_enabled(),
            requests_per_minute: default_requests_per_minute(),
            max_concurrent: default_max_concurrent(),
            burst_size: default_burst_size(),
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Log format (json, pretty, compact)
    #[serde(default = "default_log_format")]
    pub format: String,

    /// Include timestamps in logs
    #[serde(default = "default_include_timestamps")]
    pub include_timestamps: bool,
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "pretty".to_string()
}

fn default_include_timestamps() -> bool {
    true
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
            include_timestamps: default_include_timestamps(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::io::Write;
    use std::sync::LazyLock;
    static ENV_TEST_MUTEX: LazyLock<std::sync::Mutex<()>> =
        LazyLock::new(|| std::sync::Mutex::new(()));

    // --- default_* helper function tests ---

    #[test]
    fn test_default_host() {
        assert_eq!(default_host(), "127.0.0.1");
    }

    #[test]
    fn test_default_port() {
        assert_eq!(default_port(), 8080);
    }

    #[test]
    fn test_default_stdio() {
        assert!(default_stdio());
    }

    #[test]
    fn test_default_search_provider() {
        assert_eq!(default_search_provider(), "duckduckgo");
    }

    #[test]
    fn test_default_search_limit() {
        assert_eq!(default_search_limit(), 10);
    }

    #[test]
    fn test_default_max_limit() {
        assert_eq!(default_max_limit(), 100);
    }

    #[test]
    fn test_default_search_timeout() {
        assert_eq!(default_search_timeout(), 10);
    }

    #[test]
    fn test_default_rate_limit_enabled() {
        assert!(default_rate_limit_enabled());
    }

    #[test]
    fn test_default_requests_per_minute() {
        assert_eq!(default_requests_per_minute(), 60);
    }

    #[test]
    fn test_default_max_concurrent() {
        assert_eq!(default_max_concurrent(), 10);
    }

    #[test]
    fn test_default_burst_size() {
        assert_eq!(default_burst_size(), 5);
    }

    #[test]
    fn test_default_log_level() {
        assert_eq!(default_log_level(), "info");
    }

    #[test]
    fn test_default_log_format() {
        assert_eq!(default_log_format(), "pretty");
    }

    #[test]
    fn test_default_include_timestamps() {
        assert!(default_include_timestamps());
    }

    // --- HttpConfig default helper tests ---

    #[test]
    fn test_default_mcp_path() {
        assert_eq!(default_mcp_path(), "/mcp");
    }

    #[test]
    fn test_default_sse_keep_alive() {
        assert_eq!(default_sse_keep_alive(), Some(30));
    }

    #[test]
    fn test_default_json_response() {
        assert!(default_json_response());
    }

    // --- HttpConfig tests ---

    #[test]
    fn test_http_config_default() {
        let cfg = HttpConfig::default();
        assert_eq!(cfg.path, "/mcp");
        assert_eq!(cfg.sse_keep_alive_secs, Some(30));
        assert!(cfg.json_response);
        assert!(cfg.allowed_hosts.is_empty());
        assert!(!cfg.legacy_session_mode);
    }

    #[test]
    fn test_http_config_clone() {
        let cfg = HttpConfig::default();
        let cloned = cfg.clone();
        assert_eq!(cfg.path, cloned.path);
        assert_eq!(cfg.sse_keep_alive_secs, cloned.sse_keep_alive_secs);
        assert_eq!(cfg.json_response, cloned.json_response);
        assert_eq!(cfg.allowed_hosts, cloned.allowed_hosts);
        assert_eq!(cfg.legacy_session_mode, cloned.legacy_session_mode);
    }

    #[test]
    fn test_http_config_custom() {
        let cfg = HttpConfig {
            path: "/custom-mcp".to_string(),
            sse_keep_alive_secs: Some(60),
            json_response: false,
            allowed_hosts: vec!["example.com".to_string(), "localhost".to_string()],
            cors_origin: "*".to_string(),
            legacy_session_mode: true,
        };
        assert_eq!(cfg.path, "/custom-mcp");
        assert_eq!(cfg.sse_keep_alive_secs, Some(60));
        assert!(!cfg.json_response);
        assert_eq!(cfg.allowed_hosts, vec!["example.com", "localhost"]);
    }

    #[test]
    fn test_http_config_deserialize_json() {
        let json = r#"{"path":"/api/mcp","sse_keep_alive_secs":45,"json_response":false,"allowed_hosts":["example.com"]}"#;
        let cfg: HttpConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.path, "/api/mcp");
        assert_eq!(cfg.sse_keep_alive_secs, Some(45));
        assert!(!cfg.json_response);
        assert_eq!(cfg.allowed_hosts, vec!["example.com"]);
    }

    #[test]
    fn test_http_config_deserialize_yaml() {
        let yaml = r#"
path: /v1/mcp
sse_keep_alive_secs: 120
json_response: false
allowed_hosts:
  - localhost
  - example.org
"#;
        let cfg: HttpConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(cfg.path, "/v1/mcp");
        assert_eq!(cfg.sse_keep_alive_secs, Some(120));
        assert!(!cfg.json_response);
        assert_eq!(cfg.allowed_hosts, vec!["localhost", "example.org"]);
    }

    #[test]
    fn test_http_config_deserialize_empty_json() {
        let json = r#"{}"#;
        let cfg: HttpConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.path, "/mcp");
        assert_eq!(cfg.sse_keep_alive_secs, Some(30));
        assert!(cfg.json_response);
        assert!(cfg.allowed_hosts.is_empty());
    }

    #[test]
    fn test_http_config_serialize_roundtrip() {
        let cfg = HttpConfig {
            path: "/custom-mcp".to_string(),
            sse_keep_alive_secs: Some(60),
            json_response: false,
            allowed_hosts: vec!["example.com".to_string()],
            cors_origin: "*".to_string(),
            legacy_session_mode: true,
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let deserialized: HttpConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg.path, deserialized.path);
        assert_eq!(cfg.sse_keep_alive_secs, deserialized.sse_keep_alive_secs);
        assert_eq!(cfg.json_response, deserialized.json_response);
        assert_eq!(cfg.allowed_hosts, deserialized.allowed_hosts);
    }

    // --- ServerConfig tests ---

    #[test]
    fn test_server_config_default() {
        let cfg = ServerConfig::default();
        assert_eq!(cfg.host, "127.0.0.1");
        assert_eq!(cfg.port, 8080);
        assert!(cfg.socket_path.is_none());
        assert!(cfg.use_stdio);
        assert_eq!(cfg.http.path, "/mcp");
        assert_eq!(cfg.http.sse_keep_alive_secs, Some(30));
        assert!(cfg.http.json_response);
        assert!(cfg.http.allowed_hosts.is_empty());
    }

    #[test]
    fn test_server_config_clone() {
        let cfg = ServerConfig::default();
        let cloned = cfg.clone();
        assert_eq!(cfg.host, cloned.host);
        assert_eq!(cfg.port, cloned.port);
        assert_eq!(cfg.socket_path, cloned.socket_path);
        assert_eq!(cfg.use_stdio, cloned.use_stdio);
        assert_eq!(cfg.http.path, cloned.http.path);
        assert_eq!(
            cfg.http.sse_keep_alive_secs,
            cloned.http.sse_keep_alive_secs
        );
        assert_eq!(cfg.http.json_response, cloned.http.json_response);
        assert_eq!(cfg.http.allowed_hosts, cloned.http.allowed_hosts);
    }

    #[test]
    fn test_server_config_custom() {
        let cfg = ServerConfig {
            host: "0.0.0.0".to_string(),
            port: 3000,
            socket_path: Some(PathBuf::from("/tmp/mcp.sock")),
            use_stdio: false,
            http: HttpConfig::default(),
        };
        assert_eq!(cfg.host, "0.0.0.0");
        assert_eq!(cfg.port, 3000);
        assert_eq!(cfg.socket_path, Some(PathBuf::from("/tmp/mcp.sock")));
        assert!(!cfg.use_stdio);
    }

    #[test]
    fn test_server_config_deserialize_custom_json() {
        let json = r#"{"host":"0.0.0.0","port":3000,"use_stdio":false}"#;
        let cfg: ServerConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.host, "0.0.0.0");
        assert_eq!(cfg.port, 3000);
        assert!(!cfg.use_stdio);
        assert_eq!(cfg.http.path, "/mcp");
    }

    #[test]
    fn test_server_config_deserialize_empty_json() {
        let json = r#"{}"#;
        let cfg: ServerConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.host, "127.0.0.1");
        assert_eq!(cfg.port, 8080);
        assert!(cfg.socket_path.is_none());
        assert!(cfg.use_stdio);
        assert_eq!(cfg.http.path, "/mcp");
    }

    #[test]
    fn test_server_config_deserialize_yaml() {
        let yaml = r#"
host: 0.0.0.0
port: 9090
use_stdio: false
"#;
        let cfg: ServerConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(cfg.host, "0.0.0.0");
        assert_eq!(cfg.port, 9090);
        assert!(!cfg.use_stdio);
        assert_eq!(cfg.http.path, "/mcp");
    }

    #[test]
    fn test_server_config_deserialize_http_inline_json() {
        let json =
            r#"{"host":"0.0.0.0","port":3000,"http":{"path":"/custom","json_response":false}}"#;
        let cfg: ServerConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.host, "0.0.0.0");
        assert_eq!(cfg.port, 3000);
        assert_eq!(cfg.http.path, "/custom");
        assert!(!cfg.http.json_response);
    }

    #[test]
    fn test_server_config_serialize_roundtrip() {
        let cfg = ServerConfig {
            host: "0.0.0.0".to_string(),
            port: 3000,
            socket_path: Some(PathBuf::from("/tmp/mcp.sock")),
            use_stdio: false,
            http: HttpConfig::default(),
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let deserialized: ServerConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg.host, deserialized.host);
        assert_eq!(cfg.port, deserialized.port);
        assert_eq!(cfg.socket_path, deserialized.socket_path);
        assert_eq!(cfg.use_stdio, deserialized.use_stdio);
        assert_eq!(cfg.http.path, deserialized.http.path);
        assert_eq!(
            cfg.http.sse_keep_alive_secs,
            deserialized.http.sse_keep_alive_secs
        );
        assert_eq!(cfg.http.json_response, deserialized.http.json_response);
        assert_eq!(cfg.http.allowed_hosts, deserialized.http.allowed_hosts);
    }

    // --- SearchConfig tests ---

    #[test]
    fn test_search_config_default() {
        let cfg = SearchConfig::default();
        assert_eq!(cfg.provider, "duckduckgo");
        assert!(cfg.api_key.is_none());
        assert!(cfg.search_engine_id.is_none());
        assert!(cfg.base_url.is_none());
        assert_eq!(cfg.default_limit, 10);
        assert_eq!(cfg.max_limit, 100);
        assert_eq!(cfg.timeout_secs, 10);
    }

    #[test]
    fn test_search_config_custom() {
        let cfg = SearchConfig {
            provider: "brightdata".to_string(),
            api_key: Some("test-key".to_string()),
            search_engine_id: Some("engine-id".to_string()),
            base_url: Some("http://custom-api.local".to_string()),
            default_limit: 20,
            max_limit: 50,
            timeout_secs: 30,
        };
        assert_eq!(cfg.provider, "brightdata");
        assert_eq!(cfg.api_key, Some("test-key".to_string()));
        assert_eq!(cfg.search_engine_id, Some("engine-id".to_string()));
        assert_eq!(cfg.base_url, Some("http://custom-api.local".to_string()));
        assert_eq!(cfg.default_limit, 20);
        assert_eq!(cfg.max_limit, 50);
        assert_eq!(cfg.timeout_secs, 30);
    }

    #[test]
    fn test_search_config_deserialize_json() {
        let json =
            r#"{"provider":"serper","api_key":"api-123","default_limit":5,"timeout_secs":15}"#;
        let cfg: SearchConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.provider, "serper");
        assert_eq!(cfg.api_key, Some("api-123".to_string()));
        assert_eq!(cfg.default_limit, 5);
        assert_eq!(cfg.timeout_secs, 15);
        assert_eq!(cfg.max_limit, 100); // default
    }

    #[test]
    fn test_search_config_deserialize_yaml() {
        let yaml = r#"
provider: brightdata
api_key: brightdata-key-456
search_engine_id: serp_api1
default_limit: 25
max_limit: 200
timeout_secs: 20
"#;
        let cfg: SearchConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(cfg.provider, "brightdata");
        assert_eq!(cfg.api_key, Some("brightdata-key-456".to_string()));
        assert_eq!(cfg.search_engine_id, Some("serp_api1".to_string()));
        assert_eq!(cfg.default_limit, 25);
        assert_eq!(cfg.max_limit, 200);
        assert_eq!(cfg.timeout_secs, 20);
    }

    #[test]
    fn test_search_config_serialization_roundtrip() {
        let cfg = SearchConfig {
            provider: "brightdata".to_string(),
            api_key: Some("key".to_string()),
            search_engine_id: Some("eid".to_string()),
            base_url: None,
            default_limit: 5,
            max_limit: 50,
            timeout_secs: 5,
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let deserialized: SearchConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg.provider, deserialized.provider);
        assert_eq!(cfg.api_key, deserialized.api_key);
        assert_eq!(cfg.search_engine_id, deserialized.search_engine_id);
        assert_eq!(cfg.default_limit, deserialized.default_limit);
        assert_eq!(cfg.max_limit, deserialized.max_limit);
        assert_eq!(cfg.timeout_secs, deserialized.timeout_secs);
    }

    // --- RateLimitConfig tests ---

    #[test]
    fn test_rate_limit_config_default() {
        let cfg = RateLimitConfig::default();
        assert!(cfg.enabled);
        assert_eq!(cfg.requests_per_minute, 60);
        assert_eq!(cfg.max_concurrent, 10);
        assert_eq!(cfg.burst_size, 5);
    }

    #[test]
    fn test_rate_limit_config_custom() {
        let cfg = RateLimitConfig {
            enabled: false,
            requests_per_minute: 10,
            max_concurrent: 2,
            burst_size: 3,
        };
        assert!(!cfg.enabled);
        assert_eq!(cfg.requests_per_minute, 10);
        assert_eq!(cfg.max_concurrent, 2);
        assert_eq!(cfg.burst_size, 3);
    }

    #[test]
    fn test_rate_limit_config_deserialize_json() {
        let json = r#"{"enabled":false,"requests_per_minute":5,"max_concurrent":1}"#;
        let cfg: RateLimitConfig = serde_json::from_str(json).unwrap();
        assert!(!cfg.enabled);
        assert_eq!(cfg.requests_per_minute, 5);
        assert_eq!(cfg.max_concurrent, 1);
        assert_eq!(cfg.burst_size, 5); // default
    }

    #[test]
    fn test_rate_limit_config_deserialize_yaml() {
        let yaml = r#"
enabled: false
requests_per_minute: 100
max_concurrent: 20
burst_size: 10
"#;
        let cfg: RateLimitConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(!cfg.enabled);
        assert_eq!(cfg.requests_per_minute, 100);
        assert_eq!(cfg.max_concurrent, 20);
        assert_eq!(cfg.burst_size, 10);
    }

    #[test]
    fn test_rate_limit_config_serialization_roundtrip() {
        let cfg = RateLimitConfig {
            enabled: false,
            requests_per_minute: 30,
            max_concurrent: 5,
            burst_size: 10,
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let deserialized: RateLimitConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg.enabled, deserialized.enabled);
        assert_eq!(cfg.requests_per_minute, deserialized.requests_per_minute);
        assert_eq!(cfg.max_concurrent, deserialized.max_concurrent);
        assert_eq!(cfg.burst_size, deserialized.burst_size);
    }

    // --- LoggingConfig tests ---

    #[test]
    fn test_logging_config_default() {
        let cfg = LoggingConfig::default();
        assert_eq!(cfg.level, "info");
        assert_eq!(cfg.format, "pretty");
        assert!(cfg.include_timestamps);
    }

    #[test]
    fn test_logging_config_custom() {
        let cfg = LoggingConfig {
            level: "debug".to_string(),
            format: "json".to_string(),
            include_timestamps: false,
        };
        assert_eq!(cfg.level, "debug");
        assert_eq!(cfg.format, "json");
        assert!(!cfg.include_timestamps);
    }

    #[test]
    fn test_logging_config_deserialize_json() {
        let json = r#"{"level":"trace","format":"compact","include_timestamps":false}"#;
        let cfg: LoggingConfig = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.level, "trace");
        assert_eq!(cfg.format, "compact");
        assert!(!cfg.include_timestamps);
    }

    #[test]
    fn test_logging_config_deserialize_yaml() {
        let yaml = r#"
level: warn
format: json
include_timestamps: false
"#;
        let cfg: LoggingConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(cfg.level, "warn");
        assert_eq!(cfg.format, "json");
        assert!(!cfg.include_timestamps);
    }

    #[test]
    fn test_logging_config_serialization_roundtrip() {
        let cfg = LoggingConfig {
            level: "debug".to_string(),
            format: "json".to_string(),
            include_timestamps: false,
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let deserialized: LoggingConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg.level, deserialized.level);
        assert_eq!(cfg.format, deserialized.format);
        assert_eq!(cfg.include_timestamps, deserialized.include_timestamps);
    }

    // --- Config tests ---

    #[test]
    fn test_config_default() {
        let cfg = Config::default();
        assert_eq!(cfg.server.host, "127.0.0.1");
        assert_eq!(cfg.server.port, 8080);
        assert!(cfg.server.use_stdio);
        assert_eq!(cfg.server.http.path, "/mcp");
        assert_eq!(cfg.server.http.sse_keep_alive_secs, Some(30));
        assert!(cfg.server.http.json_response);
        assert_eq!(cfg.search.provider, "duckduckgo");
        assert_eq!(cfg.search.default_limit, 10);
        assert_eq!(cfg.search.max_limit, 100);
        assert!(cfg.rate_limit.enabled);
        assert_eq!(cfg.rate_limit.requests_per_minute, 60);
        assert_eq!(cfg.logging.level, "info");
        assert_eq!(cfg.logging.format, "pretty");
        assert!(cfg.logging.include_timestamps);
    }

    #[test]
    fn test_config_deserialize_full_json() {
        let json = r#"
{
  "server": { "host": "0.0.0.0", "port": 9090, "use_stdio": false, "http": { "path": "/api", "json_response": false } },
  "search": { "provider": "brightdata", "api_key": "g-key", "default_limit": 5, "timeout_secs": 30 },
  "rate_limit": { "enabled": false, "requests_per_minute": 20 },
  "logging": { "level": "debug", "format": "json", "include_timestamps": false }
}"#;
        let cfg: Config = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.server.host, "0.0.0.0");
        assert_eq!(cfg.server.port, 9090);
        assert!(!cfg.server.use_stdio);
        assert_eq!(cfg.server.http.path, "/api");
        assert!(!cfg.server.http.json_response);
        assert_eq!(cfg.search.provider, "brightdata");
        assert_eq!(cfg.search.api_key, Some("g-key".to_string()));
        assert_eq!(cfg.search.default_limit, 5);
        assert_eq!(cfg.search.timeout_secs, 30);
        assert!(!cfg.rate_limit.enabled);
        assert_eq!(cfg.rate_limit.requests_per_minute, 20);
        assert_eq!(cfg.logging.level, "debug");
        assert_eq!(cfg.logging.format, "json");
        assert!(!cfg.logging.include_timestamps);
    }

    #[test]
    fn test_config_deserialize_full_yaml() {
        let yaml = r#"
server:
  host: 0.0.0.0
  port: 9090
  use_stdio: false
  http:
    path: /v1/mcp
    json_response: false
search:
  provider: serper
  api_key: serper-key
  default_limit: 15
rate_limit:
  enabled: false
logging:
  level: warn
  format: compact
  include_timestamps: false
"#;
        let cfg: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(cfg.server.host, "0.0.0.0");
        assert_eq!(cfg.server.http.path, "/v1/mcp");
        assert!(!cfg.server.http.json_response);
        assert_eq!(cfg.search.provider, "serper");
        assert_eq!(cfg.search.api_key, Some("serper-key".to_string()));
        assert!(!cfg.rate_limit.enabled);
        assert_eq!(cfg.logging.level, "warn");
    }

    #[test]
    fn test_config_deserialize_empty_json() {
        let json = r#"{}"#;
        let cfg: Config = serde_json::from_str(json).unwrap();
        assert_eq!(cfg.server.host, "127.0.0.1");
        assert_eq!(cfg.server.http.path, "/mcp");
        assert_eq!(cfg.search.provider, "duckduckgo");
        assert!(cfg.rate_limit.enabled);
        assert_eq!(cfg.logging.level, "info");
    }

    #[test]
    fn test_config_serialize_roundtrip() {
        let cfg = Config {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 3000,
                use_stdio: false,
                ..Default::default()
            },
            search: SearchConfig {
                provider: "brightdata".to_string(),
                api_key: Some("brightdata-key".to_string()),
                search_engine_id: Some("serp_api1".to_string()),
                default_limit: 25,
                ..Default::default()
            },
            rate_limit: RateLimitConfig {
                enabled: false,
                requests_per_minute: 30,
                ..Default::default()
            },
            logging: LoggingConfig {
                level: "trace".to_string(),
                format: "json".to_string(),
                include_timestamps: false,
            },
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(cfg.server.host, deserialized.server.host);
        assert_eq!(cfg.server.http.path, deserialized.server.http.path);
        assert_eq!(cfg.search.provider, deserialized.search.provider);
        assert_eq!(cfg.rate_limit.enabled, deserialized.rate_limit.enabled);
        assert_eq!(cfg.logging.level, deserialized.logging.level);
    }

    // --- Config::from_file() tests ---

    #[test]
    fn test_config_from_file_yaml() {
        let _guard = ENV_TEST_MUTEX.lock().unwrap();
        let yaml_content = r#"
server:
  host: 0.0.0.0
  port: 9090
search:
  provider: brightdata
  api_key: yaml-key
  default_limit: 7
rate_limit:
  enabled: false
logging:
  level: debug
"#;
        let mut tmp = tempfile::Builder::new().suffix(".yaml").tempfile().unwrap();
        tmp.write_all(yaml_content.as_bytes()).unwrap();
        let content = std::fs::read_to_string(tmp.path()).unwrap();
        let cfg: Config = serde_yaml::from_str(&content).unwrap();
        assert_eq!(cfg.server.host, "0.0.0.0");
        assert_eq!(cfg.server.port, 9090);
        assert_eq!(cfg.search.provider, "brightdata");
        assert_eq!(cfg.search.api_key, Some("yaml-key".to_string()));
        assert_eq!(cfg.search.default_limit, 7);
        assert!(!cfg.rate_limit.enabled);
        assert_eq!(cfg.logging.level, "debug");
        assert_eq!(cfg.server.http.path, "/mcp");
    }

    #[test]
    fn test_config_from_file_json() {
        let json_content = r#"
{
  "server": { "host": "10.0.0.1", "port": 4000 },
  "search": { "provider": "brightdata", "api_key": "json-key", "search_engine_id": "serp_api1", "max_limit": 50 },
  "rate_limit": { "requests_per_minute": 15 },
  "logging": { "format": "compact" }
}"#;
        let cfg: Config = serde_json::from_str(json_content).unwrap();
        assert_eq!(cfg.server.host, "10.0.0.1");
        assert_eq!(cfg.server.port, 4000);
        assert_eq!(cfg.search.provider, "brightdata");
        assert_eq!(cfg.search.api_key, Some("json-key".to_string()));
        assert_eq!(cfg.search.max_limit, 50);
        assert_eq!(cfg.rate_limit.requests_per_minute, 15);
        assert_eq!(cfg.logging.format, "compact");
        assert_eq!(cfg.server.http.path, "/mcp");
    }

    #[test]
    fn test_config_from_file_nonexistent() {
        let result = Config::from_file(&PathBuf::from("/nonexistent/path/config.yaml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_config_from_file_invalid_content() {
        let _guard = ENV_TEST_MUTEX.lock().unwrap();
        let mut tmp = tempfile::Builder::new().suffix(".yaml").tempfile().unwrap();
        tmp.write_all(b"not valid yaml or json {{{").unwrap();
        let path = tmp.as_ref().to_path_buf();
        let result = Config::from_file(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_from_file_empty() {
        let _guard = ENV_TEST_MUTEX.lock().unwrap();
        let mut tmp = tempfile::Builder::new().suffix(".json").tempfile().unwrap();
        tmp.write_all(b"{}").unwrap();
        let path = tmp.as_ref().to_path_buf();
        let cfg = Config::from_file(&path).unwrap();
        assert_eq!(cfg.server.host, "127.0.0.1");
        assert_eq!(cfg.search.provider, "duckduckgo");
    }

    // --- Config::from_env() tests ---

    #[test]
    fn test_config_from_env_all_vars() {
        let _guard = ENV_TEST_MUTEX.lock().unwrap();
        env::set_var("MCP__SERVER__HOST", "0.0.0.0");
        env::set_var("MCP__SERVER__PORT", "7777");
        env::set_var("MCP__SERVER__HTTP__PATH", "/custom-mcp");
        env::set_var("MCP__SEARCH__PROVIDER", "serper");
        env::set_var("MCP__SEARCH__API_KEY", "env-key");
        env::set_var("MCP__SEARCH__DEFAULT_LIMIT", "3");
        env::set_var("MCP__RATE_LIMIT__ENABLED", "false");
        env::set_var("MCP__RATE_LIMIT__REQUESTS_PER_MINUTE", "25");
        env::set_var("MCP__LOGGING__LEVEL", "trace");

        let cfg = Config::from_env().unwrap();
        assert_eq!(cfg.server.host, "0.0.0.0");
        assert_eq!(cfg.server.port, 7777);
        assert_eq!(cfg.server.http.path, "/custom-mcp");
        assert_eq!(cfg.search.provider, "serper");
        assert_eq!(cfg.search.api_key, Some("env-key".to_string()));
        assert_eq!(cfg.search.default_limit, 3);
        assert!(!cfg.rate_limit.enabled);
        assert_eq!(cfg.rate_limit.requests_per_minute, 25);
        assert_eq!(cfg.logging.level, "trace");

        // Cleanup
        env::remove_var("MCP__SERVER__HOST");
        env::remove_var("MCP__SERVER__PORT");
        env::remove_var("MCP__SERVER__HTTP__PATH");
        env::remove_var("MCP__SEARCH__PROVIDER");
        env::remove_var("MCP__SEARCH__API_KEY");
        env::remove_var("MCP__SEARCH__DEFAULT_LIMIT");
        env::remove_var("MCP__RATE_LIMIT__ENABLED");
        env::remove_var("MCP__RATE_LIMIT__REQUESTS_PER_MINUTE");
        env::remove_var("MCP__LOGGING__LEVEL");
    }

    #[test]
    fn test_config_from_env_partial_vars() {
        let _guard = ENV_TEST_MUTEX.lock().unwrap();
        for var in &[
            "MCP__SERVER__HOST",
            "MCP__SERVER__PORT",
            "MCP__SERVER__HTTP__PATH",
            "MCP__SEARCH__PROVIDER",
            "MCP__SEARCH__API_KEY",
            "MCP__SEARCH__DEFAULT_LIMIT",
            "MCP__RATE_LIMIT__ENABLED",
            "MCP__RATE_LIMIT__REQUESTS_PER_MINUTE",
            "MCP__LOGGING__LEVEL",
            "MCP__SEARCH__MAX_LIMIT",
            "MCP__RATE_LIMIT__BURST_SIZE",
            "MCP__SEARCH__TIMEOUT_SECS",
        ] {
            env::remove_var(var);
        }
        env::set_var("MCP__SEARCH__PROVIDER", "brightdata");
        env::set_var("MCP__SEARCH__MAX_LIMIT", "200");

        let cfg = Config::from_env().unwrap();
        assert_eq!(cfg.search.provider, "brightdata");
        assert_eq!(cfg.search.max_limit, 200);
        // Rest should use defaults
        assert_eq!(cfg.server.host, "127.0.0.1");
        assert_eq!(cfg.server.http.path, "/mcp");
        assert_eq!(cfg.logging.level, "info");

        // Cleanup
        env::remove_var("MCP__SEARCH__PROVIDER");
        env::remove_var("MCP__SEARCH__MAX_LIMIT");
    }

    #[test]
    fn test_config_from_env_no_vars() {
        let _guard = ENV_TEST_MUTEX.lock().unwrap();
        for var in &[
            "MCP__SERVER__HOST",
            "MCP__SERVER__PORT",
            "MCP__SERVER__HTTP__PATH",
            "MCP__SEARCH__PROVIDER",
            "MCP__SEARCH__API_KEY",
            "MCP__SEARCH__DEFAULT_LIMIT",
            "MCP__RATE_LIMIT__ENABLED",
            "MCP__RATE_LIMIT__REQUESTS_PER_MINUTE",
            "MCP__LOGGING__LEVEL",
            "MCP__SEARCH__MAX_LIMIT",
            "MCP__RATE_LIMIT__BURST_SIZE",
            "MCP__SEARCH__TIMEOUT_SECS",
        ] {
            env::remove_var(var);
        }

        let cfg = Config::from_env().unwrap();
        assert_eq!(cfg.server.host, "127.0.0.1");
        assert_eq!(cfg.server.port, 8080);
        assert_eq!(cfg.server.http.path, "/mcp");
        assert_eq!(cfg.search.provider, "duckduckgo");
        assert!(cfg.rate_limit.enabled);
        assert_eq!(cfg.logging.format, "pretty");
    }

    #[test]
    fn test_config_from_file_with_env_override() {
        let _guard = ENV_TEST_MUTEX.lock().unwrap();
        env::set_var("MCP__SEARCH__PROVIDER", "env-override");

        let yaml_content = r#"
search:
  provider: yaml-provider
  default_limit: 10
"#;
        let mut tmp = tempfile::Builder::new().suffix(".yaml").tempfile().unwrap();
        tmp.write_all(yaml_content.as_bytes()).unwrap();
        let path = tmp.as_ref().to_path_buf();
        let cfg = Config::from_file(&path).unwrap();
        // Environment variables override file config
        assert_eq!(cfg.search.provider, "env-override");
        assert_eq!(cfg.search.default_limit, 10);

        // Cleanup
        env::remove_var("MCP__SEARCH__PROVIDER");
    }

    #[test]
    fn test_config_from_env_string_parsing() {
        let _guard = ENV_TEST_MUTEX.lock().unwrap();
        env::set_var("MCP__RATE_LIMIT__BURST_SIZE", "42");
        env::set_var("MCP__SEARCH__TIMEOUT_SECS", "99");

        let cfg = Config::from_env().unwrap();
        assert_eq!(cfg.rate_limit.burst_size, 42);
        assert_eq!(cfg.search.timeout_secs, 99);

        // Cleanup
        env::remove_var("MCP__RATE_LIMIT__BURST_SIZE");
        env::remove_var("MCP__SEARCH__TIMEOUT_SECS");
    }
}

impl Config {
    /// Load configuration from a file
    pub fn from_file(path: &std::path::Path) -> Result<Self, config::ConfigError> {
        let settings = config::Config::builder()
            .add_source(config::File::from(path))
            .add_source(
                config::Environment::with_prefix("MCP")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?;

        settings.try_deserialize()
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let settings = config::Config::builder()
            .add_source(
                config::Environment::with_prefix("MCP")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?;

        settings.try_deserialize()
    }
}
