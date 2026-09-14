// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Ole Reinartz
//
use gannet_mcp::models::FetchParams;
use gannet_mcp::services::FetchService;
use mockito::Server;

/// Helper to create a FetchService for testing
fn make_fetch_service() -> FetchService {
    FetchService::new(30).expect("Failed to create FetchService")
}

/// Helper HTML document for testing
fn simple_html_doc() -> String {
    r#"<!DOCTYPE html>
<html>
<head>
    <title>Test Page</title>
    <meta name="description" content="A test page for unit testing">
</head>
<body>
    <main>
        <h1>Welcome to the Test Page</h1>
        <p>This is a paragraph with some content for testing purposes.</p>
        <a href="/page1">Link 1</a>
        <a href="/page2">Link 2</a>
        <a href="https://external.com">External Link</a>
        <img src="/image1.png" alt="Image 1" width="100" height="50">
        <img src="/image2.png" alt="Image 2">
    </main>
</body>
</html>"#
        .to_string()
}

fn run_async<F, T>(f: F) -> T
where
    F: std::future::Future<Output = T>,
{
    tokio::runtime::Runtime::new()
        .expect("Failed to create runtime")
        .block_on(f)
}

#[test]
fn test_fetch_basic_content_extraction() {
    let mut server = Server::new();
    let html = simple_html_doc();
    let url = server.url();

    let mock = server
        .mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body(&html)
        .create();

    let result = run_async(async move {
        let service = make_fetch_service();
        let params = FetchParams {
            url,
            extract_links: false,
            extract_images: false,
            include_raw_html: false,
            ..Default::default()
        };
        service.fetch(params).await
    });

    let content = result.expect("Fetch should succeed");
    assert_eq!(content.status_code, 200);
    assert_eq!(content.title, Some("Test Page".to_string()));
    assert_eq!(
        content.description,
        Some("A test page for unit testing".to_string())
    );
    assert!(content.content.contains("Welcome to the Test Page"));
    assert!(content.content.contains("This is a paragraph"));
    assert!(content.links.is_none());
    assert!(content.images.is_none());
    assert!(content.raw_html.is_none());

    mock.assert();
}

#[test]
fn test_fetch_with_link_extraction() {
    let mut server = Server::new();
    let html = simple_html_doc();
    let url = server.url();

    let mock = server
        .mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body(&html)
        .create();

    let result = run_async(async move {
        let service = make_fetch_service();
        let params = FetchParams {
            url,
            extract_links: true,
            extract_images: false,
            ..Default::default()
        };
        service.fetch(params).await
    });

    let content = result.expect("Fetch should succeed");
    let links = content.links.expect("Links should be present");
    assert!(
        links.len() >= 2,
        "Expected at least 2 links, got {}",
        links.len()
    );
    assert!(links.iter().any(|l| l.text == Some("Link 1".to_string())));
    assert!(links.iter().any(|l| l.text == Some("Link 2".to_string())));

    mock.assert();
}

#[test]
fn test_fetch_with_image_extraction() {
    let mut server = Server::new();
    let html = simple_html_doc();
    let url = server.url();

    let mock = server
        .mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body(&html)
        .create();

    let result = run_async(async move {
        let service = make_fetch_service();
        let params = FetchParams {
            url,
            extract_links: false,
            extract_images: true,
            ..Default::default()
        };
        service.fetch(params).await
    });

    let content = result.expect("Fetch should succeed");
    let images = content.images.expect("Images should be present");
    assert!(
        !images.is_empty(),
        "Expected at least 1 image, got {}",
        images.len()
    );
    assert!(images
        .iter()
        .any(|img| img.alt == Some("Image 1".to_string())));

    mock.assert();
}

#[test]
fn test_fetch_with_raw_html() {
    let mut server = Server::new();
    let html = simple_html_doc();
    let url = server.url();

    let mock = server
        .mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body(&html)
        .create();

    let result = run_async(async move {
        let service = make_fetch_service();
        let params = FetchParams {
            url,
            include_raw_html: true,
            extract_links: false,
            extract_images: false,
            ..Default::default()
        };
        service.fetch(params).await
    });

    let content = result.expect("Fetch should succeed");
    let raw = content.raw_html.expect("Raw HTML should be present");
    assert!(raw.contains("<title>Test Page</title>"));

    mock.assert();
}

#[test]
fn test_fetch_returns_404_error() {
    let mut server = Server::new();
    let url = server.url();

    let mock = server
        .mock("GET", "/")
        .with_status(404)
        .with_body("Not Found")
        .create();

    let result = run_async(async move {
        let service = make_fetch_service();
        let params = FetchParams {
            url,
            ..Default::default()
        };
        service.fetch(params).await
    });

    assert!(result.is_err(), "Expected error for 404");

    mock.assert();
}

