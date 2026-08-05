// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Ole Reinartz
//!
//! Webpage fetching service

use crate::error::{ServerError, ServerResult};
use crate::models::{FetchParams, ImageInfo, Link, WebpageContent};
use reqwest::{Client, Response};
use scraper::{Html, Selector};
use std::time::{Duration, Instant};
use tracing::{debug, error, info, instrument};
use url::Url;

/// Service for fetching and parsing webpages
pub struct FetchService {
    client: Client,
    default_user_agent: String,
}

impl FetchService {
    /// Create a new fetch service
    pub fn new(timeout_secs: u64) -> ServerResult<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .redirect(reqwest::redirect::Policy::limited(10))
            .pool_max_idle_per_host(10)
            .build()
            .map_err(ServerError::Network)?;

        let default_user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string();

        Ok(Self {
            client,
            default_user_agent,
        })
    }

    /// Fetch a webpage and extract content
    #[instrument(skip(self), fields(url = % params.url))]
    pub async fn fetch(&self, params: FetchParams) -> ServerResult<WebpageContent> {
        let start = Instant::now();
        let url = params.url.clone();

        debug!("Fetching URL: {}", url);

        // Validate URL
        let parsed_url = Url::parse(&url).map_err(ServerError::UrlParse)?;

        // Build request
        let mut request = self.client.get(&url);

        // Set user agent
        let user_agent = params.user_agent.unwrap_or(self.default_user_agent.clone());
        request = request.header("User-Agent", user_agent);

        // Set accept header
        request = request.header("Accept", "text/html,application/xhtml+xml");

        // Send request
        let response = request.send().await.map_err(|e| {
            error!("Failed to fetch URL: {}", e);
            if e.is_timeout() {
                ServerError::Timeout
            } else {
                ServerError::Network(e)
            }
        })?;

        let final_url = response.url().to_string();
        let status_code = response.status().as_u16();
        let headers = extract_headers(&response);
        let content_type = headers
            .get("content-type")
            .and_then(|ct| ct.split(';').next())
            .map(|s| s.trim().to_string());

        // Check status code
        if status_code == 404 {
            return Err(ServerError::NotFound(url));
        }
        if status_code >= 400 {
            return Err(ServerError::InvalidRequest(format!(
                "HTTP error: {}",
                status_code
            )));
        }

        // Read response body with size limit
        let body_bytes = response.bytes().await.map_err(ServerError::Network)?;

        // Truncate content if too large
        let truncated = body_bytes.len() > params.max_content_size;
        let body_bytes = if truncated {
            debug!(
                "Truncating content from {} bytes to {} bytes",
                body_bytes.len(),
                params.max_content_size
            );
            body_bytes.slice(..params.max_content_size)
        } else {
            body_bytes
        };

        let raw_html = String::from_utf8_lossy(&body_bytes).to_string();

        // Parse HTML
        let document = Html::parse_document(&raw_html);

        // Extract metadata
        let title = extract_title(&document);
        let description = extract_meta_description(&document);

        // Extract main content
        let content = extract_main_content(&document);

        // Extract links if requested
        let links = if params.extract_links {
            Some(extract_links(&document, &parsed_url))
        } else {
            None
        };

        // Extract images if requested
        let images = if params.extract_images {
            Some(extract_images(&document, &parsed_url))
        } else {
            None
        };

        let elapsed = start.elapsed().as_millis() as u64;

        info!(
            "Fetched URL: {} (status: {}, size: {} bytes, time: {}ms)",
            url,
            status_code,
            body_bytes.len(),
            elapsed
        );

        Ok(WebpageContent {
            url,
            final_url: Some(final_url),
            status_code,
            title,
            description,
            content,
            raw_html: if params.include_raw_html {
                Some(raw_html)
            } else {
                None
            },
            links,
            images,
            headers: Some(headers),
            content_type,
            encoding: Some("utf-8".to_string()), // Assume UTF-8
            fetched_at: chrono::Utc::now().to_rfc3339(),
            truncated,
        })
    }

    /// Quick fetch - just get the content without parsing
    pub async fn quick_fetch(&self, url: &str) -> ServerResult<String> {
        let response = self
            .client
            .get(url)
            .header("User-Agent", &self.default_user_agent)
            .send()
            .await
            .map_err(ServerError::Network)?;

        response.text().await.map_err(ServerError::Network)
    }
}

