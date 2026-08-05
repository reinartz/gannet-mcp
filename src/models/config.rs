// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Ole Reinartz
//!
//! Configuration models for the web search MCP server
//!
//! This module re-exports configuration types from the main config module
//! and provides additional model-specific configuration structures.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// Re-export the main config types from the parent module
pub use crate::config::{Config, LoggingConfig, RateLimitConfig, SearchConfig, ServerConfig};

/// Configuration for search provider-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider name (duckduckgo, serper, searxng, brightdata)
    pub name: String,

    /// Whether this provider is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Provider-specific API endpoint
    #[serde(default)]
    pub endpoint: Option<String>,

    /// Custom headers to include in requests
    #[serde(default)]
    pub headers: Option<std::collections::HashMap<String, String>>,

    /// Rate limiting for this specific provider
    #[serde(default)]
    pub rate_limit: Option<ProviderRateLimit>,
}

fn default_enabled() -> bool {
    true
}

/// Rate limiting configuration for a specific provider
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ProviderRateLimit {
    /// Maximum requests per minute
    pub requests_per_minute: u32,

    /// Burst capacity
    pub burst: u32,

    /// Cooldown period in seconds after rate limit hit
    #[serde(default = "default_cooldown")]
    pub cooldown_secs: u64,
}

fn default_cooldown() -> u64 {
    60
}

/// Configuration for content extraction settings
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExtractionConfig {
    /// Enable link extraction
    #[serde(default = "default_extract_links")]
    pub extract_links: bool,

    /// Enable image extraction
    #[serde(default)]
    pub extract_images: bool,

    /// Enable metadata extraction
    #[serde(default = "default_extract_metadata")]
    pub extract_metadata: bool,

    /// Maximum number of links to extract (0 = unlimited)
    #[serde(default)]
    pub max_links: usize,

    /// Maximum number of images to extract (0 = unlimited)
    #[serde(default)]
    pub max_images: usize,

    /// CSS selectors for main content (used for content extraction priority)
    #[serde(default = "default_content_selectors")]
    pub content_selectors: Vec<String>,

    /// CSS selectors to exclude from content
    #[serde(default)]
    pub exclude_selectors: Vec<String>,
}

fn default_extract_links() -> bool {
    true
}

fn default_extract_metadata() -> bool {
    true
}

fn default_content_selectors() -> Vec<String> {
    vec![
        "article".to_string(),
        "main".to_string(),
        "[role='main']".to_string(),
        ".content".to_string(),
        "#content".to_string(),
        ".post-content".to_string(),
        ".article-content".to_string(),
        ".entry-content".to_string(),
        ".post".to_string(),
        ".page".to_string(),
    ]
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            extract_links: true,
            extract_images: false,
            extract_metadata: true,
            max_links: 0,
            max_images: 0,
            content_selectors: default_content_selectors(),
            exclude_selectors: vec![
                "script".to_string(),
                "style".to_string(),
                "nav".to_string(),
                "footer".to_string(),
                "header".to_string(),
                ".sidebar".to_string(),
                ".ad".to_string(),
                ".advertisement".to_string(),
                ".social-share".to_string(),
            ],
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CacheConfig {
    /// Enable caching
    #[serde(default)]
    pub enabled: bool,

    /// Cache directory path
    #[serde(default)]
    pub cache_dir: Option<PathBuf>,

    /// Maximum cache size in bytes
    #[serde(default = "default_cache_size")]
    pub max_size_bytes: usize,

    /// Cache TTL in seconds
    #[serde(default = "default_cache_ttl")]
    pub ttl_secs: u64,

    /// Enable cache for search results
    #[serde(default = "default_cache_search")]
    pub cache_search: bool,

    /// Enable cache for fetched pages
    #[serde(default)]
    pub cache_fetch: bool,
}

fn default_cache_size() -> usize {
    100 * 1024 * 1024 // 100MB
}

fn default_cache_ttl() -> u64 {
    3600 // 1 hour
}

fn default_cache_search() -> bool {
    true
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            cache_dir: None,
            max_size_bytes: default_cache_size(),
            ttl_secs: default_cache_ttl(),
            cache_search: true,
            cache_fetch: false,
        }
    }
}

/// Security configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecurityConfig {
    /// Allowed URL schemes
    #[serde(default = "default_allowed_schemes")]
    pub allowed_schemes: Vec<String>,

    /// Blocked domains
    #[serde(default)]
    pub blocked_domains: Vec<String>,

    /// Allowed domains (empty = all allowed)
    #[serde(default)]
    pub allowed_domains: Vec<String>,

    /// Maximum redirect depth
    #[serde(default = "default_max_redirects")]
    pub max_redirects: u32,

    /// Require HTTPS for external resources
    #[serde(default)]
    pub require_https: bool,

    /// Custom DNS servers for lookups
    #[serde(default)]
    pub dns_servers: Vec<String>,
}