#[test]
fn test_fetch_returns_500_error() {
    let mut server = Server::new();
    let url = server.url();

    let mock = server
        .mock("GET", "/")
        .with_status(500)
        .with_body("Server Error")
        .create();

    let result = run_async(async move {
        let service = make_fetch_service();
        let params = FetchParams {
            url,
            ..Default::default()
        };
        service.fetch(params).await
    });

    assert!(result.is_err(), "Expected error for 500");

    mock.assert();
}

#[test]
fn test_fetch_with_custom_user_agent() {
    let mut server = Server::new();
    let url = server.url();

    let mock = server
        .mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body("<html><body>Test</body></html>")
        .create();

    let result = run_async(async move {
        let service = make_fetch_service();
        let params = FetchParams {
            url,
            user_agent: Some("CustomAgent/1.0".to_string()),
            extract_links: false,
            ..Default::default()
        };
        service.fetch(params).await
    });

    assert!(result.is_ok(), "Fetch should succeed");

    mock.assert();
}

#[test]
fn test_fetch_handles_large_content() {
    let mut server = Server::new();
    let url = server.url();

    // Generate content that exceeds the default max (10MB)
    let large_body = "x".repeat(11 * 1024 * 1024);

    let mock = server
        .mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body(&large_body)
        .create();

    let result = run_async(async move {
        let service = make_fetch_service();
        let params = FetchParams {
            url,
            extract_links: false,
            ..Default::default()
        };
        service.fetch(params).await
    });

    assert!(
        result.is_ok(),
        "Expected oversized content to be truncated, not rejected"
    );
    let content = result.unwrap();
    assert!(content.truncated, "Expected truncated flag to be true");
    assert!(
        content.content.len() < large_body.len(),
        "Expected truncated content to be smaller than original"
    );

    mock.assert();
}

#[test]
fn test_fetch_invalid_url() {
    let result = run_async(async {
        let service = make_fetch_service();
        let params = FetchParams {
            url: "not-a-valid-url".to_string(),
            ..Default::default()
        };
        service.fetch(params).await
    });

    assert!(result.is_err(), "Expected error for invalid URL");
}

#[test]
fn test_fetch_empty_page() {
    let mut server = Server::new();
    let url = server.url();

    let mock = server
        .mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body("<html><body></body></html>")
        .create();

    let result = run_async(async move {
        let service = make_fetch_service();
        let params = FetchParams {
            url,
            extract_links: true,
            extract_images: true,
            ..Default::default()
        };
        service.fetch(params).await
    });

    let content = result.expect("Fetch should succeed");
    assert!(content.content.is_empty() || content.content.trim().is_empty());
    assert!(content.links.unwrap_or_default().is_empty());
    assert!(content.images.unwrap_or_default().is_empty());

    mock.assert();
}

#[test]
fn test_fetch_multiple_paths() {
    let mut server = Server::new();
    let base_url = server.url();

    let mock_home = server
        .mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body("<html><title>Home</title><body>Home page</body></html>")
        .create();

    let mock_about = server
        .mock("GET", "/about")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body("<html><title>About</title><body>About page</body></html>")
        .create();

    let home_url = format!("{}/", base_url);
    let about_url = format!("{}/about", base_url);

    let home_result = run_async(async move {
        let service = make_fetch_service();
        let params = FetchParams {
            url: home_url,
            extract_links: false,
            ..Default::default()
        };
        service.fetch(params).await
    });

    assert_eq!(home_result.unwrap().title, Some("Home".to_string()));

    let about_result = run_async(async move {
        let service = make_fetch_service();
        let params = FetchParams {
            url: about_url,
            extract_links: false,
            ..Default::default()
        };
        service.fetch(params).await
    });

    assert_eq!(about_result.unwrap().title, Some("About".to_string()));

    mock_home.assert();
    mock_about.assert();
}

#[test]
fn test_fetch_with_custom_max_content_size() {
    let mut server = Server::new();
    let url = server.url();

    let body = "Hello World";

    let mock = server
        .mock("GET", "/")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body(body)
        .create();

    let result = run_async(async move {
        let service = make_fetch_service();
        let params = FetchParams {
            url,
            max_content_size: 5, // Only 5 bytes max
            extract_links: false,
            ..Default::default()
        };
        service.fetch(params).await
    });

    assert!(
        result.is_ok(),
        "Expected oversized content to be truncated, not rejected"
    );
    let content = result.unwrap();
    assert!(content.truncated, "Expected truncated flag to be true");

    mock.assert();
}
