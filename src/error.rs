// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Ole Reinartz
//!
//! Error types for the web search MCP server

use thiserror::Error;

/// Custom error types for the MCP Server
#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("URL parsing error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("JSON serialization error: {0}")]
    JsonSerialize(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Search API error: {0}")]
    SearchApi(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Timeout error")]
    Timeout,

    #[error("MCP protocol error: {0}")]
    McpProtocol(String),
}

impl From<ServerError> for rmcp::ErrorData {
    fn from(err: ServerError) -> Self {
        match err {
            ServerError::InvalidRequest(msg) => rmcp::ErrorData::invalid_params(msg, None),
            ServerError::NotFound(msg) => rmcp::ErrorData::resource_not_found(msg, None),
            ServerError::RateLimitExceeded => {
                rmcp::ErrorData::invalid_params("Rate limit exceeded".to_string(), None)
            }
            ServerError::Timeout => {
                rmcp::ErrorData::internal_error("Request timed out".to_string(), None)
            }
            _ => rmcp::ErrorData::internal_error(err.to_string(), None::<serde_json::Value>),
        }
    }
}

/// Result type alias for the Server
pub type ServerResult<T> = std::result::Result<T, ServerError>;

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::model::ErrorCode;

    // ========================================================================
    // Display impl tests — all 10 ServerError variants
    // ========================================================================

    #[tokio::test]
    async fn display_network_error() {
        let result = reqwest::Client::new()
            .get("http://localhost:1")
            .send()
            .await;
        let err = match result {
            Ok(resp) => ServerError::Network(resp.error_for_status().unwrap_err()),
            Err(e) => ServerError::Network(e),
        };
        assert!(err.to_string().contains("Network error"));
    }

    #[test]
    fn display_url_parse_error() {
        let err = ServerError::UrlParse(url::ParseError::EmptyHost);
        assert_eq!(err.to_string(), "URL parsing error: empty host");
    }

    #[test]
    fn display_json_serialize_error() {
        let err = ServerError::JsonSerialize(
            serde_json::from_str::<serde_json::Value>("not json").unwrap_err(),
        );
        assert_eq!(
            err.to_string(),
            "JSON serialization error: expected ident at line 1 column 2"
        );
    }

    #[test]
    fn display_config_error() {
        let err = ServerError::Config("missing api key".to_string());
        assert_eq!(err.to_string(), "Configuration error: missing api key");
    }

    #[test]
    fn display_search_api_error() {
        let err = ServerError::SearchApi("404 Not Found".to_string());
        assert_eq!(err.to_string(), "Search API error: 404 Not Found");
    }

    #[test]
    fn display_rate_limit_exceeded() {
        let err = ServerError::RateLimitExceeded;
        assert_eq!(err.to_string(), "Rate limit exceeded");
    }

    #[test]
    fn display_invalid_request() {
        let err = ServerError::InvalidRequest("missing query".to_string());
        assert_eq!(err.to_string(), "Invalid request: missing query");
    }

    #[test]
    fn display_not_found() {
        let err = ServerError::NotFound("resource id=42".to_string());
        assert_eq!(err.to_string(), "Resource not found: resource id=42");
    }

    #[test]
    fn display_timeout() {
        let err = ServerError::Timeout;
        assert_eq!(err.to_string(), "Timeout error");
    }

    #[test]
    fn display_mcp_protocol_error() {
        let err = ServerError::McpProtocol("invalid method".to_string());
        assert_eq!(err.to_string(), "MCP protocol error: invalid method");
    }

    // ========================================================================
    // From<ServerError> for rmcp::ErrorData — verify correct MCP error codes
    // ========================================================================

    #[test]
    fn from_invalid_request_provides_invalid_params() {
        let err = ServerError::InvalidRequest("bad param".to_string());
        let error_data: rmcp::ErrorData = err.into();
        assert_eq!(error_data.code, ErrorCode::INVALID_PARAMS);
        assert_eq!(error_data.message, "bad param");
        assert!(error_data.data.is_none());
    }

    #[test]
    fn from_not_found_provides_resource_not_found() {
        let err = ServerError::NotFound("missing item".to_string());
        let error_data: rmcp::ErrorData = err.into();
        assert_eq!(error_data.code, ErrorCode::RESOURCE_NOT_FOUND);
        assert_eq!(error_data.message, "missing item");
        assert!(error_data.data.is_none());
    }

    #[test]
    fn from_rate_limit_exceeded_provides_invalid_params() {
        let err = ServerError::RateLimitExceeded;
        let error_data: rmcp::ErrorData = err.into();
        assert_eq!(error_data.code, ErrorCode::INVALID_PARAMS);
        assert_eq!(error_data.message, "Rate limit exceeded");
        assert!(error_data.data.is_none());
    }

    #[test]
    fn from_timeout_provides_internal_error() {
        let err = ServerError::Timeout;
        let error_data: rmcp::ErrorData = err.into();
        assert_eq!(error_data.code, ErrorCode::INTERNAL_ERROR);
        assert_eq!(error_data.message, "Request timed out");
        assert!(error_data.data.is_none());
    }

    #[tokio::test]
    async fn from_network_provides_internal_error() {
        let result = reqwest::Client::new()
            .get("http://localhost:1")
            .send()
            .await;
        let err = match result {
            Ok(resp) => ServerError::Network(resp.error_for_status().unwrap_err()),
            Err(e) => ServerError::Network(e),
        };
        let error_data: rmcp::ErrorData = err.into();
        assert_eq!(error_data.code, ErrorCode::INTERNAL_ERROR);
        assert!(error_data.message.contains("Network error"));
        assert!(error_data.data.is_none());
    }

    #[test]
    fn from_url_parse_provides_internal_error() {
        let err = ServerError::UrlParse(url::ParseError::EmptyHost);
        let error_data: rmcp::ErrorData = err.into();
        assert_eq!(error_data.code, ErrorCode::INTERNAL_ERROR);
        assert_eq!(error_data.message, "URL parsing error: empty host");
        assert!(error_data.data.is_none());
    }

    #[test]
    fn from_json_serialize_provides_internal_error() {
        let err = ServerError::JsonSerialize(
            serde_json::from_str::<serde_json::Value>("bad").unwrap_err(),
        );
        let error_data: rmcp::ErrorData = err.into();
        assert_eq!(error_data.code, ErrorCode::INTERNAL_ERROR);
        assert!(error_data.message.contains("JSON serialization error"));
        assert!(error_data.data.is_none());
    }

    #[test]
    fn from_config_provides_internal_error() {
        let err = ServerError::Config("bad config".to_string());
        let error_data: rmcp::ErrorData = err.into();
        assert_eq!(error_data.code, ErrorCode::INTERNAL_ERROR);
        assert_eq!(error_data.message, "Configuration error: bad config");
        assert!(error_data.data.is_none());
    }

    #[test]
    fn from_search_api_provides_internal_error() {
        let err = ServerError::SearchApi("api error".to_string());
        let error_data: rmcp::ErrorData = err.into();
        assert_eq!(error_data.code, ErrorCode::INTERNAL_ERROR);
        assert_eq!(error_data.message, "Search API error: api error");
        assert!(error_data.data.is_none());
    }

    #[test]
    fn from_mcp_protocol_provides_internal_error() {
        let err = ServerError::McpProtocol("protocol fail".to_string());
        let error_data: rmcp::ErrorData = err.into();
        assert_eq!(error_data.code, ErrorCode::INTERNAL_ERROR);
        assert_eq!(error_data.message, "MCP protocol error: protocol fail");
        assert!(error_data.data.is_none());
    }

    // ========================================================================
    // ServerResult type alias
    // ========================================================================

    #[test]
    fn server_result_is_ok() {
        let result: ServerResult<i32> = Ok(42);
        assert!(matches!(result, Ok(42)));
    }

    #[test]
    fn server_result_is_err() {
        let result: ServerResult<i32> = Err(ServerError::Timeout);
        assert!(result.is_err());
    }
}