fn default_allowed_schemes() -> Vec<String> {
    vec!["http".to_string(), "https".to_string()]
}

fn default_max_redirects() -> u32 {
    10
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            allowed_schemes: default_allowed_schemes(),
            blocked_domains: Vec::new(),
            allowed_domains: Vec::new(),
            max_redirects: default_max_redirects(),
            require_https: false,
            dns_servers: Vec::new(),
        }
    }
}

impl SecurityConfig {
    /// Check if a URL scheme is allowed
    pub fn is_scheme_allowed(&self, scheme: &str) -> bool {
        self.allowed_schemes
            .iter()
            .any(|s| s.eq_ignore_ascii_case(scheme))
    }

    /// Check if a domain is allowed
    pub fn is_domain_allowed(&self, domain: &str) -> bool {
        // If no allowed domains specified, check blocked list
        if self.allowed_domains.is_empty() {
            !self
                .blocked_domains
                .iter()
                .any(|d| domain == d || domain.ends_with(&format!(".{d}")))
        } else {
            self.allowed_domains
                .iter()
                .any(|d| domain == d || domain.ends_with(&format!(".{d}")))
        }
    }
}

/// Timeout configuration for various operations
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct TimeoutConfig {
    /// Search request timeout in seconds
    #[serde(default = "default_search_timeout")]
    pub search_secs: u64,

    /// Fetch request timeout in seconds
    #[serde(default = "default_fetch_timeout")]
    pub fetch_secs: u64,

    /// DNS lookup timeout in seconds
    #[serde(default = "default_dns_timeout")]
    pub dns_secs: u64,

    /// Connection timeout in seconds
    #[serde(default = "default_connect_timeout")]
    pub connect_secs: u64,
}

fn default_search_timeout() -> u64 {
    10
}

fn default_fetch_timeout() -> u64 {
    30
}

fn default_dns_timeout() -> u64 {
    5
}

fn default_connect_timeout() -> u64 {
    10
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            search_secs: default_search_timeout(),
            fetch_secs: default_fetch_timeout(),
            dns_secs: default_dns_timeout(),
            connect_secs: default_connect_timeout(),
        }
    }
}

/// Complete application configuration combining all settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Server configuration
    #[serde(default)]
    pub server: ServerConfig,

    /// Search configuration
    #[serde(default)]
    pub search: SearchConfig,

    /// Rate limiting configuration
    #[serde(default)]
    pub rate_limit: RateLimitConfig,

    /// Logging configuration
    #[serde(default)]
    pub logging: LoggingConfig,

    /// Content extraction configuration
    #[serde(default)]
    pub extraction: ExtractionConfig,

    /// Cache configuration
    #[serde(default)]
    pub cache: CacheConfig,

    /// Security configuration
    #[serde(default)]
    pub security: SecurityConfig,

    /// Timeout configuration
    #[serde(default)]
    pub timeouts: TimeoutConfig,
}

