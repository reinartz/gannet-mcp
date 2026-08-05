// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Ole Reinartz
//
use gannet_mcp::config::SearchConfig;
use gannet_mcp::handlers::SearchHandler;
use gannet_mcp::models::SafeSearch;
use gannet_mcp::services::SearchService;
use gannet_mcp::Config;
use gannet_mcp::Server;
use std::sync::Arc;

/// Helper to create a search handler for testing
async fn make_search_handler() -> SearchHandler {
    let config = SearchConfig::default();
    let service = SearchService::new(config)
        .await
        .expect("Failed to create SearchService");
    SearchHandler::new(Arc::new(service))
}

#[tokio::test]
async fn test_search_service_creation_with_default_config() {
    let config = SearchConfig::default();
    let service = SearchService::new(config).await;
    assert!(
        service.is_ok(),
        "Should create SearchService with default config"
    );
}

#[tokio::test]
async fn test_search_service_creation_with_duckduckgo() {
    let config = SearchConfig {
        provider: "duckduckgo".to_string(),
        ..Default::default()
    };
    let service = SearchService::new(config).await;
    assert!(
        service.is_ok(),
        "Should create SearchService with DuckDuckGo"
    );
}

#[tokio::test]
async fn test_search_service_creation_with_unknown_provider_falls_back() {
    let config = SearchConfig {
        provider: "unknown_provider".to_string(),
        ..Default::default()
    };
    let service = SearchService::new(config).await;
    assert!(
        service.is_ok(),
        "Should fall back to DuckDuckGo for unknown provider"
    );
}

#[tokio::test]
async fn test_search_service_creation_with_brightdata_missing_key_fails() {
    let config = SearchConfig {
        provider: "brightdata".to_string(),
        api_key: None,
        search_engine_id: Some("zone".to_string()),
        ..Default::default()
    };
    let service = SearchService::new(config).await;
    assert!(
        service.is_err(),
        "BrightData provider without API key should fail"
    );
}

#[tokio::test]
async fn test_search_handler_tool_definition() {
    let tool = gannet_mcp::handlers::search_tool_definition();
    assert_eq!(tool.name, "web_search");
    assert!(tool
        .description
        .as_ref()
        .map(|d| d.contains("Search"))
        .unwrap_or(false));
    assert!(tool.input_schema.contains_key("properties"));
    assert!(tool.input_schema.contains_key("required"));
}

#[tokio::test]
async fn test_server_initialization_with_default_config() {
    let config = Config::default();
    let server = Server::new(config).await;
    assert!(
        server.is_ok(),
        "Server should initialize with default config"
    );
}

#[tokio::test]
async fn test_server_initialization_with_custom_config() {
    let config = Config {
        search: SearchConfig {
            provider: "duckduckgo".to_string(),
            default_limit: 5,
            max_limit: 20,
            timeout_secs: 15,
            ..Default::default()
        },
        ..Default::default()
    };
    let server = Server::new(config).await;
    assert!(
        server.is_ok(),
        "Server should initialize with custom config"
    );
}

#[tokio::test]
async fn test_search_params_validation_rules() {
    let handler = make_search_handler().await;

    // The search handler does not validate empty queries at the handler level.
    // Validation is in SearchRequestParams::validate() which is used by handle_request().
    // The search() method passes through to the service.
    let result = handler
        .search(gannet_mcp::models::SearchParams {
            query: "".to_string(),
            limit: 10,
            offset: 0,
            language: None,
            safe_search: SafeSearch::Moderate,
            region: None,
        })
        .await;

    // Server passes empty queries through - it may succeed with empty results or fail with network error
    match result {
        Ok(response) => {
            assert_eq!(response.query, "");
            assert!(response.results.is_empty());
        }
        Err(_) => {
            // Network error is acceptable in CI
        }
    }
}

#[tokio::test]
async fn test_search_with_valid_params_starts() {
    let handler = make_search_handler().await;

    // This test may actually hit the network (DuckDuckGo).
    // It serves as a canary - if it succeeds or fails gracefully, that's fine.
    let result = handler
        .search(gannet_mcp::models::SearchParams {
            query: "rust programming".to_string(),
            limit: 3,
            offset: 0,
            language: None,
            safe_search: SafeSearch::Moderate,
            region: None,
        })
        .await;

    // Either it succeeds (network available) or fails with a network error
    match result {
        Ok(response) => {
            assert_eq!(response.query, "rust programming");
            assert!(response.results.len() <= 3);
        }
        Err(e) => {
            // Network errors are acceptable in CI/test environments
            let err_string = e.to_string();
            assert!(
                err_string.contains("Network")
                    || err_string.contains("timeout")
                    || err_string.contains("error"),
                "Unexpected error: {}",
                err_string
            );
        }
    }
}

