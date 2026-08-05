// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Ole Reinartz
//!
//! Fetch handler for MCP protocol

use crate::error::{ServerError, ServerResult};
use crate::models::FetchParams;
use crate::services::FetchService;
use rmcp::model::CallToolRequestParams;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::debug;

/// Handler for fetch-related MCP requests
#[derive(Clone)]
pub struct FetchHandler {
    service: Arc<FetchService>,
}

impl FetchHandler {
    /// Create a new fetch handler
    pub fn new(service: Arc<FetchService>) -> Self {
        Self { service }
    }

    /// Handle a fetch request
    pub async fn handle_request(
        &self,
        request: &CallToolRequestParams,
    ) -> ServerResult<rmcp::model::CallToolResponse> {
        let params: FetchRequestParams = serde_json::from_value(serde_json::Value::Object(
            request.arguments.clone().unwrap_or_default(),
        ))
        .map_err(|e| ServerError::InvalidRequest(format!("Invalid parameters: {}", e)))?;

        debug!("Handling fetch request: url='{}'", params.url);

        let fetch_params = FetchParams {
            url: params.url,
            include_raw_html: params.include_raw_html.unwrap_or(false),
            extract_links: params.extract_links.unwrap_or(true),
            extract_images: params.extract_images.unwrap_or(false),
            timeout_secs: params.timeout_secs.unwrap_or(30),
            user_agent: params.user_agent,
            max_content_size: params.max_content_size.unwrap_or(10 * 1024 * 1024),
        };

        let content = self.service.fetch(fetch_params).await?;

        let response_text = serde_json::to_string(&content).map_err(ServerError::JsonSerialize)?;

        Ok(rmcp::model::CallToolResponse::Complete(
            rmcp::model::CallToolResult::success(vec![rmcp::model::ContentBlock::text(
                response_text,
            )]),
        ))
    }

    /// Get the fetch service reference
    pub fn service(&self) -> &Arc<FetchService> {
        &self.service
    }
}

/// Parameters for fetch requests
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Simplified fetch response
#[derive(Debug, Serialize, Deserialize)]
pub struct FetchResponse {
    pub url: String,
    pub content: String,
    pub fetched_at: String,
}

/// Resource content
#[derive(Debug, Clone, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct ResourceContent {
    pub uri: String,
    pub mimeType: String,
    pub text: String,
}