impl AppConfig {
    /// Load configuration from a YAML file
    pub fn from_file(path: &Path) -> Result<Self, config::ConfigError> {
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

    /// Create a new AppConfig with default values
    pub fn new() -> Self {
        Self {
            server: ServerConfig::default(),
            search: SearchConfig::default(),
            rate_limit: RateLimitConfig::default(),
            logging: LoggingConfig::default(),
            extraction: ExtractionConfig::default(),
            cache: CacheConfig::default(),
            security: SecurityConfig::default(),
            timeouts: TimeoutConfig::default(),
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate search config
        if self.search.timeout_secs == 0 {
            return Err("Search timeout must be greater than 0".to_string());
        }

        if self.search.default_limit == 0 {
            return Err("Default search limit must be greater than 0".to_string());
        }

        if self.search.default_limit > self.search.max_limit {
            return Err("Default limit cannot exceed max limit".to_string());
        }

        // Validate rate limit config
        if self.rate_limit.requests_per_minute == 0 {
            return Err("Requests per minute must be greater than 0".to_string());
        }

        // Validate cache config
        if self.cache.max_size_bytes == 0 {
            return Err("Cache max size must be greater than 0".to_string());
        }

        // Validate timeouts
        if self.timeouts.search_secs == 0 {
            return Err("Search timeout must be greater than 0".to_string());
        }

        if self.timeouts.fetch_secs == 0 {
            return Err("Fetch timeout must be greater than 0".to_string());
        }

        Ok(())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::new()
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

    #[test]
    fn test_security_config_scheme_allowed() {
        let config = SecurityConfig::default();
        assert!(config.is_scheme_allowed("http"));
        assert!(config.is_scheme_allowed("https"));
        assert!(!config.is_scheme_allowed("ftp"));
        assert!(!config.is_scheme_allowed("javascript"));
    }

    #[test]
    fn test_security_config_domain_allowed() {
        let mut config = SecurityConfig::default();

        // No restrictions - all allowed
        assert!(config.is_domain_allowed("example.com"));
        assert!(config.is_domain_allowed("google.com"));

        // Test blocked domains
        config.blocked_domains.push("evil.com".to_string());
        assert!(!config.is_domain_allowed("evil.com"));
        assert!(!config.is_domain_allowed("sub.evil.com"));
        assert!(config.is_domain_allowed("good.com"));

        // Test allowed domains only
        let mut config2 = SecurityConfig::default();
        config2.allowed_domains.push("trusted.com".to_string());
        assert!(config2.is_domain_allowed("trusted.com"));
        assert!(config2.is_domain_allowed("sub.trusted.com"));
        assert!(!config2.is_domain_allowed("other.com"));
    }

    #[test]
    fn test_extraction_config_default() {
        let config = ExtractionConfig::default();
        assert!(config.extract_links);
        assert!(!config.extract_images);
        assert!(config.extract_metadata);
        assert!(!config.content_selectors.is_empty());
    }

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.max_size_bytes, 100 * 1024 * 1024);
        assert_eq!(config.ttl_secs, 3600);
    }

    #[test]
    fn test_timeout_config_default() {
        let config = TimeoutConfig::default();
        assert_eq!(config.search_secs, 10);
        assert_eq!(config.fetch_secs, 30);
    }

    #[test]
    fn test_app_config_validate() {
        let config = AppConfig::new();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_app_config_validate_invalid_search_limit() {
        let mut config = AppConfig::new();
        config.search.default_limit = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_provider_rate_limit_default() {
        let limit = ProviderRateLimit {
            requests_per_minute: 60,
            burst: 10,
            cooldown_secs: 60,
        };
        assert_eq!(limit.cooldown_secs, 60);
    }

    // --- ProviderConfig default_* helper tests ---

    #[test]
    fn test_default_enabled() {
        assert!(default_enabled());
    }

    #[test]
    fn test_default_cooldown() {
        assert_eq!(default_cooldown(), 60);
    }

    #[test]
    fn test_default_extract_links_config() {
        assert!(default_extract_links());
    }

    #[test]
    fn test_default_extract_metadata() {
        assert!(default_extract_metadata());
    }

    #[test]
    fn test_default_content_selectors() {
        let selectors = default_content_selectors();
        assert!(!selectors.is_empty());
        assert_eq!(selectors[0], "article");
        assert_eq!(selectors[1], "main");
        assert_eq!(selectors[2], "[role='main']");
        assert!(selectors.len() >= 5);
    }

    #[test]
    fn test_default_cache_size() {
        assert_eq!(default_cache_size(), 100 * 1024 * 1024);
    }

    #[test]
    fn test_default_cache_ttl() {
        assert_eq!(default_cache_ttl(), 3600);
    }

    #[test]
    fn test_default_cache_search() {
        assert!(default_cache_search());
    }

    #[test]
    fn test_default_allowed_schemes() {
        let schemes = default_allowed_schemes();
        assert_eq!(schemes.len(), 2);
        assert_eq!(schemes[0], "http");
        assert_eq!(schemes[1], "https");
    }

    #[test]
    fn test_default_max_redirects() {
        assert_eq!(default_max_redirects(), 10);
    }

    #[test]
    fn test_default_search_timeout() {
        assert_eq!(default_search_timeout(), 10);
    }

    #[test]
    fn test_default_fetch_timeout() {
        assert_eq!(default_fetch_timeout(), 30);
    }

    #[test]
    fn test_default_dns_timeout() {
        assert_eq!(default_dns_timeout(), 5);
    }

    #[test]
    fn test_default_connect_timeout() {
        assert_eq!(default_connect_timeout(), 10);
    }

    // --- ProviderConfig tests ---

    #[test]
    fn test_provider_config_default() {
        let config = ProviderConfig {
            name: "duckduckgo".to_string(),
            enabled: true,
            endpoint: None,
            headers: None,
            rate_limit: None,
        };
        assert_eq!(config.name, "duckduckgo");
        assert!(config.enabled);
        assert!(config.endpoint.is_none());
        assert!(config.headers.is_none());
        assert!(config.rate_limit.is_none());
    }

    #[test]
    fn test_provider_config_custom() {
        let mut headers = std::collections::HashMap::new();
        headers.insert("X-Custom-Header".to_string(), "custom-value".to_string());
        let rate_limit = ProviderRateLimit {
            requests_per_minute: 30,
            burst: 5,
            cooldown_secs: 120,
        };
        let config = ProviderConfig {
            name: "brightdata".to_string(),
            enabled: false,
            endpoint: Some("https://custom.brightdata.api".to_string()),
            headers: Some(headers),
            rate_limit: Some(rate_limit),
        };
        assert_eq!(config.name, "brightdata");
        assert!(!config.enabled);
        assert_eq!(
            config.endpoint,
            Some("https://custom.brightdata.api".to_string())
        );
        assert!(config.headers.is_some());
        assert!(config.rate_limit.is_some());
        assert_eq!(config.rate_limit.as_ref().unwrap().requests_per_minute, 30);
    }

    #[test]
    fn test_provider_config_clone() {
        let config = ProviderConfig {
            name: "clone-provider".to_string(),
            enabled: true,
            endpoint: Some("https://clone.api".to_string()),
            headers: None,
            rate_limit: None,
        };
        let cloned = config.clone();
        assert_eq!(config.name, cloned.name);
        assert_eq!(config.enabled, cloned.enabled);
        assert_eq!(config.endpoint, cloned.endpoint);
    }

    #[test]
    fn test_provider_config_deserialize_json() {
        let json = r#"{
            "name": "serper",
            "enabled": true,
            "endpoint": "https://serper.googleapis.com",
            "headers": {
                "Authorization": "Bearer token"
            }
        }"#;
        let config: ProviderConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.name, "serper");
        assert!(config.enabled);
        assert_eq!(
            config.endpoint,
            Some("https://serper.googleapis.com".to_string())
        );
        assert!(config.headers.is_some());
        assert!(config.rate_limit.is_none());
    }

    #[test]
    fn test_provider_config_deserialize_json_minimal() {
        let json = r#"{"name": "brightdata"}"#;
        let config: ProviderConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.name, "brightdata");
        assert!(config.enabled);
        assert!(config.endpoint.is_none());
        assert!(config.headers.is_none());
        assert!(config.rate_limit.is_none());
    }