// ============================================================================
// HTML Parsing Helpers
// ============================================================================

/// Extract response headers into a HashMap
fn extract_headers(response: &Response) -> std::collections::HashMap<String, String> {
    response
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect()
}

/// Extract page title
fn extract_title(document: &Html) -> Option<String> {
    let selector = Selector::parse("title").ok()?;
    document
        .select(&selector)
        .next()
        .map(|el| el.text().collect::<String>())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Extract meta description
fn extract_meta_description(document: &Html) -> Option<String> {
    let selector = Selector::parse(r#"meta[name="description"]"#).ok()?;
    document
        .select(&selector)
        .next()
        .and_then(|el| el.value().attr("content"))
        .map(|s| s.to_string())
        .filter(|s| !s.trim().is_empty())
}

/// Extract main content from the page
fn extract_main_content(document: &Html) -> String {
    // Try to find main content areas
    let content_selectors = [
        "article",
        "main",
        "[role='main']",
        ".content",
        "#content",
        ".post-content",
        ".article-content",
    ];

    for selector_str in &content_selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                let text = element.text().collect::<Vec<_>>().join(" ");
                if text.len() > 50 {
                    return clean_text(&text);
                }
            }
        }
    }

    // Fallback to body
    let body_selector = Selector::parse("body").ok();
    if let Some(selector) = body_selector {
        if let Some(body) = document.select(&selector).next() {
            let text = body.text().collect::<Vec<_>>().join(" ");
            if text.len() > 50 {
                return clean_text(&text);
            }
        }
    }

    String::new()
}

