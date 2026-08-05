// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Ole Reinartz
//!
//! RMCP server implementation

use crate::error::ServerResult;
use crate::handlers::{FetchHandler, SearchHandler};
use crate::models::config::Config;
use crate::services::{FetchService, SearchService};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, ServerCapabilities,
    ServerInfo,
};
use rmcp::schemars::JsonSchema;
use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager,
    tower::{StreamableHttpServerConfig, StreamableHttpService},
};
use rmcp::{
    service::RequestContext, tool, tool_handler, tool_router, ErrorData, RoleServer, ServerHandler,
    ServiceExt,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::CorsLayer;
use tracing::{debug, info};

/// RMCP Server for web search and fetch operations
#[derive(Clone)]
pub struct Server {
    config: Config,
    search_handler: Arc<SearchHandler>,
    fetch_handler: Arc<FetchHandler>,
}

impl Server {
    /// Create a new RMCP server
    pub async fn new(config: Config) -> ServerResult<Self> {
        info!("Initializing RMCP server...");

        // Initialize services
        let search_service = SearchService::new(config.search.clone()).await?;
        let fetch_service = FetchService::new(config.search.timeout_secs)?;

        // Initialize handlers
        let search_handler = Arc::new(SearchHandler::new(Arc::new(search_service)));
        let fetch_handler = Arc::new(FetchHandler::new(Arc::new(fetch_service)));

        info!("RMCP server initialized successfully");
        Ok(Self {
            config,
            search_handler,
            fetch_handler,
        })
    }

    /// Run the server in STDIO mode (standard for MCP)
    pub async fn run_stdio(&self) -> anyhow::Result<()> {
        info!("Starting RMCP server in STDIO mode...");
        let transport = rmcp::transport::stdio();
        let _server_handle = self.clone().serve(transport).await?;
        // Wait indefinitely
        tokio::signal::ctrl_c().await?;
        Ok(())
    }

    /// Run the server in Streamable HTTP mode
    pub async fn run_http(&self) -> anyhow::Result<()> {
        info!("Starting RMCP server in Streamable HTTP mode...");

        let server = self.clone();
        let http_cfg = &server.config.server.http;
        let host = server.config.server.host.clone();
        let port = server.config.server.port;
        let path = http_cfg.path.clone();
        let allowed_hosts = if http_cfg.allowed_hosts.is_empty() {
            vec!["localhost".to_string(), "127.0.0.1".to_string()]
        } else {
            http_cfg.allowed_hosts.clone()
        };
        let sse_keep_alive = http_cfg.sse_keep_alive_secs;
        let json_response = http_cfg.json_response;
        let legacy_session_mode = http_cfg.legacy_session_mode;

        let cors_origin = http_cfg.cors_origin.clone();

        let config = StreamableHttpServerConfig::default()
            .with_sse_keep_alive(sse_keep_alive.map(Duration::from_secs))
            .with_json_response(json_response)
            .with_allowed_hosts(allowed_hosts)
            .with_legacy_session_mode(legacy_session_mode);

        let service = StreamableHttpService::new(
            move || Ok(server.clone()),
            Arc::new(LocalSessionManager::default()),
            config,
        );

        let addr = format!("{}:{}", host, port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        info!(
            "MCP Streamable HTTP server listening on http://{}{} (cors_origin: {})",
            addr, path, cors_origin
        );

        let cors = if cors_origin == "*" {
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods([
                    axum::http::Method::GET,
                    axum::http::Method::POST,
                    axum::http::Method::OPTIONS,
                ])
                .allow_headers(tower_http::cors::Any)
        } else {
            let origin = cors_origin
                .parse::<axum::http::HeaderValue>()
                .unwrap_or_else(|_| axum::http::HeaderValue::from_static("*"));
            CorsLayer::new()
                .allow_origin(origin)
                .allow_methods([
                    axum::http::Method::GET,
                    axum::http::Method::POST,
                    axum::http::Method::OPTIONS,
                ])
                .allow_headers([
                    axum::http::header::CONTENT_TYPE,
                    axum::http::header::ACCEPT,
                    axum::http::header::AUTHORIZATION,
                    axum::http::header::HeaderName::from_static("x-requested-with"),
                ])
        };

        let app = axum::Router::new().nest_service(&path, service).layer(cors);
        axum::serve(listener, app).await?;
        Ok(())
    }
}

#[tool_router]
impl Server {
    /// Search the web for information
    #[tool(
        name = "web_search",
        description = "Search the web for information using various search providers (DuckDuckGo, Google, Bing, Serper)"
    )]
    async fn search(
        &self,
        Parameters(SearchRequestParams {
            query,
            limit,
            offset,
            language,
            safe_search,
            region,
        }): Parameters<SearchRequestParams>,
    ) -> Result<String, ErrorData> {
        debug!(
            "Handling search request: query='{}', limit={:?}",
            query, limit
        );

        if query.trim().is_empty() {
            let empty = crate::models::SearchResponse {
                results: vec![],
                total_results: 0,
                count: 0,
                query: query.clone(),
                offset: offset.unwrap_or(0),
                search_time_ms: None,
            };
            return serde_json::to_string(&empty)
                .map_err(|e| ErrorData::internal_error(e.to_string(), None));
        }

        let search_params = crate::models::SearchParams {
            query,
            limit: limit.unwrap_or(10),
            offset: offset.unwrap_or(0),
            language,
            safe_search: safe_search.unwrap_or_default(),
            region,
        };

        let response = self.search_handler.service().search(search_params).await;
        match response {
            Ok(resp) => serde_json::to_string(&resp)
                .map_err(|e| ErrorData::internal_error(e.to_string(), None)),
            Err(e) => Err(ErrorData::internal_error(e.to_string(), None)),
        }
    }

    /// Fetch and parse a webpage
    #[tool(
        name = "web_fetch",
        description = "Fetch and parse a webpage, extracting content and optionally links and images"
    )]
    async fn fetch(
        &self,
        Parameters(FetchRequestParams {
            url,
            include_raw_html,
            extract_links,
            extract_images,
            timeout_secs,
            user_agent,
            max_content_size,
        }): Parameters<FetchRequestParams>,
    ) -> Result<String, ErrorData> {
        debug!("Handling fetch request: url='{}'", url);

        let fetch_params = crate::models::FetchParams {
            url,
            include_raw_html: include_raw_html.unwrap_or(false),
            extract_links: extract_links.unwrap_or(true),
            extract_images: extract_images.unwrap_or(false),
            timeout_secs: timeout_secs.unwrap_or(30),
            user_agent,
            max_content_size: max_content_size.unwrap_or(10 * 1024 * 1024),
        };

        let content = self.fetch_handler.service().fetch(fetch_params).await;
        match content {
            Ok(c) => {
                let json = serde_json::to_string(&c)
                    .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
                Ok(json)
            }
            Err(e) => Err(ErrorData::internal_error(e.to_string(), None)),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct SearchRequestParams {
    /// The search query string (required)
    pub query: String,

    /// Maximum number of results to return (default: 10, max: 100)
    #[serde(default)]
    pub limit: Option<u32>,

    /// Offset for pagination (default: 0)
    #[serde(default)]
    pub offset: Option<u32>,

    /// Language code (e.g., "en", "es", "fr", "de")
    #[serde(default)]
    pub language: Option<String>,

    /// Safe search filter level
    #[serde(default)]
    pub safe_search: Option<crate::models::SafeSearch>,

    /// Region code for localized results (e.g., "us-en", "uk-en")
    #[serde(default)]
    pub region: Option<String>,
}

#[derive(Debug, Deserialize, Default, Serialize, JsonSchema)]
pub struct FetchRequestParams {
    /// URL to fetch
    pub url: String,

    /// Include raw HTML in response
    #[serde(default)]
    pub include_raw_html: Option<bool>,

    /// Extract links from the page
    #[serde(default)]
    pub extract_links: Option<bool>,

    /// Extract images from the page
    #[serde(default)]
    pub extract_images: Option<bool>,

    /// Timeout in seconds
    #[serde(default)]
    pub timeout_secs: Option<u64>,

    /// User agent string
    #[serde(default)]
    pub user_agent: Option<String>,

    /// Maximum content size in bytes
    #[serde(default)]
    pub max_content_size: Option<usize>,
}

#[tool_handler(name = "gannet-mcp")]
impl ServerHandler for Server {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<CallToolResponse, ErrorData> {
        match request.name.as_ref() {
            "web_search" => {
                let args: SearchRequestParams = serde_json::from_value(serde_json::Value::Object(
                    request.arguments.unwrap_or_default(),
                ))
                .map_err(|e| ErrorData::invalid_params(e.to_string(), None))?;

                let result = match self.search(Parameters(args)).await {
                    Ok(r) => r,
                    Err(e) => return Err(ErrorData::internal_error(e.to_string(), None)),
                };

                Ok(CallToolResponse::Complete(CallToolResult::success(vec![
                    ContentBlock::text(result),
                ])))
            }
            "web_fetch" => {
                let args: FetchRequestParams = serde_json::from_value(serde_json::Value::Object(
                    request.arguments.unwrap_or_default(),
                ))
                .map_err(|e| ErrorData::invalid_params(e.to_string(), None))?;

                let result = match self.fetch(Parameters(args)).await {
                    Ok(r) => r,
                    Err(e) => return Err(e),
                };

                Ok(CallToolResponse::Complete(CallToolResult::success(vec![
                    ContentBlock::text(result),
                ])))
            }
            _ => Err(ErrorData::invalid_params(
                format!("Unknown tool: {}", request.name),
                None,
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SafeSearch;

    // --- SearchRequestParams serde / default tests ---

    #[test]
    fn test_search_request_params_full_json() {
        let json = r#"{
            "query": "rust programming",
            "limit": 25,
            "offset": 5,
            "language": "en",
            "safe_search": "strict",
            "region": "us-en"
        }"#;
        let params: SearchRequestParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.query, "rust programming");
        assert_eq!(params.limit, Some(25));
        assert_eq!(params.offset, Some(5));
        assert_eq!(params.language, Some("en".to_string()));
        assert_eq!(params.safe_search, Some(SafeSearch::Strict));
        assert_eq!(params.region, Some("us-en".to_string()));
    }

    #[test]
    fn test_search_request_params_minimal_json() {
        let json = r#"{"query": "test query"}"#;
        let params: SearchRequestParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.query, "test query");
        assert!(params.limit.is_none());
        assert!(params.offset.is_none());
        assert!(params.language.is_none());
        assert!(params.safe_search.is_none());
        assert!(params.region.is_none());
    }

    #[test]
    fn test_search_request_params_empty_object_defaults() {
        let json = r#"{}"#;
        let result: Result<SearchRequestParams, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_search_request_params_missing_query_is_error() {
        let json = r#"{"limit": 5}"#;
        let result: Result<SearchRequestParams, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_search_request_params_safe_search_off() {
        let json = r#"{"query": "test", "safe_search": "off"}"#;
        let params: SearchRequestParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.safe_search, Some(SafeSearch::Off));
    }

    #[test]
    fn test_search_request_params_safe_search_moderate() {
        let json = r#"{"query": "test", "safe_search": "moderate"}"#;
        let params: SearchRequestParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.safe_search, Some(SafeSearch::Moderate));
    }

    #[test]
    fn test_search_request_params_serialization_roundtrip() {
        let params = SearchRequestParams {
            query: "hello world".to_string(),
            limit: Some(15),
            offset: Some(3),
            language: Some("es".to_string()),
            safe_search: Some(SafeSearch::Strict),
            region: Some("mx-es".to_string()),
        };
        let json = serde_json::to_string(&params).unwrap();
        let deserialized: SearchRequestParams = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.query, params.query);
        assert_eq!(deserialized.limit, params.limit);
        assert_eq!(deserialized.offset, params.offset);
        assert_eq!(deserialized.language, params.language);
        assert_eq!(deserialized.safe_search, params.safe_search);
        assert_eq!(deserialized.region, params.region);
    }

    // --- FetchRequestParams serde / default tests ---

    #[test]
    fn test_fetch_request_params_full_json() {
        let json = r#"{
            "url": "https://example.com",
            "include_raw_html": true,
            "extract_links": false,
            "extract_images": true,
            "timeout_secs": 60,
            "user_agent": "my-agent/1.0",
            "max_content_size": 5242880
        }"#;
        let params: FetchRequestParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.url, "https://example.com");
        assert_eq!(params.include_raw_html, Some(true));
        assert_eq!(params.extract_links, Some(false));
        assert_eq!(params.extract_images, Some(true));
        assert_eq!(params.timeout_secs, Some(60));
        assert_eq!(params.user_agent, Some("my-agent/1.0".to_string()));
        assert_eq!(params.max_content_size, Some(5242880));
    }

    #[test]
    fn test_fetch_request_params_minimal_json() {
        let json = r#"{"url": "https://example.com"}"#;
        let params: FetchRequestParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.url, "https://example.com");
        assert!(params.include_raw_html.is_none());
        assert!(params.extract_links.is_none());
        assert!(params.extract_images.is_none());
        assert!(params.timeout_secs.is_none());
        assert!(params.user_agent.is_none());
        assert!(params.max_content_size.is_none());
    }

    #[test]
    fn test_fetch_request_params_empty_object_defaults() {
        let json = r#"{}"#;
        let result: Result<FetchRequestParams, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_fetch_request_params_missing_url_is_error() {
        let json = r#"{"include_raw_html": true}"#;
        let result: Result<FetchRequestParams, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_fetch_request_params_default_derive() {
        let params = FetchRequestParams::default();
        assert_eq!(params.url, "");
        assert_eq!(params.include_raw_html, None);
        assert_eq!(params.extract_links, None);
        assert_eq!(params.extract_images, None);
        assert_eq!(params.timeout_secs, None);
        assert_eq!(params.user_agent, None);
        assert_eq!(params.max_content_size, None);
    }

    #[test]
    fn test_fetch_request_params_partial_json() {
        let json = r#"{"url": "https://example.com", "timeout_secs": 45}"#;
        let params: FetchRequestParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.url, "https://example.com");
        assert_eq!(params.timeout_secs, Some(45));
        assert_eq!(params.include_raw_html, None);
        assert_eq!(params.extract_links, None);
    }

    #[test]
    fn test_fetch_request_params_serialization_roundtrip() {
        let params = FetchRequestParams {
            url: "https://example.com/page".to_string(),
            include_raw_html: Some(true),
            extract_links: Some(false),
            extract_images: Some(true),
            timeout_secs: Some(120),
            user_agent: Some("test-agent".to_string()),
            max_content_size: Some(1048576),
        };
        let json = serde_json::to_string(&params).unwrap();
        let deserialized: FetchRequestParams = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.url, params.url);
        assert_eq!(deserialized.include_raw_html, params.include_raw_html);
        assert_eq!(deserialized.extract_links, params.extract_links);
        assert_eq!(deserialized.extract_images, params.extract_images);
        assert_eq!(deserialized.timeout_secs, params.timeout_secs);
        assert_eq!(deserialized.user_agent, params.user_agent);
        assert_eq!(deserialized.max_content_size, params.max_content_size);
    }

    // --- ServerHandler::get_info() tests ---

    #[tokio::test]
    async fn test_server_get_info_returns_tools_capability() {
        let server = Server::new(Config::default()).await.unwrap();
        let info = server.get_info();
        // Should have tools capability enabled
        assert!(info.capabilities.tools.is_some());
    }

    #[tokio::test]
    async fn test_server_get_info_returns_server_info() {
        let server = Server::new(Config::default()).await.unwrap();
        let info = server.get_info();
        // ServerInfo should be valid and non-empty
        assert!(!info.server_info.name.is_empty());
    }

    #[tokio::test]
    async fn test_server_get_info_protocol_version() {
        let server = Server::new(Config::default()).await.unwrap();
        let info = server.get_info();
        assert_eq!(info.protocol_version.as_str(), "2025-11-25");
    }

    // --- ServerHandler::call_tool() edge case tests ---
    // (call_tool tests moved to integration tests in steps 10-11
    //  due to #[non_exhaustive] on CallToolRequestParams in rmcp)

    #[test]
    fn test_call_tool_args_deserialization_invalid_query_type() {
        // Invalid JSON value for query (number instead of string)
        let args: serde_json::Map<String, serde_json::Value> =
            serde_json::from_value(serde_json::json!({ "query": 12345 })).unwrap();
        let result: Result<SearchRequestParams, _> =
            serde_json::from_value(serde_json::Value::Object(args));
        assert!(result.is_err());
    }

    #[test]
    fn test_call_tool_args_deserialization_missing_query() {
        // Missing the required "query" field
        let args: serde_json::Map<String, serde_json::Value> =
            serde_json::from_value(serde_json::json!({ "limit": 5 })).unwrap();
        let result: Result<SearchRequestParams, _> =
            serde_json::from_value(serde_json::Value::Object(args));
        assert!(result.is_err());
    }

    #[test]
    fn test_call_tool_args_deserialization_missing_url() {
        // Missing the required "url" field
        let args: serde_json::Map<String, serde_json::Value> =
            serde_json::from_value(serde_json::json!({ "timeout_secs": 10 })).unwrap();
        let result: Result<FetchRequestParams, _> =
            serde_json::from_value(serde_json::Value::Object(args));
        assert!(result.is_err());
    }

    #[test]
    fn test_call_tool_args_deserialization_invalid_url_type() {
        // url is a number instead of string
        let args: serde_json::Map<String, serde_json::Value> =
            serde_json::from_value(serde_json::json!({ "url": 42 })).unwrap();
        let result: Result<FetchRequestParams, _> =
            serde_json::from_value(serde_json::Value::Object(args));
        assert!(result.is_err());
    }

    #[test]
    fn test_call_tool_args_deserialization_empty_args() {
        // Empty object, no query
        let args: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
        let result: Result<SearchRequestParams, _> =
            serde_json::from_value(serde_json::Value::Object(args));
        assert!(result.is_err());
    }

    #[test]
    fn test_search_request_params_all_safe_search_variants() {
        let variants = ["off", "moderate", "strict"];
        for variant in variants {
            let json = format!(r#"{{"query": "test", "safe_search": "{}"}}"#, variant);
            let params: SearchRequestParams = serde_json::from_str(&json).unwrap();
            assert!(
                params.safe_search.is_some(),
                "Expected safe_search for variant '{}'",
                variant
            );
        }
    }

    #[test]
    fn test_fetch_request_params_bool_defaults() {
        // Verify that bool optional fields are None (not false) when omitted
        let json = r#"{"url": "https://example.com"}"#;
        let params: FetchRequestParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.include_raw_html, None);
        assert_eq!(params.extract_links, None);
        assert_eq!(params.extract_images, None);
    }

    #[test]
    fn test_fetch_request_params_zero_timeout() {
        // Timeout of 0 should be valid (just zero)
        let json = r#"{"url": "https://example.com", "timeout_secs": 0}"#;
        let params: FetchRequestParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.timeout_secs, Some(0));
    }

    #[test]
    fn test_fetch_request_params_large_max_content_size() {
        let json = r#"{
            "url": "https://example.com",
            "max_content_size": 104857600
        }"#;
        let params: FetchRequestParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.max_content_size, Some(104857600));
    }
}
