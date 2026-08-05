// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Ole Reinartz
//!
//! Webpage content data models

use serde::{Deserialize, Serialize};

/// Represents the content extracted from a webpage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebpageContent {
    /// The original URL that was fetched
    pub url: String,

    /// Final URL after any redirects
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_url: Option<String>,

    /// HTTP status code
    pub status_code: u16,

    /// Page title (from <title> tag)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Meta description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Extracted text content (HTML stripped)
    pub content: String,

    /// Raw HTML (optional, for debugging)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_html: Option<String>,

    /// Links found on the page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<Link>>,

    /// Images found on the page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<ImageInfo>>,

    /// Response headers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<std::collections::HashMap<String, String>>,

    /// Content type (MIME type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,

    /// Character encoding
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,

    /// Fetch timestamp (ISO 8601)
    pub fetched_at: String,

    /// Whether the content was truncated due to size limits
    #[serde(default)]
    pub truncated: bool,
}

/// A link found on a webpage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    /// The URL the link points to
    pub url: String,

    /// Link text content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    /// Whether this is an internal link
    #[serde(default)]
    pub is_internal: bool,
}

/// Image information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageInfo {
    /// Image URL
    pub url: String,

    /// Alt text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alt: Option<String>,

    /// Image dimensions (width, height)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<(u32, u32)>,

    /// MIME type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// Parameters for fetching a webpage
#[derive(Debug, Clone, Deserialize)]
pub struct FetchParams {
    /// URL to fetch
    pub url: String,

    /// Include raw HTML in response
    #[serde(default)]
    pub include_raw_html: bool,

    /// Extract links from the page
    #[serde(default = "default_extract_links")]
    pub extract_links: bool,

    /// Extract images from the page
    #[serde(default)]
    pub extract_images: bool,

    /// Timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    /// User agent string
    #[serde(default)]
    pub user_agent: Option<String>,

    /// Maximum content size in bytes (default: 10MB)
    #[serde(default = "default_max_size")]
    pub max_content_size: usize,
}

fn default_extract_links() -> bool {
    true
}

fn default_timeout() -> u64 {
    30
}

fn default_max_size() -> usize {
    10 * 1024 * 1024 // 10MB
}