    #[test]
    fn test_provider_config_serde_roundtrip() {
        let headers = {
            let mut h = std::collections::HashMap::new();
            h.insert("X-Api-Key".to_string(), "secret".to_string());
            Some(h)
        };
        let rate_limit = ProviderRateLimit {
            requests_per_minute: 20,
            burst: 3,
            cooldown_secs: 30,
        };
        let config = ProviderConfig {
            name: "roundtrip".to_string(),
            enabled: true,
            endpoint: Some("https://roundtrip.api".to_string()),
            headers,
            rate_limit: Some(rate_limit),
        };
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ProviderConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.name, deserialized.name);
        assert_eq!(config.enabled, deserialized.enabled);
        assert_eq!(config.endpoint, deserialized.endpoint);
        assert!(deserialized.headers.is_some());
        assert!(deserialized.rate_limit.is_some());
    }

    // --- ExtractionConfig default_* tests ---

    #[test]
    fn test_extraction_config_custom() {
        let config = ExtractionConfig {
            extract_links: false,
            extract_images: true,
            extract_metadata: false,
            max_links: 50,
            max_images: 100,
            content_selectors: vec!["div.content".to_string()],
            exclude_selectors: vec!["nav".to_string()],
        };
        assert!(!config.extract_links);
        assert!(config.extract_images);
        assert!(!config.extract_metadata);
        assert_eq!(config.max_links, 50);
        assert_eq!(config.max_images, 100);
        assert_eq!(config.content_selectors, vec!["div.content".to_string()]);
        assert_eq!(config.exclude_selectors, vec!["nav".to_string()]);
    }

    #[test]
    fn test_extraction_config_deserialize_json() {
        let json = r#"{
            "extract_links": true,
            "extract_images": true,
            "extract_metadata": false,
            "max_links": 25,
            "max_images": 50,
            "content_selectors": ["article", "main"],
            "exclude_selectors": ["footer"]
        }"#;
        let config: ExtractionConfig = serde_json::from_str(json).unwrap();
        assert!(config.extract_links);
        assert!(config.extract_images);
        assert!(!config.extract_metadata);
        assert_eq!(config.max_links, 25);
        assert_eq!(config.max_images, 50);
        assert_eq!(
            config.content_selectors,
            vec!["article".to_string(), "main".to_string()]
        );
        assert_eq!(config.exclude_selectors, vec!["footer".to_string()]);
    }

    #[test]
    fn test_extraction_config_deserialize_yaml() {
        let yaml = r#"
extract_links: false
extract_images: true
extract_metadata: true
max_links: 0
max_images: 0
content_selectors:
  - "section"
  - "article"
exclude_selectors:
  - "aside"
"#;
        let config: ExtractionConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(!config.extract_links);
        assert!(config.extract_images);
        assert_eq!(
            config.content_selectors,
            vec!["section".to_string(), "article".to_string()]
        );
    }

    // --- CacheConfig tests ---

    #[test]
    fn test_cache_config_custom() {
        let config = CacheConfig {
            enabled: true,
            cache_dir: Some(PathBuf::from("/tmp/cache")),
            max_size_bytes: 50 * 1024 * 1024,
            ttl_secs: 1800,
            cache_search: false,
            cache_fetch: true,
        };
        assert!(config.enabled);
        assert_eq!(config.cache_dir, Some(PathBuf::from("/tmp/cache")));
        assert_eq!(config.max_size_bytes, 50 * 1024 * 1024);
        assert_eq!(config.ttl_secs, 1800);
        assert!(!config.cache_search);
        assert!(config.cache_fetch);
    }

    #[test]
    fn test_cache_config_deserialize_json() {
        let json = r#"{
            "enabled": true,
            "cache_dir": "/var/cache/mcp",
            "max_size_bytes": 209715200,
            "ttl_secs": 7200,
            "cache_search": true,
            "cache_fetch": false
        }"#;
        let config: CacheConfig = serde_json::from_str(json).unwrap();
        assert!(config.enabled);
        assert_eq!(config.cache_dir, Some(PathBuf::from("/var/cache/mcp")));
        assert_eq!(config.max_size_bytes, 209715200);
        assert_eq!(config.ttl_secs, 7200);
    }

    #[test]
    fn test_cache_config_deserialize_yaml() {
        let yaml = r#"
enabled: true
cache_dir: /tmp/search-cache
max_size_bytes: 52428800
ttl_secs: 900
"#;
        let config: CacheConfig = serde_yaml::from_str(yaml).unwrap();
        assert!(config.enabled);
        assert_eq!(config.cache_dir, Some(PathBuf::from("/tmp/search-cache")));
        assert_eq!(config.max_size_bytes, 52428800);
        assert_eq!(config.ttl_secs, 900);
    }

    // --- SecurityConfig tests ---

    #[test]
    fn test_security_config_custom() {
        let config = SecurityConfig {
            allowed_schemes: vec!["https".to_string()],
            blocked_domains: vec!["malware.com".to_string()],
            allowed_domains: vec![],
            max_redirects: 3,
            require_https: true,
            dns_servers: vec!["8.8.8.8".to_string(), "8.8.4.4".to_string()],
        };
        assert_eq!(config.allowed_schemes, vec!["https".to_string()]);
        assert!(config.is_scheme_allowed("https"));
        assert!(!config.is_scheme_allowed("http"));
        assert!(!config.is_domain_allowed("malware.com"));
        assert!(config.is_domain_allowed("safe.com"));
        assert!(config.require_https);
        assert_eq!(config.max_redirects, 3);
    }

    #[test]
    fn test_security_config_allowed_domains_override() {
        let mut config = SecurityConfig::default();
        config.allowed_domains.push("trusted.com".to_string());
        config.blocked_domains.push("malware.com".to_string());
        // allowed_domains takes priority when set
        assert!(config.is_domain_allowed("trusted.com"));
        assert!(!config.is_domain_allowed("malware.com"));
        assert!(!config.is_domain_allowed("untrusted.com"));
    }

    // --- TimeoutConfig default_* tests ---

    #[test]
    fn test_timeout_config_custom() {
        let config = TimeoutConfig {
            search_secs: 30,
            fetch_secs: 60,
            dns_secs: 10,
            connect_secs: 15,
        };
        assert_eq!(config.search_secs, 30);
        assert_eq!(config.fetch_secs, 60);
        assert_eq!(config.dns_secs, 10);
        assert_eq!(config.connect_secs, 15);
    }

    #[test]
    fn test_timeout_config_deserialize_json() {
        let json = r#"{
            "search_secs": 15,
            "fetch_secs": 45,
            "dns_secs": 3,
            "connect_secs": 5
        }"#;
        let config: TimeoutConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.search_secs, 15);
        assert_eq!(config.fetch_secs, 45);
        assert_eq!(config.dns_secs, 3);
        assert_eq!(config.connect_secs, 5);
    }

    // --- AppConfig tests ---

    #[test]
    fn test_app_config_all_defaults() {
        let config = AppConfig::new();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.search.provider, "duckduckgo");
        assert_eq!(config.search.default_limit, 10);
        assert!(config.rate_limit.enabled);
        assert_eq!(config.logging.level, "info");
        assert!(config.extraction.extract_links);
        assert!(!config.cache.enabled);
        assert!(config.security.is_scheme_allowed("http"));
        assert!(config.timeouts.search_secs > 0);
        assert!(config.timeouts.fetch_secs > 0);
    }

    #[test]
    fn test_app_config_clone() {
        let config = AppConfig::new();
        let cloned = config.clone();
        assert_eq!(config.server.host, cloned.server.host);
        assert_eq!(config.search.provider, cloned.search.provider);
        assert_eq!(
            config.rate_limit.requests_per_minute,
            cloned.rate_limit.requests_per_minute
        );
        assert_eq!(config.logging.level, cloned.logging.level);
        assert_eq!(config.cache.max_size_bytes, cloned.cache.max_size_bytes);
    }

    #[test]
    fn test_app_config_validate_valid() {
        let config = AppConfig::new();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_app_config_validate_zero_search_timeout() {
        let mut config = AppConfig::new();
        config.timeouts.search_secs = 0;
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Search timeout"));
    }

    #[test]
    fn test_app_config_validate_zero_fetch_timeout() {
        let mut config = AppConfig::new();
        config.timeouts.fetch_secs = 0;
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Fetch timeout"));
    }

    #[test]
    fn test_app_config_validate_zero_search_limit() {
        let mut config = AppConfig::new();
        config.search.default_limit = 0;
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Default search limit"));
    }

    #[test]
    fn test_app_config_validate_default_exceeds_max() {
        let mut config = AppConfig::new();
        config.search.default_limit = config.search.max_limit + 1;
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Default limit cannot exceed"));
    }

    #[test]
    fn test_app_config_validate_zero_rate_limit() {
        let mut config = AppConfig::new();
        config.rate_limit.requests_per_minute = 0;
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Requests per minute"));
    }

    #[test]
    fn test_app_config_validate_zero_cache_size() {
        let mut config = AppConfig::new();
        config.cache.max_size_bytes = 0;
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Cache max size"));
    }

    #[test]
    fn test_app_config_validate_edge_valid() {
        let mut config = AppConfig::new();
        config.search.default_limit = 5;
        config.search.max_limit = 100;
        config.search.timeout_secs = 1;
        config.rate_limit.requests_per_minute = 1;
        config.cache.max_size_bytes = 1;
        config.timeouts.search_secs = 1;
        config.timeouts.fetch_secs = 1;
        assert!(config.validate().is_ok());
    }

    // --- AppConfig::from_file() tests ---

    #[test]
    fn test_app_config_from_file_yaml() {
        let _guard = ENV_TEST_MUTEX.lock().unwrap();
        let yaml_content = r#"
server:
  host: 0.0.0.0
  port: 9090
search:
  provider: brightdata
  api_key: yaml-search-key
  default_limit: 7
  max_limit: 50
  timeout_secs: 15
rate_limit:
  enabled: false
  requests_per_minute: 20
logging:
  level: debug
  format: json
extraction:
  extract_links: true
  extract_images: true
  extract_metadata: false
cache:
  enabled: true
  max_size_bytes: 52428800
  ttl_secs: 1800
security:
  max_redirects: 5
  require_https: true
timeouts:
  search_secs: 20
  fetch_secs: 60
  dns_secs: 3
  connect_secs: 8
"#;
        let mut tmp = tempfile::Builder::new().suffix(".yaml").tempfile().unwrap();
        tmp.write_all(yaml_content.as_bytes()).unwrap();
        let path = tmp.as_ref().to_path_buf();
        let config = AppConfig::from_file(&path).unwrap();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 9090);
        assert_eq!(config.search.provider, "brightdata");
        assert_eq!(config.search.api_key, Some("yaml-search-key".to_string()));
        assert_eq!(config.search.default_limit, 7);
        assert_eq!(config.search.max_limit, 50);
        assert!(!config.rate_limit.enabled);
        assert_eq!(config.logging.level, "debug");
        assert!(config.extraction.extract_images);
        assert!(!config.extraction.extract_metadata);
        assert!(config.cache.enabled);
        assert_eq!(config.cache.max_size_bytes, 52428800);
        assert_eq!(config.cache.ttl_secs, 1800);
        assert_eq!(config.security.max_redirects, 5);
        assert!(config.security.require_https);
        assert_eq!(config.timeouts.search_secs, 20);
        assert_eq!(config.timeouts.fetch_secs, 60);
        assert_eq!(config.timeouts.dns_secs, 3);
        assert_eq!(config.timeouts.connect_secs, 8);
    }

    #[test]
    fn test_app_config_from_file_json() {
        let _guard = ENV_TEST_MUTEX.lock().unwrap();
        let json_content = r#"{
            "server": { "host": "10.0.0.1", "port": 4000 },
            "search": { "provider": "brightdata", "api_key": "json-brightdata-key", "search_engine_id": "serp_api1", "default_limit": 15 },
            "rate_limit": { "requests_per_minute": 30 },
            "logging": { "level": "warn", "format": "compact" },
            "extraction": { "extract_links": false, "max_links": 10 },
            "cache": { "enabled": false },
            "security": { "max_redirects": 2 },
            "timeouts": { "search_secs": 5, "fetch_secs": 15 }
        }"#;
        let mut tmp = tempfile::Builder::new().suffix(".json").tempfile().unwrap();
        tmp.write_all(json_content.as_bytes()).unwrap();
        let path = tmp.as_ref().to_path_buf();
        let config = AppConfig::from_file(&path).unwrap();
        assert_eq!(config.server.host, "10.0.0.1");
        assert_eq!(config.server.port, 4000);
        assert_eq!(config.search.provider, "brightdata");
        assert_eq!(
            config.search.api_key,
            Some("json-brightdata-key".to_string())
        );
        assert_eq!(config.search.default_limit, 15);
        assert_eq!(config.rate_limit.requests_per_minute, 30);
        assert_eq!(config.logging.level, "warn");
        assert!(!config.extraction.extract_links);
        assert_eq!(config.extraction.max_links, 10);
        assert_eq!(config.security.max_redirects, 2);
        assert_eq!(config.timeouts.search_secs, 5);
        assert_eq!(config.timeouts.fetch_secs, 15);
    }

    #[test]
    fn test_app_config_from_file_nonexistent() {
        let result = AppConfig::from_file(&PathBuf::from("/nonexistent/path/config.yaml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_app_config_from_file_invalid_content() {
        let _guard = ENV_TEST_MUTEX.lock().unwrap();
        let mut tmp = tempfile::Builder::new().suffix(".yaml").tempfile().unwrap();
        tmp.write_all(b"not valid {{{ yaml content").unwrap();
        let path = tmp.as_ref().to_path_buf();
        let result = AppConfig::from_file(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_app_config_from_file_empty() {
        let _guard = ENV_TEST_MUTEX.lock().unwrap();
        let mut tmp = tempfile::Builder::new().suffix(".json").tempfile().unwrap();
        tmp.write_all(b"{}").unwrap();
        let path = tmp.as_ref().to_path_buf();
        let config = AppConfig::from_file(&path).unwrap();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.search.provider, "duckduckgo");
        assert!(config.extraction.extract_links);
    }

    // --- AppConfig::from_env() tests ---

    #[test]
    fn test_app_config_from_env_all_vars() {
        let _guard = ENV_TEST_MUTEX.lock().unwrap();
        env::set_var("MCP__SERVER__HOST", "0.0.0.0");
        env::set_var("MCP__SERVER__PORT", "7777");
        env::set_var("MCP__SEARCH__PROVIDER", "serper");
        env::set_var("MCP__SEARCH__API_KEY", "env-search-key");
        env::set_var("MCP__SEARCH__DEFAULT_LIMIT", "8");
        env::set_var("MCP__SEARCH__MAX_LIMIT", "75");
        env::set_var("MCP__RATE_LIMIT__ENABLED", "false");
        env::set_var("MCP__RATE_LIMIT__REQUESTS_PER_MINUTE", "40");
        env::set_var("MCP__LOGGING__LEVEL", "trace");
        env::set_var("MCP__LOGGING__FORMAT", "compact");
        env::set_var("MCP__EXTRACTION__EXTRACT_LINKS", "true");
        env::set_var("MCP__EXTRACTION__EXTRACT_IMAGES", "true");
        env::set_var("MCP__CACHE__ENABLED", "true");
        env::set_var("MCP__CACHE__MAX_SIZE_BYTES", "268435456");
        env::set_var("MCP__SECURITY__MAX_REDIRECTS", "5");
        env::set_var("MCP__SECURITY__REQUIRE_HTTPS", "true");
        env::set_var("MCP__TIMEOUTS__SEARCH_SECS", "25");
        env::set_var("MCP__TIMEOUTS__FETCH_SECS", "90");

        let config = AppConfig::from_env().unwrap();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 7777);
        assert_eq!(config.search.provider, "serper");
        assert_eq!(config.search.api_key, Some("env-search-key".to_string()));
        assert_eq!(config.search.default_limit, 8);
        assert_eq!(config.search.max_limit, 75);
        assert!(!config.rate_limit.enabled);
        assert_eq!(config.rate_limit.requests_per_minute, 40);
        assert_eq!(config.logging.level, "trace");
        assert_eq!(config.logging.format, "compact");
        assert!(config.extraction.extract_links);
        assert!(config.extraction.extract_images);
        assert!(config.cache.enabled);
        assert_eq!(config.cache.max_size_bytes, 268435456);
        assert_eq!(config.security.max_redirects, 5);
        assert!(config.security.require_https);
        assert_eq!(config.timeouts.search_secs, 25);
        assert_eq!(config.timeouts.fetch_secs, 90);

        // Cleanup
        for var in &[
            "MCP__SERVER__HOST",
            "MCP__SERVER__PORT",
            "MCP__SEARCH__PROVIDER",
            "MCP__SEARCH__API_KEY",
            "MCP__SEARCH__DEFAULT_LIMIT",
            "MCP__SEARCH__MAX_LIMIT",
            "MCP__RATE_LIMIT__ENABLED",
            "MCP__RATE_LIMIT__REQUESTS_PER_MINUTE",
            "MCP__LOGGING__LEVEL",
            "MCP__LOGGING__FORMAT",
            "MCP__EXTRACTION__EXTRACT_LINKS",
            "MCP__EXTRACTION__EXTRACT_IMAGES",
            "MCP__CACHE__ENABLED",
            "MCP__CACHE__MAX_SIZE_BYTES",
            "MCP__SECURITY__MAX_REDIRECTS",
            "MCP__SECURITY__REQUIRE_HTTPS",
            "MCP__TIMEOUTS__SEARCH_SECS",
            "MCP__TIMEOUTS__FETCH_SECS",
        ] {
            env::remove_var(var);
        }
    }

    #[test]
    fn test_app_config_from_env_partial_vars() {
        let _guard = ENV_TEST_MUTEX.lock().unwrap();
        for var in &[
            "MCP__SERVER__HOST",
            "MCP__SERVER__PORT",
            "MCP__SEARCH__PROVIDER",
            "MCP__SEARCH__API_KEY",
            "MCP__SEARCH__DEFAULT_LIMIT",
            "MCP__RATE_LIMIT__ENABLED",
            "MCP__RATE_LIMIT__REQUESTS_PER_MINUTE",
            "MCP__LOGGING__LEVEL",
        ] {
            env::remove_var(var);
        }
        env::set_var("MCP__SEARCH__PROVIDER", "brightdata");
        env::set_var("MCP__SEARCH__MAX_LIMIT", "200");

        let config = AppConfig::from_env().unwrap();
        assert_eq!(config.search.provider, "brightdata");
        assert_eq!(config.search.max_limit, 200);
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.logging.level, "info");

        // Cleanup
        env::remove_var("MCP__SEARCH__PROVIDER");
        env::remove_var("MCP__SEARCH__MAX_LIMIT");
    }

    #[test]
    fn test_app_config_from_env_no_vars() {
        let _guard = ENV_TEST_MUTEX.lock().unwrap();
        for var in &[
            "MCP__SERVER__HOST",
            "MCP__SERVER__PORT",
            "MCP__SEARCH__PROVIDER",
            "MCP__SEARCH__API_KEY",
            "MCP__SEARCH__DEFAULT_LIMIT",
            "MCP__RATE_LIMIT__ENABLED",
            "MCP__RATE_LIMIT__REQUESTS_PER_MINUTE",
            "MCP__LOGGING__LEVEL",
        ] {
            env::remove_var(var);
        }

        let config = AppConfig::from_env().unwrap();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.search.provider, "duckduckgo");
        assert!(config.rate_limit.enabled);
        assert_eq!(config.logging.format, "pretty");
    }

    #[test]
    fn test_app_config_from_file_with_env_override() {
        let _guard = ENV_TEST_MUTEX.lock().unwrap();
        env::set_var("MCP__SEARCH__PROVIDER", "env-override-provider");

        let yaml_content = r#"
search:
  provider: yaml-provider
  default_limit: 10
"#;
        let mut tmp = tempfile::Builder::new().suffix(".yaml").tempfile().unwrap();
        tmp.write_all(yaml_content.as_bytes()).unwrap();
        let path = tmp.as_ref().to_path_buf();
        let config = AppConfig::from_file(&path).unwrap();
        assert_eq!(config.search.provider, "env-override-provider");
        assert_eq!(config.search.default_limit, 10);

        // Cleanup
        env::remove_var("MCP__SEARCH__PROVIDER");
    }

    #[test]
    fn test_app_config_from_env_integer_parsing() {
        let _guard = ENV_TEST_MUTEX.lock().unwrap();
        env::set_var("MCP__RATE_LIMIT__BURST_SIZE", "42");
        env::set_var("MCP__SEARCH__TIMEOUT_SECS", "99");
        env::set_var("MCP__CACHE__TTL_SECS", "7200");

        let config = AppConfig::from_env().unwrap();
        assert_eq!(config.rate_limit.burst_size, 42);
        assert_eq!(config.search.timeout_secs, 99);
        assert_eq!(config.cache.ttl_secs, 7200);

        // Cleanup
        env::remove_var("MCP__RATE_LIMIT__BURST_SIZE");
        env::remove_var("MCP__SEARCH__TIMEOUT_SECS");
        env::remove_var("MCP__CACHE__TTL_SECS");
    }

    #[test]
    fn test_app_config_from_file_yaml_full_roundtrip() {
        let _guard = ENV_TEST_MUTEX.lock().unwrap();
        let yaml_content = r#"
server:
  host: 0.0.0.0
  port: 8888
search:
  provider: brightdata
  api_key: roundtrip-key
  default_limit: 12
rate_limit:
  enabled: true
  requests_per_minute: 50
logging:
  level: debug
  format: json
extraction:
  extract_links: true
cache:
  enabled: true
security:
  max_redirects: 8
timeouts:
  search_secs: 10
  fetch_secs: 30
"#;
        let mut tmp = tempfile::Builder::new().suffix(".yaml").tempfile().unwrap();
        tmp.write_all(yaml_content.as_bytes()).unwrap();
        let path = tmp.as_ref().to_path_buf();
        let config = AppConfig::from_file(&path).unwrap();
        // Serialize and deserialize back
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.server.host, deserialized.server.host);
        assert_eq!(config.search.provider, deserialized.search.provider);
        assert_eq!(config.search.api_key, deserialized.search.api_key);
        assert_eq!(config.rate_limit.enabled, deserialized.rate_limit.enabled);
        assert_eq!(config.logging.level, deserialized.logging.level);
    }
}