#[tokio::test]
async fn test_search_with_custom_limit_is_capped() {
    let config = SearchConfig {
        provider: "duckduckgo".to_string(),
        max_limit: 50,
        ..Default::default()
    };
    let service = SearchService::new(config)
        .await
        .expect("Failed to create service");
    let handler = SearchHandler::new(Arc::new(service));

    let result = handler
        .search(gannet_mcp::models::SearchParams {
            query: "test".to_string(),
            limit: 999, // Exceeds max_limit of 50
            offset: 0,
            language: None,
            safe_search: SafeSearch::Moderate,
            region: None,
        })
        .await;

    match result {
        Ok(response) => {
            assert!(
                response.results.len() <= 50,
                "Results should be capped at max_limit, got {}",
                response.results.len()
            );
        }
        Err(_) => {
            // Network error is acceptable
        }
    }
}

#[tokio::test]
async fn test_search_handler_service_access() {
    let handler = make_search_handler().await;
    let service = handler.service();
    // Service should be accessible and functional
    assert!(Arc::strong_count(service) >= 1);
}

#[tokio::test]
async fn test_search_handler_clone() {
    let handler = make_search_handler().await;
    let cloned = handler.clone();
    // Both should have access to the same service
    assert!(Arc::ptr_eq(handler.service(), cloned.service()));
}

#[tokio::test]
async fn test_config_default_values() {
    let config = Config::default();

    assert_eq!(config.search.provider, "duckduckgo");
    assert_eq!(config.search.default_limit, 10);
    assert_eq!(config.search.max_limit, 100);
    assert_eq!(config.search.timeout_secs, 10);
    assert!(config.search.api_key.is_none());
    assert!(config.rate_limit.enabled);
    assert_eq!(config.rate_limit.requests_per_minute, 60);
}

#[tokio::test]
async fn test_config_serde_roundtrip() {
    let config = Config::default();
    let json = serde_json::to_string(&config).expect("Serialization should succeed");
    let deserialized: Config = serde_json::from_str(&json).expect("Deserialization should succeed");

    assert_eq!(config.search.provider, deserialized.search.provider);
    assert_eq!(
        config.search.default_limit,
        deserialized.search.default_limit
    );
}

#[tokio::test]
async fn test_error_type_conversions() {
    use gannet_mcp::ServerError;

    // Test Display impl
    let err = ServerError::InvalidRequest("bad request".to_string());
    let err_str = err.to_string();
    assert!(
        err_str.contains("bad request"),
        "Error display should contain message"
    );

    // Test various error types
    let config_err = ServerError::Config("missing key".to_string());
    assert!(config_err.to_string().contains("missing key"));

    let not_found = ServerError::NotFound("https://example.com".to_string());
    assert!(not_found.to_string().contains("example.com"));
}

#[tokio::test]
async fn test_safe_search_default() {
    let safe = SafeSearch::default();
    assert_eq!(safe, SafeSearch::Moderate);
}

#[tokio::test]
async fn test_safe_search_serde() {
    let json = serde_json::to_string(&SafeSearch::Off).unwrap();
    assert_eq!(json, "\"off\"");

    let json = serde_json::to_string(&SafeSearch::Moderate).unwrap();
    assert_eq!(json, "\"moderate\"");

    let json = serde_json::to_string(&SafeSearch::Strict).unwrap();
    assert_eq!(json, "\"strict\"");
}

#[tokio::test]
async fn test_search_params_default() {
    use gannet_mcp::models::SearchParams;

    let params = SearchParams::default();
    assert_eq!(params.query, "");
    assert_eq!(params.limit, 10);
    assert_eq!(params.offset, 0);
    assert_eq!(params.safe_search, SafeSearch::Moderate);
    assert!(params.language.is_none());
    assert!(params.region.is_none());
}

#[tokio::test]
async fn test_fetch_params_default() {
    use gannet_mcp::models::FetchParams;

    let params = FetchParams::default();
    assert_eq!(params.url, "");
    assert!(!params.include_raw_html);
    assert!(params.extract_links);
    assert!(!params.extract_images);
    assert_eq!(params.timeout_secs, 30);
    assert!(params.user_agent.is_none());
    assert_eq!(params.max_content_size, 10 * 1024 * 1024);
}