impl Default for FetchParams {
    fn default() -> Self {
        Self {
            url: String::new(),
            include_raw_html: false,
            extract_links: true,
            extract_images: false,
            timeout_secs: 30,
            user_agent: None,
            max_content_size: 10 * 1024 * 1024,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- default_* helper functions ---

    #[test]
    fn test_default_extract_links() {
        assert!(default_extract_links());
    }

    #[test]
    fn test_default_timeout() {
        assert_eq!(default_timeout(), 30);
    }

    #[test]
    fn test_default_max_size() {
        assert_eq!(default_max_size(), 10 * 1024 * 1024);
    }

    // --- WebpageContent tests ---

    #[test]
    fn test_webpage_content_all_fields() {
        let content = WebpageContent {
            url: "https://example.com".to_string(),
            final_url: Some("https://example.com/final".to_string()),
            status_code: 200,
            title: Some("Page Title".to_string()),
            description: Some("Meta description".to_string()),
            content: "Extracted text content".to_string(),
            raw_html: Some("<html><body>raw html</body></html>".to_string()),
            links: Some(vec![Link {
                url: "https://example.com/link".to_string(),
                text: Some("link text".to_string()),
                is_internal: true,
            }]),
            images: Some(vec![ImageInfo {
                url: "https://example.com/img.png".to_string(),
                alt: Some("alt text".to_string()),
                dimensions: Some((800, 600)),
                mime_type: Some("image/png".to_string()),
            }]),
            headers: Some({
                let mut map = std::collections::HashMap::new();
                map.insert("content-type".to_string(), "text/html".to_string());
                map
            }),
            content_type: Some("text/html".to_string()),
            encoding: Some("utf-8".to_string()),
            fetched_at: "2024-01-15T10:30:00Z".to_string(),
            truncated: false,
        };
        assert_eq!(content.url, "https://example.com");
        assert_eq!(
            content.final_url,
            Some("https://example.com/final".to_string())
        );
        assert_eq!(content.status_code, 200);
        assert_eq!(content.title, Some("Page Title".to_string()));
        assert_eq!(content.description, Some("Meta description".to_string()));
        assert_eq!(content.content, "Extracted text content");
        assert!(content.raw_html.is_some());
        assert!(content.links.is_some());
        assert!(content.images.is_some());
        assert!(content.headers.is_some());
        assert_eq!(content.content_type, Some("text/html".to_string()));
        assert_eq!(content.encoding, Some("utf-8".to_string()));
        assert_eq!(content.fetched_at, "2024-01-15T10:30:00Z");
        assert!(!content.truncated);
    }

    #[test]
    fn test_webpage_content_minimal() {
        let content = WebpageContent {
            url: "https://minimal.example.com".to_string(),
            final_url: None,
            status_code: 200,
            title: None,
            description: None,
            content: String::new(),
            raw_html: None,
            links: None,
            images: None,
            headers: None,
            content_type: None,
            encoding: None,
            fetched_at: "2024-06-01T00:00:00Z".to_string(),
            truncated: false,
        };
        assert!(content.final_url.is_none());
        assert!(content.title.is_none());
        assert!(content.description.is_none());
        assert!(content.raw_html.is_none());
        assert!(content.links.is_none());
        assert!(content.images.is_none());
        assert!(content.headers.is_none());
        assert!(content.content_type.is_none());
        assert!(content.encoding.is_none());
        assert_eq!(content.content, "");
    }

    #[test]
    fn test_webpage_content_truncated() {
        let content = WebpageContent {
            url: "https://large.example.com".to_string(),
            final_url: None,
            status_code: 200,
            title: Some("Large Page".to_string()),
            description: None,
            content: "x".repeat(100000),
            raw_html: None,
            links: None,
            images: None,
            headers: None,
            content_type: None,
            encoding: None,
            fetched_at: "2024-07-01T12:00:00Z".to_string(),
            truncated: true,
        };
        assert!(content.truncated);
        assert_eq!(content.content.len(), 100000);
    }

    #[test]
    fn test_webpage_content_clone() {
        let content = WebpageContent {
            url: "https://clone.example.com".to_string(),
            final_url: Some("https://clone.example.com/final".to_string()),
            status_code: 301,
            title: Some("Cloned".to_string()),
            description: Some("Cloned desc".to_string()),
            content: "Cloned content".to_string(),
            raw_html: None,
            links: None,
            images: None,
            headers: None,
            content_type: Some("text/html".to_string()),
            encoding: Some("utf-8".to_string()),
            fetched_at: "2024-01-01T00:00:00Z".to_string(),
            truncated: false,
        };
        let cloned = content.clone();
        assert_eq!(content.url, cloned.url);
        assert_eq!(content.final_url, cloned.final_url);
        assert_eq!(content.status_code, cloned.status_code);
        assert_eq!(content.title, cloned.title);
        assert_eq!(content.description, cloned.description);
        assert_eq!(content.content, cloned.content);
    }

    #[test]
    fn test_webpage_content_serde_json() {
        let content = WebpageContent {
            url: "https://serde.example.com".to_string(),
            final_url: None,
            status_code: 200,
            title: Some("Serde Page".to_string()),
            description: None,
            content: "Serde content".to_string(),
            raw_html: None,
            links: None,
            images: None,
            headers: None,
            content_type: Some("text/html".to_string()),
            encoding: None,
            fetched_at: "2024-03-15T08:00:00Z".to_string(),
            truncated: true,
        };
        let json = serde_json::to_string(&content).unwrap();
        let deserialized: WebpageContent = serde_json::from_str(&json).unwrap();
        assert_eq!(content.url, deserialized.url);
        assert_eq!(content.final_url, deserialized.final_url);
        assert_eq!(content.status_code, deserialized.status_code);
        assert_eq!(content.title, deserialized.title);
        assert_eq!(content.description, deserialized.description);
        assert_eq!(content.content, deserialized.content);
        assert_eq!(content.content_type, deserialized.content_type);
        assert_eq!(content.encoding, deserialized.encoding);
        assert_eq!(content.fetched_at, deserialized.fetched_at);
        assert_eq!(content.truncated, deserialized.truncated);
    }

    // --- Link tests ---

    #[test]
    fn test_link_with_text() {
        let link = Link {
            url: "https://example.com/link".to_string(),
            text: Some("Click here".to_string()),
            is_internal: true,
        };
        assert_eq!(link.url, "https://example.com/link");
        assert_eq!(link.text, Some("Click here".to_string()));
        assert!(link.is_internal);
    }

    #[test]
    fn test_link_without_text() {
        let link = Link {
            url: "https://example.com/nolink".to_string(),
            text: None,
            is_internal: false,
        };
        assert_eq!(link.url, "https://example.com/nolink");
        assert!(link.text.is_none());
        assert!(!link.is_internal);
    }

    #[test]
    fn test_link_clone() {
        let link = Link {
            url: "https://clone-link.example.com".to_string(),
            text: Some("Clone link".to_string()),
            is_internal: true,
        };
        let cloned = link.clone();
        assert_eq!(link.url, cloned.url);
        assert_eq!(link.text, cloned.text);
        assert_eq!(link.is_internal, cloned.is_internal);
    }

    #[test]
    fn test_link_serde_json_with_text() {
        let link = Link {
            url: "https://with-text.example.com".to_string(),
            text: Some("With Text".to_string()),
            is_internal: true,
        };
        let json = serde_json::to_string(&link).unwrap();
        let deserialized: Link = serde_json::from_str(&json).unwrap();
        assert_eq!(link.url, deserialized.url);
        assert_eq!(link.text, deserialized.text);
        assert_eq!(link.is_internal, deserialized.is_internal);
    }

    #[test]
    fn test_link_serde_json_no_text() {
        let link = Link {
            url: "https://skip-test.example.com".to_string(),
            text: None,
            is_internal: false,
        };
        let json = serde_json::to_string(&link).unwrap();
        assert!(!json.contains("text"));
        let deserialized: Link = serde_json::from_str(&json).unwrap();
        assert_eq!(link.url, deserialized.url);
        assert!(deserialized.text.is_none());
        assert!(!deserialized.is_internal);
    }

    // --- ImageInfo tests ---

    #[test]
    fn test_image_info_full() {
        let image = ImageInfo {
            url: "https://example.com/image.png".to_string(),
            alt: Some("An image".to_string()),
            dimensions: Some((1920, 1080)),
            mime_type: Some("image/png".to_string()),
        };
        assert_eq!(image.url, "https://example.com/image.png");
        assert_eq!(image.alt, Some("An image".to_string()));
        assert_eq!(image.dimensions, Some((1920, 1080)));
        assert_eq!(image.mime_type, Some("image/png".to_string()));
    }

    #[test]
    fn test_image_info_minimal() {
        let image = ImageInfo {
            url: "https://example.com/mini.jpg".to_string(),
            alt: None,
            dimensions: None,
            mime_type: None,
        };
        assert_eq!(image.url, "https://example.com/mini.jpg");
        assert!(image.alt.is_none());
        assert!(image.dimensions.is_none());
        assert!(image.mime_type.is_none());
    }

    #[test]
    fn test_image_info_clone() {
        let image = ImageInfo {
            url: "https://clone-image.example.com".to_string(),
            alt: Some("Cloned".to_string()),
            dimensions: Some((100, 200)),
            mime_type: Some("image/jpeg".to_string()),
        };
        let cloned = image.clone();
        assert_eq!(image.url, cloned.url);
        assert_eq!(image.alt, cloned.alt);
        assert_eq!(image.dimensions, cloned.dimensions);
        assert_eq!(image.mime_type, cloned.mime_type);
    }

    #[test]
    fn test_image_info_serde_json_full() {
        let image = ImageInfo {
            url: "https://full-image.example.com".to_string(),
            alt: Some("Full image".to_string()),
            dimensions: Some((640, 480)),
            mime_type: Some("image/webp".to_string()),
        };
        let json = serde_json::to_string(&image).unwrap();
        let deserialized: ImageInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(image.url, deserialized.url);
        assert_eq!(image.alt, deserialized.alt);
        assert_eq!(image.dimensions, deserialized.dimensions);
        assert_eq!(image.mime_type, deserialized.mime_type);
    }

    #[test]
    fn test_image_info_serde_json_minimal() {
        let image = ImageInfo {
            url: "https://minimal-image.example.com".to_string(),
            alt: None,
            dimensions: None,
            mime_type: None,
        };
        let json = serde_json::to_string(&image).unwrap();
        assert!(!json.contains("alt"));
        assert!(!json.contains("dimensions"));
        assert!(!json.contains("mime_type"));
        let deserialized: ImageInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(image.url, deserialized.url);
        assert!(deserialized.alt.is_none());
        assert!(deserialized.dimensions.is_none());
        assert!(deserialized.mime_type.is_none());
    }

    // --- FetchParams tests ---

    #[test]
    fn test_fetch_params_default() {
        let params = FetchParams::default();
        assert_eq!(params.url, "");
        assert!(!params.include_raw_html);
        assert!(params.extract_links);
        assert!(!params.extract_images);
        assert_eq!(params.timeout_secs, 30);
        assert!(params.user_agent.is_none());
        assert_eq!(params.max_content_size, 10 * 1024 * 1024);
    }

    #[test]
    fn test_fetch_params_custom() {
        let params = FetchParams {
            url: "https://custom.example.com".to_string(),
            include_raw_html: true,
            extract_links: false,
            extract_images: true,
            timeout_secs: 60,
            user_agent: Some("CustomBot/1.0".to_string()),
            max_content_size: 5 * 1024 * 1024,
        };
        assert_eq!(params.url, "https://custom.example.com");
        assert!(params.include_raw_html);
        assert!(!params.extract_links);
        assert!(params.extract_images);
        assert_eq!(params.timeout_secs, 60);
        assert_eq!(params.user_agent, Some("CustomBot/1.0".to_string()));
        assert_eq!(params.max_content_size, 5 * 1024 * 1024);
    }

    #[test]
    fn test_fetch_params_clone() {
        let params = FetchParams {
            url: "https://clone-params.example.com".to_string(),
            include_raw_html: true,
            extract_links: false,
            extract_images: true,
            timeout_secs: 45,
            user_agent: Some("CloneBot/2.0".to_string()),
            max_content_size: 1 * 1024 * 1024,
        };
        let cloned = params.clone();
        assert_eq!(params.url, cloned.url);
        assert_eq!(params.include_raw_html, cloned.include_raw_html);
        assert_eq!(params.extract_links, cloned.extract_links);
        assert_eq!(params.extract_images, cloned.extract_images);
        assert_eq!(params.timeout_secs, cloned.timeout_secs);
        assert_eq!(params.user_agent, cloned.user_agent);
        assert_eq!(params.max_content_size, cloned.max_content_size);
    }

    #[test]
    fn test_fetch_params_deserialize_json_defaults() {
        let json = r#"{"url": "https://default-params.example.com"}"#;
        let params: FetchParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.url, "https://default-params.example.com");
        assert!(!params.include_raw_html);
        assert!(params.extract_links);
        assert!(!params.extract_images);
        assert_eq!(params.timeout_secs, 30);
        assert!(params.user_agent.is_none());
        assert_eq!(params.max_content_size, 10 * 1024 * 1024);
    }

    #[test]
    fn test_fetch_params_deserialize_json_full() {
        let json = r#"{
            "url": "https://full-params.example.com",
            "include_raw_html": true,
            "extract_links": false,
            "extract_images": true,
            "timeout_secs": 120,
            "user_agent": "FullBot/3.0",
            "max_content_size": 20971520
        }"#;
        let params: FetchParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.url, "https://full-params.example.com");
        assert!(params.include_raw_html);
        assert!(!params.extract_links);
        assert!(params.extract_images);
        assert_eq!(params.timeout_secs, 120);
        assert_eq!(params.user_agent, Some("FullBot/3.0".to_string()));
        assert_eq!(params.max_content_size, 20971520);
    }

    #[test]
    fn test_fetch_params_deserialize_yaml() {
        let yaml = r#"
url: https://yaml-params.example.com
include_raw_html: true
extract_links: true
extract_images: true
timeout_secs: 45
user_agent: "YamlBot/1.0"
max_content_size: 5242880
"#;
        let params: FetchParams = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(params.url, "https://yaml-params.example.com");
        assert!(params.include_raw_html);
        assert!(params.extract_links);
        assert!(params.extract_images);
        assert_eq!(params.timeout_secs, 45);
        assert_eq!(params.user_agent, Some("YamlBot/1.0".to_string()));
        assert_eq!(params.max_content_size, 5242880);
    }

    #[test]
    fn test_fetch_params_deserialize_yaml_partial() {
        let yaml = r#"
url: https://partial-yaml.example.com
extract_images: true
"#;
        let params: FetchParams = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(params.url, "https://partial-yaml.example.com");
        assert!(!params.include_raw_html);
        assert!(params.extract_links);
        assert!(params.extract_images);
        assert_eq!(params.timeout_secs, 30);
        assert!(params.user_agent.is_none());
        assert_eq!(params.max_content_size, 10 * 1024 * 1024);
    }

    #[test]
    fn test_fetch_params_deserialize_json_roundtrip() {
        let json = r#"{
            "url": "https://roundtrip.example.com",
            "include_raw_html": true,
            "extract_links": true,
            "extract_images": false,
            "timeout_secs": 90,
            "user_agent": "RoundTripBot/1.0",
            "max_content_size": 8388608
        }"#;
        let params: FetchParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.url, "https://roundtrip.example.com");
        assert!(params.include_raw_html);
        assert!(params.extract_links);
        assert!(!params.extract_images);
        assert_eq!(params.timeout_secs, 90);
        assert_eq!(params.user_agent, Some("RoundTripBot/1.0".to_string()));
        assert_eq!(params.max_content_size, 8 * 1024 * 1024);
    }
}
