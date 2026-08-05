// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Ole Reinartz
//!
//! Search handler for MCP protocol
//!
//! This module provides the MCP tool handler for web search operations.

use crate::error::{ServerError, ServerResult};
use crate::models::{SafeSearch, SearchParams, SearchResponse};
use crate::services::SearchService;
use rmcp::model::{CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};

/// Handler for search-related MCP requests
#[derive(Clone)]
pub struct SearchHandler {
    /// The search service for performing searches
    service: Arc<SearchService>,
}

impl SearchHandler {
    /// Create a new search handler
    ///
    /// # Arguments
    ///
    /// * `service` - The search service to use for performing searches
    pub fn new(service: Arc<SearchService>) -> Self {
        Self { service }
    }

    /// Handle a search request
    ///
    /// # Arguments
    ///
    /// * `request` - The MCP call tool request containing search parameters
    ///
    /// # Returns
    ///
    /// A `CallToolResponse` containing the search results as JSON
    pub async fn handle_request(
        &self,
        request: &CallToolRequestParams,
    ) -> ServerResult<CallToolResponse> {
        let params: SearchRequestParams = serde_json::from_value(serde_json::Value::Object(
            request.arguments.clone().unwrap_or_default(),
        ))
        .map_err(|e| ServerError::InvalidRequest(format!("Invalid parameters: {}", e)))?;

        debug!(
            "Handling search request: query='{}', limit={:?}",
            params.query, params.limit
        );

        let search_params = SearchParams {
            query: params.query,
            limit: params.limit.unwrap_or(10),
            offset: params.offset.unwrap_or(0),
            language: params.language,
            safe_search: params.safe_search.unwrap_or_default(),
            region: params.region,
        };

        let response = self.service.search(search_params).await?;

        let content = serde_json::to_string(&response).map_err(ServerError::JsonSerialize)?;

        Ok(CallToolResponse::Complete(CallToolResult::success(vec![
            ContentBlock::text(content),
        ])))
    }

    /// Handle a search request and return the raw response
    ///
    /// This is useful when you need direct access to the SearchResponse object.
    pub async fn search(&self, params: SearchParams) -> ServerResult<SearchResponse> {
        info!(
            "Processing search: query='{}', limit={}, offset={}",
            params.query, params.limit, params.offset
        );
        self.service.search(params).await
    }

    /// Get the search service reference
    pub fn service(&self) -> &Arc<SearchService> {
        &self.service
    }
}

/// Parameters for search requests
///
/// These parameters are deserialized from the MCP tool call arguments.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub safe_search: Option<SafeSearch>,

    /// Region code for localized results (e.g., "us-en", "uk-en")
    #[serde(default)]
    pub region: Option<String>,
}

impl SearchRequestParams {
    /// Validate the search parameters
    pub fn validate(&self) -> ServerResult<()> {
        if self.query.trim().is_empty() {
            return Err(ServerError::InvalidRequest(
                "Search query cannot be empty".to_string(),
            ));
        }

        if let Some(limit) = self.limit {
            if limit == 0 {
                return Err(ServerError::InvalidRequest(
                    "Limit must be greater than 0".to_string(),
                ));
            }
            if limit > 100 {
                return Err(ServerError::InvalidRequest(
                    "Limit cannot exceed 100".to_string(),
                ));
            }
        }

        Ok(())
    }
}

impl Default for SearchRequestParams {
    fn default() -> Self {
        Self {
            query: String::new(),
            limit: Some(10),
            offset: Some(0),
            language: None,
            safe_search: Some(SafeSearch::Moderate),
            region: None,
        }
    }
}

/// Tool definition for the web search tool
///
/// This defines the MCP tool schema for the web_search tool.
pub fn search_tool_definition() -> rmcp::model::Tool {
    let schema_json = serde_json::json!({
        "type": "object",
        "properties": {
            "query": {
                "type": "string",
                "description": "The search query string"
            },
            "limit": {
                "type": "integer",
                "description": "Maximum number of results to return",
                "minimum": 1,
                "maximum": 100,
                "default": 10
            },
            "offset": {
                "type": "integer",
                "description": "Offset for pagination (starting position)",
                "minimum": 0,
                "default": 0
            },
            "language": {
                "type": "string",
                "description": "Language code for results (e.g., 'en', 'es', 'fr', 'de', 'zh')"
            },
            "safe_search": {
                "type": "string",
                "enum": ["off", "moderate", "strict"],
                "description": "Safe search filter level",
                "default": "moderate"
            },
            "region": {
                "type": "string",
                "description": "Region code for localized results (e.g., 'us-en', 'uk-en', 'wt-wt')"
            }
        },
        "required": ["query"]
    });

    let schema_map = schema_json.as_object().cloned().unwrap_or_default();
    rmcp::model::Tool::new(
        "web_search",
        "Search the web for information",
        std::sync::Arc::new(schema_map),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_request_params_default() {
        let params = SearchRequestParams::default();
        assert_eq!(params.query, "");
        assert_eq!(params.limit, Some(10));
        assert_eq!(params.offset, Some(0));
    }

    #[test]
    fn test_search_request_params_validation_empty_query() {
        let params = SearchRequestParams {
            query: "   ".to_string(),
            ..Default::default()
        };
        assert!(params.validate().is_err());
    }

    #[test]
    fn test_search_request_params_validation_zero_limit() {
        let params = SearchRequestParams {
            query: "test".to_string(),
            limit: Some(0),
            ..Default::default()
        };
        assert!(params.validate().is_err());
    }

    #[test]
    fn test_search_request_params_validation_exceed_limit() {
        let params = SearchRequestParams {
            query: "test".to_string(),
            limit: Some(150),
            ..Default::default()
        };
        assert!(params.validate().is_err());
    }

    #[test]
    fn test_search_request_params_validation_valid() {
        let params = SearchRequestParams {
            query: "rust programming".to_string(),
            limit: Some(50),
            offset: Some(10),
            language: Some("en".to_string()),
            safe_search: Some(SafeSearch::Moderate),
            region: Some("us-en".to_string()),
        };
        assert!(params.validate().is_ok());
    }

    #[test]
    fn test_search_tool_definition() {
        let tool = search_tool_definition();
        assert_eq!(tool.name, "web_search");
        assert!(tool.description.is_some() && !tool.description.unwrap().is_empty());
    }
}