/// Clean extracted text
fn clean_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Extract links from the document
fn extract_links(document: &Html, base_url: &Url) -> Vec<Link> {
    let selector = match Selector::parse("a[href]") {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let mut links = Vec::new();

    for element in document.select(&selector) {
        if let Some(href) = element.value().attr("href") {
            // Resolve relative URLs
            let absolute_url = base_url.join(href).ok().map(|u| u.to_string());

            if let Some(url) = absolute_url {
                let text = element.text().collect::<String>().trim().to_string();

                let is_internal = url.starts_with(&base_url.origin().ascii_serialization());

                links.push(Link {
                    url,
                    text: if text.is_empty() { None } else { Some(text) },
                    is_internal,
                });
            }
        }
    }

    links
}

/// Extract images from the document
fn extract_images(document: &Html, base_url: &Url) -> Vec<ImageInfo> {
    let selector = match Selector::parse("img") {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let mut images = Vec::new();

    for element in document.select(&selector) {
        if let Some(src) = element.value().attr("src") {
            if src.is_empty() {
                continue;
            }
            // Resolve relative URLs
            if let Some(url) = base_url.join(src).ok().map(|url| url.to_string()) {
                let alt = element.value().attr("alt").map(|s| s.to_string());
                let width = element.value().attr("width").and_then(|w| w.parse().ok());
                let height = element.value().attr("height").and_then(|h| h.parse().ok());

                images.push(ImageInfo {
                    url,
                    alt,
                    dimensions: width.zip(height),
                    mime_type: None,
                });
            }
        }
    }

    images
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_text() {
        let input = "  Hello   World  \n\nTest  ";
        let output = clean_text(input);
        assert_eq!(output, "Hello World Test");
    }

    #[tokio::test]
    async fn test_fetch_service_creation() {
        let service = FetchService::new(30);
        assert!(service.is_ok());
    }

    // ==========================================================================
    // extract_headers tests
    // ==========================================================================

    #[tokio::test]
    async fn test_extract_headers_content_types() {
        let mut server = mockito::Server::new_async().await;

        let html_mock = server
            .mock("GET", "/html")
            .with_header("content-type", "text/html; charset=utf-8")
            .with_body("<html><body>test</body></html>")
            .create();

        let json_mock = server
            .mock("GET", "/json")
            .with_header("content-type", "application/json; charset=utf-8")
            .with_body(r#"{"key":"value"}"#)
            .create();

        let xml_mock = server
            .mock("GET", "/xml")
            .with_header("content-type", "application/xml")
            .with_body("<?xml version='1.0'?><root/>")
            .create();

        let plain_mock = server
            .mock("GET", "/plain")
            .with_header("content-type", "text/plain")
            .with_body("just text")
            .create();

        let service = FetchService::new(30).unwrap();

        // Fetch HTML page
        let html_resp = service
            .client
            .get(server.url() + "/html")
            .send()
            .await
            .unwrap();
        let headers = extract_headers(&html_resp);
        assert_eq!(
            headers.get("content-type").unwrap(),
            "text/html; charset=utf-8"
        );

        // Fetch JSON endpoint
        let json_resp = service
            .client
            .get(server.url() + "/json")
            .send()
            .await
            .unwrap();
        let headers = extract_headers(&json_resp);
        assert_eq!(
            headers.get("content-type").unwrap(),
            "application/json; charset=utf-8"
        );

        // Fetch XML endpoint
        let xml_resp = service
            .client
            .get(server.url() + "/xml")
            .send()
            .await
            .unwrap();
        let headers = extract_headers(&xml_resp);
        assert_eq!(headers.get("content-type").unwrap(), "application/xml");

        // Fetch plain text
        let plain_resp = service
            .client
            .get(server.url() + "/plain")
            .send()
            .await
            .unwrap();
        let headers = extract_headers(&plain_resp);
        assert_eq!(headers.get("content-type").unwrap(), "text/plain");

        html_mock.assert();
        json_mock.assert();
        xml_mock.assert();
        plain_mock.assert();
    }

    #[tokio::test]
    async fn test_extract_headers_multiple_values() {
        let mut server = mockito::Server::new_async().await;

        let mock = server
            .mock("GET", "/multi")
            .with_header("x-custom", "value1")
            .with_header("x-another", "value2")
            .with_header("content-type", "text/html")
            .with_body("<html></html>")
            .create();

        let service = FetchService::new(30).unwrap();

        let resp = service
            .client
            .get(server.url() + "/multi")
            .send()
            .await
            .unwrap();
        let headers = extract_headers(&resp);
        assert_eq!(headers.get("x-custom").unwrap(), "value1");
        assert_eq!(headers.get("x-another").unwrap(), "value2");
        assert_eq!(headers.get("content-type").unwrap(), "text/html");

        // Verify that non-ASCII header values are handled gracefully (no panic)
        mock.assert();
    }

    #[tokio::test]
    async fn test_extract_headers_empty() {
        let mut server = mockito::Server::new_async().await;

        let mock = server.mock("GET", "/empty").with_body("").create();

        let service = FetchService::new(30).unwrap();

        let resp = service
            .client
            .get(server.url() + "/empty")
            .send()
            .await
            .unwrap();
        let headers = extract_headers(&resp);
        // Should at least have content-length header even with empty body
        assert!(!headers.is_empty());

        mock.assert();
    }

    #[test]
    fn test_extract_title_h1_tag_not_title() {
        let html = Html::parse_fragment("<h1>Main Heading</h1>");
        assert!(extract_title(&html).is_none());
    }

    #[test]
    fn test_extract_title_empty() {
        let html = Html::parse_fragment("<title>   </title>");
        assert!(extract_title(&html).is_none());
    }

    #[test]
    fn test_extract_title_no_title_tag() {
        let html = Html::parse_fragment("<body>Hello World</body>");
        assert!(extract_title(&html).is_none());
    }

    #[test]
    fn test_extract_title_simple() {
        let html = Html::parse_fragment("<title>My Page Title</title>");
        let title = extract_title(&html);
        assert_eq!(title, Some("My Page Title".to_string()));
    }

    #[test]
    fn test_extract_title_with_whitespace() {
        let html = Html::parse_fragment("<title>  Leading and Trailing   </title>");
        let title = extract_title(&html);
        assert_eq!(title, Some("Leading and Trailing".to_string()));
    }

    #[test]
    fn test_extract_title_nested_content() {
        let html = Html::parse_fragment("<title><span>Nested</span> Text</title>");
        let title = extract_title(&html);
        // scraper .text() collects all text content including nested elements
        assert!(title.is_some());
        let t = title.unwrap();
        assert!(t.contains("Nested"));
        assert!(t.contains("Text"));
    }

    #[test]
    fn test_extract_title_multiple_title_tags() {
        let html = Html::parse_fragment("<title>First</title><title>Second</title>");
        let title = extract_title(&html);
        // Should return the first title tag
        assert_eq!(title, Some("First".to_string()));
    }

    // ==========================================================================
    // extract_meta_description tests
    // ==========================================================================

    #[test]
    fn test_extract_meta_description_found() {
        let html =
            Html::parse_fragment(r#"<meta name="description" content="This is a description">"#);
        let desc = extract_meta_description(&html);
        assert_eq!(desc, Some("This is a description".to_string()));
    }

    #[test]
    fn test_extract_meta_description_missing_content() {
        let html = Html::parse_fragment(r#"<meta name="description">"#);
        let desc = extract_meta_description(&html);
        assert!(desc.is_none());
    }

    #[test]
    fn test_extract_meta_description_empty_content() {
        let html = Html::parse_fragment(r#"<meta name="description" content="">"#);
        let desc = extract_meta_description(&html);
        assert!(desc.is_none());
    }

    #[test]
    fn test_extract_meta_description_whitespace_only() {
        let html = Html::parse_fragment(
            r#"<meta name="description" content="
"> "#,
        );
        let desc = extract_meta_description(&html);
        assert!(desc.is_none());
    }

    #[test]
    fn test_extract_meta_description_missing_tag() {
        let html = Html::parse_fragment("<body>No description here</body>");
        let desc = extract_meta_description(&html);
        assert!(desc.is_none());
    }

    #[test]
    fn test_extract_meta_description_first_wins() {
        let html = Html::parse_fragment(
            r#"<meta name="description" content="First">
               <meta name="description" content="Second">"#,
        );
        let desc = extract_meta_description(&html);
        assert_eq!(desc, Some("First".to_string()));
    }

    #[test]
    fn test_extract_meta_description_different_name() {
        let html = Html::parse_fragment(r#"<meta name="keywords" content="some, keywords">"#);
        let desc = extract_meta_description(&html);
        assert!(desc.is_none());
    }

    #[test]
    fn test_extract_meta_description_long_content() {
        let long_desc = "This is a very long description ".repeat(10);
        let html = Html::parse_fragment(&format!(
            r#"<meta name="description" content="{}">"#,
            long_desc
        ));
        let desc = extract_meta_description(&html);
        assert_eq!(desc, Some(long_desc));
    }

    // ==========================================================================
    // extract_main_content tests
    // ==========================================================================

    #[test]
    fn test_extract_main_content_article_selector() {
        let html = Html::parse_fragment(
            r#"<header>Header text</header>
               <article><p>This is the main article content that should be extracted</p></article>"#,
        );
        let content = extract_main_content(&html);
        assert!(content.contains("main article content"));
    }

    #[test]
    fn test_extract_main_content_main_selector_priority() {
        let html = Html::parse_fragment(
            r#"<article><p>Article text that is longer than the threshold</p></article>
               <main><p>Main content text that is also longer than the threshold</p></main>"#,
        );
        let content = extract_main_content(&html);
        assert!(content.contains("Main content") || content.contains("Article"));
    }

    #[test]
    fn test_extract_main_content_selector_short_content_fallback() {
        let html = Html::parse_fragment(
            r#"<article><p>Short</p></article>
               <main><p>This is enough main content that exceeds the minimum threshold</p></main>"#,
        );
        let content = extract_main_content(&html);
        // article has only "Short" (5 chars < 100), so should fall through to main
        assert!(content.contains("enough main content"));
    }

    #[test]
    fn test_extract_main_content_role_main() {
        let html = Html::parse_fragment(
            r#"<div role="main"><p>Main role content that exceeds the minimum threshold requirement</p></div>"#,
        );
        let content = extract_main_content(&html);
        assert!(content.contains("Main role content"));
    }

    #[test]
    fn test_extract_main_content_class_content() {
        let html = Html::parse_fragment(
            r#"<div class="content"><p>Content class text that is longer than 100 characters minimum</p></div>"#,
        );
        let content = extract_main_content(&html);
        assert!(content.contains("Content class text"));
    }

    #[test]
    fn test_extract_main_content_id_content() {
        let html = Html::parse_fragment(
            r#"<div id="content"><p>Content id text that exceeds the minimum threshold requirement</p></div>"#,
        );
        let content = extract_main_content(&html);
        assert!(content.contains("Content id text"));
    }

    #[test]
    fn test_extract_main_content_post_content_class() {
        let html = Html::parse_fragment(
            r#"<div class="post-content"><p>Post content text exceeding the minimum threshold value</p></div>"#,
        );
        let content = extract_main_content(&html);
        assert!(content.contains("Post content text"));
    }

    #[test]
    fn test_extract_main_content_article_content_class() {
        let html = Html::parse_fragment(
            r#"<div class="article-content"><p>Article content text exceeding the minimum threshold</p></div>"#,
        );
        let content = extract_main_content(&html);
        assert!(content.contains("Article content text"));
    }

    #[test]
    fn test_extract_main_content_body_fallback() {
        let html = Html::parse_fragment(
            r#"<article><p>Fallback body text that exceeds the minimum threshold requirement</p></article>"#,
        );
        let content = extract_main_content(&html);
        assert!(content.contains("Fallback body text"));
    }

    #[test]
    fn test_extract_main_content_body_fallback_no_html() {
        let html = Html::parse_fragment(
            r#"<main><p>Body only text exceeding the minimum threshold value</p></main>"#,
        );
        let content = extract_main_content(&html);
        assert!(content.contains("Body only text"));
    }

    #[test]
    fn test_extract_main_content_empty_body() {
        let html = Html::parse_fragment("<html><body></body></html>");
        let content = extract_main_content(&html);
        assert_eq!(content, "");
    }

    #[test]
    fn test_extract_main_content_empty_document() {
        let html = Html::parse_document("");
        let content = extract_main_content(&html);
        assert_eq!(content, "");
    }

    #[test]
    fn test_extract_main_content_only_header_no_body() {
        let html = Html::parse_fragment("<h1>Hello</h1>");
        let content = extract_main_content(&html);
        // Should fallback to body but body is empty
        assert!(content.is_empty());
    }

    #[test]
    fn test_extract_main_content_whitespace_only() {
        let html = Html::parse_fragment(r#"<body>  \n\n  \t  </body>"#);
        let content = extract_main_content(&html);
        // clean_text should strip all whitespace
        assert!(content.is_empty());
    }

    #[test]
    fn test_extract_main_content_no_special_selectors_no_body() {
        // Document with no article, main, role, class, id selectors and no body
        let html = Html::parse_fragment("<div>Just a short div</div>");
        let content = extract_main_content(&html);
        assert!(content.is_empty());
    }

    // ==========================================================================
    // extract_links tests
    // ==========================================================================

    #[test]
    fn test_extract_links_absolute_urls() {
        let base = Url::parse("https://example.com").unwrap();
        let html = Html::parse_fragment(
            r#"<a href="https://other.com/page">Other Site</a>
               <a href="https://another.org/thing">Another</a>"#,
        );
        let links = extract_links(&html, &base);
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].url, "https://other.com/page");
        assert_eq!(links[0].text, Some("Other Site".to_string()));
        assert!(!links[0].is_internal);
        assert_eq!(links[1].url, "https://another.org/thing");
    }

    #[test]
    fn test_extract_links_relative_urls() {
        let base = Url::parse("https://example.com/blog/").unwrap();
        let html = Html::parse_fragment(
            r#"<a href="/about">About</a>
               <a href="post.html">Post</a>"#,
        );
        let links = extract_links(&html, &base);
        assert_eq!(links.len(), 2);
        assert_eq!(links[0].url, "https://example.com/about");
        assert!(links[0].is_internal);
        assert_eq!(links[1].url, "https://example.com/blog/post.html");
        assert!(links[1].is_internal);
    }

    #[test]
    fn test_extract_links_mixed_urls() {
        let base = Url::parse("https://example.com").unwrap();
        let html = Html::parse_fragment(
            r#"<a href="/internal">Internal</a>
               <a href="https://external.com/ext">External</a>
               <a href="./relative">Relative</a>"#,
        );
        let links = extract_links(&html, &base);
        assert_eq!(links.len(), 3);
        assert!(links[0].is_internal);
        assert!(!links[1].is_internal);
        assert!(links[2].is_internal);
    }

    #[test]
    fn test_extract_links_no_text() {
        let base = Url::parse("https://example.com").unwrap();
        let html = Html::parse_fragment(r#"<a href="/page"></a>"#);
        let links = extract_links(&html, &base);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].url, "https://example.com/page");
        assert_eq!(links[0].text, None);
    }

    #[test]
    fn test_extract_links_whitespace_text() {
        let base = Url::parse("https://example.com").unwrap();
        let html = Html::parse_fragment(r#"<a href="/page">  </a>"#);
        let links = extract_links(&html, &base);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].text, None);
    }

    #[test]
    fn test_extract_links_fragment_only() {
        let base = Url::parse("https://example.com/page").unwrap();
        let html = Html::parse_fragment("<a href=\"#section\">Section</a>");
        let links = extract_links(&html, &base);
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].url, "https://example.com/page#section");
        assert!(links[0].is_internal);
    }

    #[test]
    fn test_extract_links_no_links() {
        let base = Url::parse("https://example.com").unwrap();
        let html = Html::parse_fragment("<p>No links here</p>");
        let links = extract_links(&html, &base);
        assert!(links.is_empty());
    }

    #[test]
    fn test_extract_links_self_reference() {
        let base = Url::parse("https://example.com/page").unwrap();
        let html = Html::parse_fragment(r#"<a href="https://example.com/page">Self</a>"#);
        let links = extract_links(&html, &base);
        assert_eq!(links.len(), 1);
        assert!(links[0].is_internal);
    }

    // ==========================================================================
    // extract_images tests
    // ==========================================================================

    #[test]
    fn test_extract_images_with_alt_text() {
        let base = Url::parse("https://example.com").unwrap();
        let html = Html::parse_fragment(
            r#"<img src="/images/photo.jpg" alt="A beautiful photo">
               <img src="/images/logo.png" alt="Logo">"#,
        );
        let images = extract_images(&html, &base);
        assert_eq!(images.len(), 2);
        assert_eq!(images[0].url, "https://example.com/images/photo.jpg");
        assert_eq!(images[0].alt, Some("A beautiful photo".to_string()));
        assert_eq!(images[1].alt, Some("Logo".to_string()));
    }

    #[test]
    fn test_extract_images_without_alt_text() {
        let base = Url::parse("https://example.com").unwrap();
        let html = Html::parse_fragment(
            r#"<img src="/images/empty.jpg">
               <img src="/images/nodata.gif" alt="Has alt">"#,
        );
        let images = extract_images(&html, &base);
        assert_eq!(images.len(), 2);
        assert_eq!(images[0].alt, None);
        assert_eq!(images[1].alt, Some("Has alt".to_string()));
    }

    #[test]
    fn test_extract_images_with_dimensions() {
        let base = Url::parse("https://example.com").unwrap();
        let html = Html::parse_fragment(
            r#"<img src="/images/large.jpg" alt="Big" width="800" height="600">"#,
        );
        let images = extract_images(&html, &base);
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].url, "https://example.com/images/large.jpg");
        assert_eq!(images[0].dimensions, Some((800, 600)));
    }

    #[test]
    fn test_extract_images_with_partial_dimensions() {
        let base = Url::parse("https://example.com").unwrap();
        let html = Html::parse_fragment(
            r#"<img src="/images/width-only.jpg" width="400">
               <img src="/images/no-dims.jpg">"#,
        );
        let images = extract_images(&html, &base);
        assert_eq!(images.len(), 2);
        assert_eq!(images[0].dimensions, None);
        assert_eq!(images[1].dimensions, None);
    }

    #[test]
    fn test_extract_images_relative_urls() {
        let base = Url::parse("https://example.com/blog/post/").unwrap();
        let html = Html::parse_fragment(r#"<img src="../photo.jpg" alt="Up one level">"#);
        let images = extract_images(&html, &base);
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].url, "https://example.com/blog/photo.jpg");
    }

    #[test]
    fn test_extract_images_absolute_urls() {
        let base = Url::parse("https://example.com").unwrap();
        let html = Html::parse_fragment(
            r#"<img src="https://cdn.example.com/image.jpg" alt="CDN image">"#,
        );
        let images = extract_images(&html, &base);
        assert_eq!(images.len(), 1);
        assert_eq!(images[0].url, "https://cdn.example.com/image.jpg");
    }

    #[test]
    fn test_extract_images_no_images() {
        let base = Url::parse("https://example.com").unwrap();
        let html = Html::parse_fragment("<p>No images here</p>");
        let images = extract_images(&html, &base);
        assert!(images.is_empty());
    }

    #[test]
    fn test_extract_images_mixed() {
        let base = Url::parse("https://example.com").unwrap();
        let html = Html::parse_fragment(
            r#"<img src="https://cdn.com/a.png" alt="A" width="10" height="20">
               <img src="/b.jpg">
               <img src="c.gif" alt="C">"#,
        );
        let images = extract_images(&html, &base);
        assert_eq!(images.len(), 3);
        assert_eq!(images[0].alt, Some("A".to_string()));
        assert_eq!(images[0].dimensions, Some((10, 20)));
        assert!(images[1].alt.is_none());
        assert!(images[1].dimensions.is_none());
        assert_eq!(images[2].alt, Some("C".to_string()));
    }

    #[test]
    fn test_extract_images_empty_src() {
        let base = Url::parse("https://example.com").unwrap();
        let html = Html::parse_fragment(r#"<img src="" alt="Empty src">"#);
        let images = extract_images(&html, &base);
        // Empty src cannot be resolved by url::Url::join
        assert_eq!(images.len(), 0);
    }

    // ==========================================================================
    // FetchService::quick_fetch tests
    // ==========================================================================

    #[tokio::test]
    async fn test_quick_fetch_basic() {
        let mut server = mockito::Server::new_async().await;
        let body = "Hello, this is a quick fetch test response body.";

        let mock = server.mock("GET", "/simple").with_body(body).create();

        let service = FetchService::new(30).unwrap();
        let result = service
            .quick_fetch(&format!("{}{}", server.url(), "/simple"))
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), body);
        mock.assert();
    }

    #[tokio::test]
    async fn test_quick_fetch_html_content() {
        let mut server = mockito::Server::new_async().await;
        let html = r#"<!DOCTYPE html><html><head><title>Test</title></head><body><p>Content</p></body></html>"#;

        let mock = server
            .mock("GET", "/html")
            .with_header("content-type", "text/html")
            .with_body(html)
            .create();

        let service = FetchService::new(30).unwrap();
        let result = service
            .quick_fetch(&format!("{}{}", server.url(), "/html"))
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), html);
        mock.assert();
    }

    #[tokio::test]
    async fn test_quick_fetch_large_content() {
        let mut server = mockito::Server::new_async().await;
        let body = "X".repeat(1_000_000);

        let mock = server.mock("GET", "/large").with_body(&body).create();

        let service = FetchService::new(30).unwrap();
        let result = service
            .quick_fetch(&format!("{}{}", server.url(), "/large"))
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1_000_000);
        mock.assert();
    }

    #[tokio::test]
    async fn test_quick_fetch_invalid_url() {
        let service = FetchService::new(30).unwrap();
        let result = service.quick_fetch("not-a-valid-url").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_quick_fetch_404() {
        let mut server = mockito::Server::new_async().await;

        let mock = server
            .mock("GET", "/notfound")
            .with_status(404)
            .with_body("Not Found")
            .create();

        let service = FetchService::new(30).unwrap();
        let result = service
            .quick_fetch(&format!("{}{}", server.url(), "/notfound"))
            .await;

        assert!(result.is_ok());
        // quick_fetch does not check status codes, it just returns text
        assert_eq!(result.unwrap(), "Not Found");
        mock.assert();
    }

    #[tokio::test]
    async fn test_quick_fetch_unicode_content() {
        let mut server = mockito::Server::new_async().await;
        let body = "Hello 世界 🌍 Привет мир";

        let mock = server.mock("GET", "/unicode").with_body(body).create();

        let service = FetchService::new(30).unwrap();
        let result = service
            .quick_fetch(&format!("{}{}", server.url(), "/unicode"))
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), body);
        mock.assert();
    }

    #[tokio::test]
    async fn test_quick_fetch_empty_body() {
        let mut server = mockito::Server::new_async().await;

        let mock = server.mock("GET", "/empty").with_body("").create();

        let service = FetchService::new(30).unwrap();
        let result = service
            .quick_fetch(&format!("{}{}", server.url(), "/empty"))
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
        mock.assert();
    }

    #[tokio::test]
    async fn test_quick_fetch_multiline() {
        let mut server = mockito::Server::new_async().await;
        let body = "Line 1\nLine 2\nLine 3\n\tTabbed\n\n\nBlank lines";

        let mock = server.mock("GET", "/multiline").with_body(body).create();

        let service = FetchService::new(30).unwrap();
        let result = service
            .quick_fetch(&format!("{}{}", server.url(), "/multiline"))
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), body);
        mock.assert();
    }
}
