// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Ole Reinartz
//!
//! Search-related data models

use serde::{Deserialize, Serialize};

/// A single search result from a web search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Title of the search result
    pub title: String,

    /// URL of the search result
    pub url: String,

    /// Brief snippet/description of the result
    pub snippet: String,

    /// Display URL (may be different from actual URL)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_url: Option<String>,

    /// Position in search results (1-indexed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<u32>,

    /// Source/engine that provided this result
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,

    /// Publication date if available
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
}

/// Complete search response containing multiple results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    /// List of search results
    pub results: Vec<SearchResult>,

    /// Total number of results found
    pub total_results: u64,

    /// Number of results returned
    pub count: usize,

    /// Search query that was executed
    pub query: String,

    /// Offset/starting position
    pub offset: u32,

    /// Time taken for search in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_time_ms: Option<u64>,
}

/// Parameters for a web search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchParams {
    /// The search query string
    pub query: String,

    /// Maximum number of results to return (default: 10)
    #[serde(default = "default_limit")]
    pub limit: u32,

    /// Offset for pagination
    #[serde(default)]
    pub offset: u32,

    /// Language code (e.g., "en", "es", "fr")
    #[serde(default)]
    pub language: Option<String>,

    /// Safe search level
    #[serde(default)]
    pub safe_search: SafeSearch,

    /// Search region/code for localized results
    #[serde(default)]
    pub region: Option<String>,
}

impl Default for SearchParams {
    fn default() -> Self {
        Self {
            query: String::new(),
            limit: 10,
            offset: 0,
            language: None,
            safe_search: SafeSearch::default(),
            region: None,
        }
    }
}

fn default_limit() -> u32 {
    10
}

/// Safe search filter levels
#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, schemars::JsonSchema, PartialEq, Eq, Default,
)]
#[serde(rename_all = "lowercase")]
pub enum SafeSearch {
    /// No filtering
    Off,
    /// Moderate filtering (default)
    #[default]
    Moderate,
    /// Strict filtering
    Strict,
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- default_limit helper ---

    #[test]
    fn test_default_limit() {
        assert_eq!(default_limit(), 10);
    }

    // --- SearchResult tests ---

    #[test]
    fn test_search_result_defaults() {
        let result = SearchResult {
            title: "Test Title".to_string(),
            url: "https://example.com".to_string(),
            snippet: "Test snippet".to_string(),
            display_url: None,
            position: None,
            source: None,
            date: None,
        };
        assert_eq!(result.title, "Test Title");
        assert_eq!(result.url, "https://example.com");
        assert_eq!(result.snippet, "Test snippet");
        assert!(result.display_url.is_none());
        assert!(result.position.is_none());
        assert!(result.source.is_none());
        assert!(result.date.is_none());
    }

    #[test]
    fn test_search_result_full() {
        let result = SearchResult {
            title: "Full Title".to_string(),
            url: "https://full.example.com/page".to_string(),
            snippet: "Full snippet text".to_string(),
            display_url: Some("full.example.com".to_string()),
            position: Some(1),
            source: Some("test".to_string()),
            date: Some("2024-01-15".to_string()),
        };
        assert_eq!(result.display_url, Some("full.example.com".to_string()));
        assert_eq!(result.position, Some(1));
        assert_eq!(result.source, Some("test".to_string()));
        assert_eq!(result.date, Some("2024-01-15".to_string()));
    }

    #[test]
    fn test_search_result_clone() {
        let result = SearchResult {
            title: "Clone Test".to_string(),
            url: "https://example.com".to_string(),
            snippet: "Snippet".to_string(),
            display_url: None,
            position: None,
            source: None,
            date: None,
        };
        let cloned = result.clone();
        assert_eq!(result.title, cloned.title);
        assert_eq!(result.url, cloned.url);
        assert_eq!(result.snippet, cloned.snippet);
        assert_eq!(result.display_url, cloned.display_url);
        assert_eq!(result.position, cloned.position);
    }