/// Tool definition for fetch
pub fn fetch_tool_definition() -> rmcp::model::Tool {
    let schema_json = serde_json::json!({
        "type": "object",
        "properties": {
            "url": {
                "type": "string",
                "description": "URL of the webpage to fetch"
            },
            "include_raw_html": {
                "type": "boolean",
                "description": "Include raw HTML in the response"
            },
            "extract_links": {
                "type": "boolean",
                "description": "Extract links from the page",
                "default": true
            },
            "extract_images": {
                "type": "boolean",
                "description": "Extract images from the page"
            },
            "timeout_secs": {
                "type": "integer",
                "description": "Request timeout in seconds",
                "default": 30
            },
            "user_agent": {
                "type": "string",
                "description": "Custom user agent string"
            },
            "max_content_size": {
                "type": "integer",
                "description": "Maximum content size in bytes",
                "default": 10485760
            }
        },
        "required": ["url"]
    });

    let schema_map = schema_json.as_object().cloned().unwrap_or_default();
    rmcp::model::Tool::new(
        "web_fetch",
        "Fetch and parse a webpage",
        std::sync::Arc::new(schema_map),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::model::CallToolRequestParams;

    fn make_fetch_request(
        name: String,
        args: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> CallToolRequestParams {
        match args {
            Some(m) => CallToolRequestParams::new(name).with_arguments(m),
            None => CallToolRequestParams::new(name),
        }
    }

    #[test]
    fn test_fetch_handler_new() {
        let service = FetchService::new(30).unwrap();
        let handler = FetchHandler::new(Arc::new(service));
        let svc = handler.service();
        assert!(Arc::strong_count(svc) >= 1);
    }

    #[test]
    fn test_fetch_handler_service() {
        let service = FetchService::new(30).unwrap();
        let handler = FetchHandler::new(Arc::new(service));
        let retrieved = handler.service();
        assert!(Arc::strong_count(retrieved) >= 1);
    }

    #[test]
    fn test_fetch_request_params_default() {
        let params = FetchRequestParams {
            url: "https://example.com".to_string(),
            include_raw_html: None,
            extract_links: None,
            extract_images: None,
            timeout_secs: None,
            user_agent: None,
            max_content_size: None,
        };

        assert_eq!(params.url, "https://example.com");
        assert_eq!(params.include_raw_html, None);
        assert_eq!(params.extract_links, None);
        assert_eq!(params.extract_images, None);
        assert_eq!(params.timeout_secs, None);
        assert_eq!(params.user_agent, None);
        assert_eq!(params.max_content_size, None);
    }

    #[test]
    fn test_fetch_request_params_deserialize_valid() {
        let json = serde_json::json!({
            "url": "https://example.com",
            "include_raw_html": true,
            "extract_links": false,
            "extract_images": true,
            "timeout_secs": 60,
            "user_agent": "CustomAgent/1.0",
            "max_content_size": 5242880
        });

        let params: FetchRequestParams = serde_json::from_value(json).unwrap();

        assert_eq!(params.url, "https://example.com");
        assert_eq!(params.include_raw_html, Some(true));
        assert_eq!(params.extract_links, Some(false));
        assert_eq!(params.extract_images, Some(true));
        assert_eq!(params.timeout_secs, Some(60));
        assert_eq!(params.user_agent, Some("CustomAgent/1.0".to_string()));
        assert_eq!(params.max_content_size, Some(5242880));
    }

    #[test]
    fn test_fetch_request_params_deserialize_minimal() {
        let json = serde_json::json!({
            "url": "https://example.com"
        });

        let params: FetchRequestParams = serde_json::from_value(json).unwrap();

        assert_eq!(params.url, "https://example.com");
        assert_eq!(params.include_raw_html, None);
        assert_eq!(params.extract_links, None);
        assert_eq!(params.extract_images, None);
        assert_eq!(params.timeout_secs, None);
        assert_eq!(params.user_agent, None);
        assert_eq!(params.max_content_size, None);
    }

    #[test]
    fn test_fetch_request_params_deserialize_missing_url() {
        let json = serde_json::json!({
            "include_raw_html": true
        });

        let result: Result<FetchRequestParams, _> = serde_json::from_value(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_fetch_request_params_deserialize_empty_object() {
        let json = serde_json::json!({});
        let result: Result<FetchRequestParams, _> = serde_json::from_value(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_fetch_response_serialize() {
        let response = FetchResponse {
            url: "https://example.com".to_string(),
            content: "Hello World".to_string(),
            fetched_at: "2025-01-01T00:00:00+00:00".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"url\":\"https://example.com\""));
        assert!(json.contains("\"content\":\"Hello World\""));
        assert!(json.contains("\"fetched_at\":\"2025-01-01T00:00:00+00:00\""));
    }

    #[test]
    fn test_fetch_response_deserialize() {
        let json = serde_json::json!({
            "url": "https://example.com",
            "content": "Hello World",
            "fetched_at": "2025-01-01T00:00:00+00:00"
        });

        let response: FetchResponse = serde_json::from_value(json).unwrap();
        assert_eq!(response.url, "https://example.com");
        assert_eq!(response.content, "Hello World");
        assert_eq!(response.fetched_at, "2025-01-01T00:00:00+00:00");
    }

    #[test]
    fn test_call_tool_response_content_block_any_text() {
        let json = serde_json::json!({
            "type": "text",
            "text": "Hello World"
        });

        let block: rmcp::model::ContentBlock = serde_json::from_value(json).unwrap();
        if let rmcp::model::ContentBlock::Text(text) = block {
            assert_eq!(text.text, "Hello World");
        } else {
            panic!("Expected Text variant");
        }
    }

    #[test]
    fn test_call_tool_response_content_block_any_resource() {
        let json = serde_json::json!({
            "type": "resource",
            "resource": {
                "uri": "https://example.com/page.html",
                "mimeType": "text/html",
                "text": "<html><body>Hello</body></html>"
            }
        });

        let block: rmcp::model::ContentBlock = serde_json::from_value(json).unwrap();
        if let rmcp::model::ContentBlock::Resource(embedded) = block {
            if let rmcp::model::ResourceContents::TextResourceContents {
                uri,
                mime_type,
                text,
                ..
            } = &embedded.resource
            {
                assert_eq!(uri, "https://example.com/page.html");
                assert_eq!(mime_type.as_deref(), Some("text/html"));
                assert_eq!(text, "<html><body>Hello</body></html>");
            } else {
                panic!("Expected TextResourceContents");
            }
        } else {
            panic!("Expected Resource variant");
        }
    }

    #[test]
    fn test_resource_content_serialize() {
        let rc = ResourceContent {
            uri: "https://example.com/data.json".to_string(),
            mimeType: "application/json".to_string(),
            text: "{\"key\":\"value\"}".to_string(),
        };

        let json = serde_json::to_string(&rc).unwrap();
        assert!(json.contains("\"uri\":\"https://example.com/data.json\""));
        assert!(json.contains("\"mimeType\":\"application/json\""));
        assert!(json.contains("\"text\":\"{\\\"key\\\":\\\"value\\\"}\""));
    }

    #[test]
    fn test_resource_content_deserialize() {
        let json = serde_json::json!({
            "uri": "https://example.com/file.txt",
            "mimeType": "text/plain",
            "text": "Plain text content"
        });

        let rc: ResourceContent = serde_json::from_value(json).unwrap();
        assert_eq!(rc.uri, "https://example.com/file.txt");
        assert_eq!(rc.mimeType, "text/plain");
        assert_eq!(rc.text, "Plain text content");
    }

    #[test]
    fn test_resource_content_various_mime_types() {
        let mime_types = vec![
            "text/html",
            "application/json",
            "text/plain",
            "image/png",
            "application/xml",
            "text/css",
            "application/javascript",
            "text/csv",
        ];

        for mime_type in mime_types {
            let json = serde_json::json!({
                "uri": "https://example.com/test",
                "mimeType": mime_type,
                "text": "content"
            });

            let rc: ResourceContent = serde_json::from_value(json).unwrap();
            assert_eq!(rc.mimeType, mime_type);
        }
    }

    #[test]
    fn test_fetch_request_params_partial_fields() {
        let json = serde_json::json!({
            "url": "https://example.com",
            "timeout_secs": 10,
            "extract_images": true
        });

        let params: FetchRequestParams = serde_json::from_value(json).unwrap();
        assert_eq!(params.url, "https://example.com");
        assert_eq!(params.timeout_secs, Some(10));
        assert_eq!(params.extract_images, Some(true));
        assert_eq!(params.include_raw_html, None);
        assert_eq!(params.extract_links, None);
        assert_eq!(params.user_agent, None);
        assert_eq!(params.max_content_size, None);
    }

    #[test]
    fn test_fetch_request_params_timeout_zero() {
        let json = serde_json::json!({
            "url": "https://example.com",
            "timeout_secs": 0
        });

        let params: FetchRequestParams = serde_json::from_value(json).unwrap();
        assert_eq!(params.url, "https://example.com");
        assert_eq!(params.timeout_secs, Some(0));
    }

    #[test]
    fn test_fetch_request_params_deserialize_invalid_type() {
        let json = serde_json::json!({
            "url": 12345
        });

        let result: Result<FetchRequestParams, _> = serde_json::from_value(json);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_fetch_handler_handle_request_valid_params() {
        let service = FetchService::new(30).unwrap();
        let handler = FetchHandler::new(Arc::new(service));

        let args: serde_json::Map<String, serde_json::Value> =
            serde_json::from_value(serde_json::json!({
                "url": "https://example.com",
                "timeout_secs": 5,
                "include_raw_html": true
            }))
            .unwrap();

        let request = make_fetch_request("web_fetch".to_string(), Some(args));

        let result = handler.handle_request(&request).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_fetch_handler_handle_request_missing_url() {
        let service = FetchService::new(30).unwrap();
        let handler = FetchHandler::new(Arc::new(service));

        let args: serde_json::Map<String, serde_json::Value> =
            serde_json::from_value(serde_json::json!({
                "timeout_secs": 5
            }))
            .unwrap();

        let request = make_fetch_request("web_fetch".to_string(), Some(args));

        let result = futures::executor::block_on(handler.handle_request(&request));
        assert!(result.is_err());

        match result {
            Err(ServerError::InvalidRequest(msg)) => {
                assert!(msg.contains("Invalid parameters"));
            }
            _ => panic!("Expected InvalidRequest error, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_fetch_handler_handle_request_invalid_json_args() {
        let service = FetchService::new(30).unwrap();
        let handler = FetchHandler::new(Arc::new(service));

        let args: serde_json::Map<String, serde_json::Value> =
            serde_json::from_value(serde_json::json!({
                "url": "not a valid url for parsing but we want json parse error first",
                "timeout_secs": "not_a_number"
            }))
            .unwrap();

        let request = make_fetch_request("web_fetch".to_string(), Some(args));

        let result = handler.handle_request(&request).await;
        assert!(result.is_err());

        match result {
            Err(ServerError::InvalidRequest(msg)) => {
                assert!(msg.contains("Invalid parameters"));
            }
            _ => panic!("Expected InvalidRequest error, got {:?}", result),
        }
    }

    #[test]
    fn test_fetch_handler_handle_request_empty_arguments() {
        let service = FetchService::new(30).unwrap();
        let handler = FetchHandler::new(Arc::new(service));

        let args: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
        let request = make_fetch_request("web_fetch".to_string(), Some(args));

        let result = futures::executor::block_on(handler.handle_request(&request));
        assert!(result.is_err());

        match result {
            Err(ServerError::InvalidRequest(msg)) => {
                assert!(msg.contains("Invalid parameters"));
            }
            _ => panic!("Expected InvalidRequest error, got {:?}", result),
        }
    }

    #[test]
    fn test_fetch_handler_handle_request_null_arguments() {
        let service = FetchService::new(30).unwrap();
        let handler = FetchHandler::new(Arc::new(service));

        let request = make_fetch_request("web_fetch".to_string(), None);

        let result = futures::executor::block_on(handler.handle_request(&request));
        assert!(result.is_err());

        match result {
            Err(ServerError::InvalidRequest(msg)) => {
                assert!(msg.contains("Invalid parameters"));
            }
            _ => panic!("Expected InvalidRequest error, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_fetch_handler_handle_request_with_all_options() {
        let service = FetchService::new(30).unwrap();
        let handler = FetchHandler::new(Arc::new(service));

        let args: serde_json::Map<String, serde_json::Value> =
            serde_json::from_value(serde_json::json!({
                "url": "https://example.com",
                "include_raw_html": true,
                "extract_links": true,
                "extract_images": true,
                "timeout_secs": 15,
                "user_agent": "TestBot/2.0",
                "max_content_size": 1048576
            }))
            .unwrap();

        let request = make_fetch_request("web_fetch".to_string(), Some(args));

        let result = handler.handle_request(&request).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_fetch_tool_definition_name() {
        let tool = fetch_tool_definition();
        assert_eq!(tool.name, "web_fetch");
    }

    #[test]
    fn test_fetch_tool_definition_description() {
        let tool = fetch_tool_definition();
        let desc = tool.description.unwrap_or_default();
        assert!(!desc.is_empty());
    }

    #[test]
    fn test_fetch_tool_definition_schema_properties() {
        let tool = fetch_tool_definition();
        let schema: &serde_json::Map<String, serde_json::Value> = &tool.input_schema;
        let props = schema.get("properties").unwrap().as_object().unwrap();

        assert!(props.contains_key("url"));
        assert!(props.contains_key("include_raw_html"));
        assert!(props.contains_key("extract_links"));
        assert!(props.contains_key("extract_images"));
        assert!(props.contains_key("timeout_secs"));
        assert!(props.contains_key("user_agent"));
        assert!(props.contains_key("max_content_size"));
    }

    #[test]
    fn test_fetch_tool_definition_required_fields() {
        let tool = fetch_tool_definition();
        let schema: &serde_json::Map<String, serde_json::Value> = &tool.input_schema;
        let props = schema.get("properties").unwrap().as_object().unwrap();

        // Check that url has a description
        let url_prop = props.get("url").unwrap();
        assert!(url_prop.get("type").is_some());
        assert!(url_prop.get("description").is_some());
    }

    #[test]
    fn test_fetch_tool_definition_schema_type() {
        let tool = fetch_tool_definition();
        let schema: &serde_json::Map<String, serde_json::Value> = &tool.input_schema;

        assert_eq!(schema.get("type").unwrap(), "object");
        assert!(schema.contains_key("properties"));
    }

    #[test]
    fn test_fetch_tool_definition_schema_defaults() {
        let tool = fetch_tool_definition();
        let schema: &serde_json::Map<String, serde_json::Value> = &tool.input_schema;

        let props = schema.get("properties").unwrap().as_object().unwrap();
        let timeout_prop = props.get("timeout_secs").unwrap();
        assert_eq!(timeout_prop.get("default").unwrap(), 30);

        let max_size_prop = props.get("max_content_size").unwrap();
        assert_eq!(max_size_prop.get("default").unwrap(), 10485760);

        let extract_links_prop = props.get("extract_links").unwrap();
        assert_eq!(extract_links_prop.get("default").unwrap(), true);
    }

    #[test]
    fn test_fetch_handler_clone() {
        let service = FetchService::new(30).unwrap();
        let handler1 = FetchHandler::new(Arc::new(service));
        let handler2 = handler1.clone();
        assert!(Arc::strong_count(handler2.service()) >= 2);
    }

    #[test]
    fn test_fetch_response_fetched_at_format() {
        use chrono::Utc;
        let response = FetchResponse {
            url: "https://example.com".to_string(),
            content: "content".to_string(),
            fetched_at: Utc::now().to_rfc3339(),
        };

        let json = serde_json::to_string(&response).unwrap();
        // RFC3339 format should contain 'T' separator
        assert!(json.contains('T'));
    }
}