    #[test]
    fn test_search_result_serde_json_full() {
        let result = SearchResult {
            title: "Full Result".to_string(),
            url: "https://example.com/result".to_string(),
            snippet: "Full result snippet".to_string(),
            display_url: Some("example.com".to_string()),
            position: Some(3),
            source: Some("duckduckgo".to_string()),
            date: Some("2024-06-01".to_string()),
        };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: SearchResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result.title, deserialized.title);
        assert_eq!(result.url, deserialized.url);
        assert_eq!(result.snippet, deserialized.snippet);
        assert_eq!(result.display_url, deserialized.display_url);
        assert_eq!(result.position, deserialized.position);
        assert_eq!(result.source, deserialized.source);
        assert_eq!(result.date, deserialized.date);
    }

    #[test]
    fn test_search_result_serde_json_minimal() {
        let result = SearchResult {
            title: "Minimal".to_string(),
            url: "https://minimal.example.com".to_string(),
            snippet: "Minimal snippet".to_string(),
            display_url: None,
            position: None,
            source: None,
            date: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(!json.contains("display_url"));
        assert!(!json.contains("position"));
        assert!(!json.contains("source"));
        assert!(!json.contains("date"));
        let deserialized: SearchResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result.title, deserialized.title);
        assert_eq!(result.url, deserialized.url);
        assert_eq!(result.snippet, deserialized.snippet);
        assert!(deserialized.display_url.is_none());
        assert!(deserialized.position.is_none());
    }

    #[test]
    fn test_search_result_deserialize_json() {
        let json = r#"{
            "title": "Parsed Title",
            "url": "https://parsed.example.com",
            "snippet": "Parsed snippet",
            "display_url": "parsed.example.com",
            "position": 5,
            "source": "brightdata",
            "date": "2024-12-25"
        }"#;
        let result: SearchResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.title, "Parsed Title");
        assert_eq!(result.url, "https://parsed.example.com");
        assert_eq!(result.snippet, "Parsed snippet");
        assert_eq!(result.display_url, Some("parsed.example.com".to_string()));
        assert_eq!(result.position, Some(5));
        assert_eq!(result.source, Some("brightdata".to_string()));
        assert_eq!(result.date, Some("2024-12-25".to_string()));
    }

    #[test]
    fn test_search_result_serialize_roundtrip() {
        let result = SearchResult {
            title: "Roundtrip".to_string(),
            url: "https://roundtrip.example.com".to_string(),
            snippet: "Roundtrip snippet".to_string(),
            display_url: Some("roundtrip.example.com".to_string()),
            position: Some(2),
            source: Some("serper".to_string()),
            date: Some("2024-03-20".to_string()),
        };
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: SearchResult = serde_json::from_str(&json).unwrap();
        assert_eq!(result.title, deserialized.title);
        assert_eq!(result.url, deserialized.url);
        assert_eq!(result.snippet, deserialized.snippet);
        assert_eq!(result.display_url, deserialized.display_url);
        assert_eq!(result.position, deserialized.position);
        assert_eq!(result.source, deserialized.source);
        assert_eq!(result.date, deserialized.date);
    }

    // --- SearchResponse tests ---

    #[test]
    fn test_search_response_empty() {
        let response = SearchResponse {
            results: vec![],
            total_results: 0,
            count: 0,
            query: "".to_string(),
            offset: 0,
            search_time_ms: None,
        };
        assert_eq!(response.results.len(), 0);
        assert_eq!(response.total_results, 0);
        assert_eq!(response.count, 0);
        assert_eq!(response.query, "");
        assert_eq!(response.offset, 0);
        assert!(response.search_time_ms.is_none());
    }

    #[test]
    fn test_search_response_with_results() {
        let results = vec![
            SearchResult {
                title: "Result 1".to_string(),
                url: "https://example.com/1".to_string(),
                snippet: "Snippet 1".to_string(),
                display_url: None,
                position: Some(1),
                source: None,
                date: None,
            },
            SearchResult {
                title: "Result 2".to_string(),
                url: "https://example.com/2".to_string(),
                snippet: "Snippet 2".to_string(),
                display_url: None,
                position: Some(2),
                source: None,
                date: None,
            },
        ];
        let response = SearchResponse {
            results,
            total_results: 150,
            count: 2,
            query: "test query".to_string(),
            offset: 0,
            search_time_ms: Some(125),
        };
        assert_eq!(response.results.len(), 2);
        assert_eq!(response.total_results, 150);
        assert_eq!(response.count, 2);
        assert_eq!(response.query, "test query");
        assert_eq!(response.offset, 0);
        assert_eq!(response.search_time_ms, Some(125));
    }

    #[test]
    fn test_search_response_clone() {
        let response = SearchResponse {
            results: vec![SearchResult {
                title: "Cloned".to_string(),
                url: "https://cloned.example.com".to_string(),
                snippet: "Cloned snippet".to_string(),
                display_url: None,
                position: None,
                source: None,
                date: None,
            }],
            total_results: 10,
            count: 1,
            query: "clone test".to_string(),
            offset: 5,
            search_time_ms: Some(50),
        };
        let cloned = response.clone();
        assert_eq!(response.results.len(), cloned.results.len());
        assert_eq!(response.total_results, cloned.total_results);
        assert_eq!(response.count, cloned.count);
        assert_eq!(response.query, cloned.query);
        assert_eq!(response.offset, cloned.offset);
        assert_eq!(response.search_time_ms, cloned.search_time_ms);
    }

    #[test]
    fn test_search_response_serde_json() {
        let response = SearchResponse {
            results: vec![SearchResult {
                title: "Response Result".to_string(),
                url: "https://response.example.com".to_string(),
                snippet: "Response snippet".to_string(),
                display_url: Some("response.example.com".to_string()),
                position: Some(1),
                source: Some("test".to_string()),
                date: Some("2024-01-01".to_string()),
            }],
            total_results: 999,
            count: 1,
            query: "search response".to_string(),
            offset: 0,
            search_time_ms: Some(200),
        };
        let json = serde_json::to_string(&response).unwrap();
        let deserialized: SearchResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(response.results.len(), deserialized.results.len());
        assert_eq!(response.total_results, deserialized.total_results);
        assert_eq!(response.count, deserialized.count);
        assert_eq!(response.query, deserialized.query);
        assert_eq!(response.offset, deserialized.offset);
        assert_eq!(response.search_time_ms, deserialized.search_time_ms);
        assert_eq!(response.results[0].title, deserialized.results[0].title);
    }

    #[test]
    fn test_search_response_deserialize_with_time() {
        let json = r#"{
            "results": [
                {
                    "title": "Deserialized Result",
                    "url": "https://deserialized.example.com",
                    "snippet": "Deserialized snippet"
                }
            ],
            "total_results": 42,
            "count": 1,
            "query": "deserialize test",
            "offset": 0,
            "search_time_ms": 75
        }"#;
        let response: SearchResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.total_results, 42);
        assert_eq!(response.count, 1);
        assert_eq!(response.query, "deserialize test");
        assert_eq!(response.offset, 0);
        assert_eq!(response.search_time_ms, Some(75));
    }

    #[test]
    fn test_search_response_deserialize_without_time() {
        let json = r#"{
            "results": [],
            "total_results": 0,
            "count": 0,
            "query": "no time test",
            "offset": 0
        }"#;
        let response: SearchResponse = serde_json::from_str(json).unwrap();
        assert!(response.search_time_ms.is_none());
        assert_eq!(response.query, "no time test");
    }

    // --- SearchParams tests ---

    #[test]
    fn test_search_params_default() {
        let params = SearchParams::default();
        assert_eq!(params.query, "");
        assert_eq!(params.limit, 10);
        assert_eq!(params.offset, 0);
        assert!(params.language.is_none());
        assert_eq!(params.safe_search, SafeSearch::Moderate);
        assert!(params.region.is_none());
    }

    #[test]
    fn test_search_params_all_fields() {
        let params = SearchParams {
            query: "all fields test".to_string(),
            limit: 25,
            offset: 10,
            language: Some("es".to_string()),
            safe_search: SafeSearch::Strict,
            region: Some("US".to_string()),
        };
        assert_eq!(params.query, "all fields test");
        assert_eq!(params.limit, 25);
        assert_eq!(params.offset, 10);
        assert_eq!(params.language, Some("es".to_string()));
        assert_eq!(params.safe_search, SafeSearch::Strict);
        assert_eq!(params.region, Some("US".to_string()));
    }

    #[test]
    fn test_search_params_clone() {
        let params = SearchParams {
            query: "clone params".to_string(),
            limit: 5,
            offset: 2,
            language: Some("fr".to_string()),
            safe_search: SafeSearch::Off,
            region: Some("FR".to_string()),
        };
        let cloned = params.clone();
        assert_eq!(params.query, cloned.query);
        assert_eq!(params.limit, cloned.limit);
        assert_eq!(params.offset, cloned.offset);
        assert_eq!(params.language, cloned.language);
        assert_eq!(params.safe_search, cloned.safe_search);
        assert_eq!(params.region, cloned.region);
    }

    #[test]
    fn test_search_params_deserialize_json_defaults() {
        let json = r#"{"query": "just query"}"#;
        let params: SearchParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.query, "just query");
        assert_eq!(params.limit, 10);
        assert_eq!(params.offset, 0);
        assert!(params.language.is_none());
        assert_eq!(params.safe_search, SafeSearch::Moderate);
        assert!(params.region.is_none());
    }

    #[test]
    fn test_search_params_deserialize_json_full() {
        let json = r#"{
            "query": "full params",
            "limit": 15,
            "offset": 5,
            "language": "de",
            "safe_search": "strict",
            "region": "DE"
        }"#;
        let params: SearchParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.query, "full params");
        assert_eq!(params.limit, 15);
        assert_eq!(params.offset, 5);
        assert_eq!(params.language, Some("de".to_string()));
        assert_eq!(params.safe_search, SafeSearch::Strict);
        assert_eq!(params.region, Some("DE".to_string()));
    }

    #[test]
    fn test_search_params_serde_roundtrip() {
        let params = SearchParams {
            query: "roundtrip params".to_string(),
            limit: 20,
            offset: 3,
            language: Some("ja".to_string()),
            safe_search: SafeSearch::Off,
            region: Some("JP".to_string()),
        };
        let json = serde_json::to_string(&params).unwrap();
        let deserialized: SearchParams = serde_json::from_str(&json).unwrap();
        assert_eq!(params.query, deserialized.query);
        assert_eq!(params.limit, deserialized.limit);
        assert_eq!(params.offset, deserialized.offset);
        assert_eq!(params.language, deserialized.language);
        assert_eq!(params.safe_search, deserialized.safe_search);
        assert_eq!(params.region, deserialized.region);
    }

    // --- SafeSearch tests ---

    #[test]
    fn test_safe_search_default() {
        assert_eq!(SafeSearch::default(), SafeSearch::Moderate);
    }

    #[test]
    fn test_safe_search_clone_copy() {
        let ss = SafeSearch::Strict;
        let cloned = ss;
        assert_eq!(cloned, SafeSearch::Strict);
    }

    #[test]
    fn test_safe_search_eq() {
        assert_eq!(SafeSearch::Off, SafeSearch::Off);
        assert_eq!(SafeSearch::Moderate, SafeSearch::Moderate);
        assert_eq!(SafeSearch::Strict, SafeSearch::Strict);
        assert_ne!(SafeSearch::Off, SafeSearch::Moderate);
        assert_ne!(SafeSearch::Moderate, SafeSearch::Strict);
    }

    #[test]
    fn test_safe_search_serialize_json() {
        assert_eq!(serde_json::to_string(&SafeSearch::Off).unwrap(), "\"off\"");
        assert_eq!(
            serde_json::to_string(&SafeSearch::Moderate).unwrap(),
            "\"moderate\""
        );
        assert_eq!(
            serde_json::to_string(&SafeSearch::Strict).unwrap(),
            "\"strict\""
        );
    }

    #[test]
    fn test_safe_search_deserialize_json() {
        let off: SafeSearch = serde_json::from_str("\"off\"").unwrap();
        assert_eq!(off, SafeSearch::Off);
        let moderate: SafeSearch = serde_json::from_str("\"moderate\"").unwrap();
        assert_eq!(moderate, SafeSearch::Moderate);
        let strict: SafeSearch = serde_json::from_str("\"strict\"").unwrap();
        assert_eq!(strict, SafeSearch::Strict);
    }

    #[test]
    fn test_safe_search_debug_display() {
        let _ = format!("{:?}", SafeSearch::Off);
        let _ = format!("{:?}", SafeSearch::Moderate);
        let _ = format!("{:?}", SafeSearch::Strict);
    }
}
