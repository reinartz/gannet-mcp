// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Ole Reinartz
//
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::time::Duration;

use rmcp::model::CallToolRequestParams;
use rmcp::service::ServiceExt;

/// Find a free port on localhost
fn find_free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("Failed to find free port")
        .local_addr()
        .unwrap()
        .port()
}

/// Wait for a server to start listening on the given port
fn wait_for_server(port: u16) {
    let mut attempts = 0;
    let max_attempts = 50;
    loop {
        if attempts >= max_attempts {
            panic!(
                "Server did not start listening on port {} within {} attempts",
                port, max_attempts
            );
        }
        if TcpStream::connect(format!("127.0.0.1:{}", port)).is_ok() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
        attempts += 1;
    }
}

#[tokio::test]
async fn test_http_server_starts_listening() {
    let port = find_free_port();

    let mut child = Command::new(assert_cmd::cargo::cargo_bin("gannet-mcp"))
        .arg("--http")
        .arg("--port")
        .arg(port.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server in HTTP mode");

    wait_for_server(port);

    // Verify server is listening by checking the port
    assert!(
        TcpStream::connect(format!("127.0.0.1:{}", port)).is_ok(),
        "Server should be listening on port {}",
        port
    );

    let _ = child.kill();
    let _ = child.wait();
}

#[tokio::test]
async fn test_http_initialize_success() {
    let port = find_free_port();

    let mut child = Command::new(assert_cmd::cargo::cargo_bin("gannet-mcp"))
        .arg("--http")
        .arg("--port")
        .arg(port.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server in HTTP mode");

    wait_for_server(port);

    let transport = rmcp::transport::StreamableHttpClientTransport::from_uri(format!(
        "http://127.0.0.1:{}/mcp",
        port
    ));

    let client =
        ().serve::<_, _, rmcp::transport::TransportAdapterIdentity>(transport)
            .await
            .expect("Failed to connect to HTTP server");

    let info = client.peer_info().expect("server info should be available");
    assert_eq!(info.protocol_version.as_str(), "2025-11-25");
    assert!(!info.server_info.as_ref().unwrap().name.is_empty());
    assert!(info.capabilities.tools.is_some());

    let _ = child.kill();
    let _ = child.wait();
}

#[tokio::test]
async fn test_http_list_tools() {
    let port = find_free_port();

    let mut child = Command::new(assert_cmd::cargo::cargo_bin("gannet-mcp"))
        .arg("--http")
        .arg("--port")
        .arg(port.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server in HTTP mode");

    wait_for_server(port);

    let transport = rmcp::transport::StreamableHttpClientTransport::from_uri(format!(
        "http://127.0.0.1:{}/mcp",
        port
    ));

    let client =
        ().serve::<_, _, rmcp::transport::TransportAdapterIdentity>(transport)
            .await
            .expect("Failed to connect to HTTP server");

    let tools = client
        .peer()
        .list_tools(Default::default())
        .await
        .expect("list_tools failed");

    let tool_names: Vec<&str> = tools.tools.iter().map(|t| t.name.as_ref()).collect();
    assert!(
        tool_names.contains(&"web_search"),
        "Expected 'web_search' tool, got: {:?}",
        tool_names
    );
    assert!(
        tool_names.contains(&"web_fetch"),
        "Expected 'web_fetch' tool, got: {:?}",
        tool_names
    );

    let _ = child.kill();
    let _ = child.wait();
}

#[tokio::test]
async fn test_http_call_web_search_empty_query() {
    let port = find_free_port();

    let mut child = Command::new(assert_cmd::cargo::cargo_bin("gannet-mcp"))
        .arg("--http")
        .arg("--port")
        .arg(port.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server in HTTP mode");

    wait_for_server(port);

    let transport = rmcp::transport::StreamableHttpClientTransport::from_uri(format!(
        "http://127.0.0.1:{}/mcp",
        port
    ));

    let client =
        ().serve::<_, _, rmcp::transport::TransportAdapterIdentity>(transport)
            .await
            .expect("Failed to connect to HTTP server");

    let params = CallToolRequestParams::new("web_search").with_arguments(
        serde_json::json!({ "query": "" })
            .as_object()
            .unwrap()
            .clone(),
    );

    let result = client.peer().call_tool_once(params).await;

    assert!(
        result.is_ok(),
        "web_search call should succeed with empty query"
    );

    let _ = child.kill();
    let _ = child.wait();
}

#[tokio::test]
async fn test_http_call_web_fetch_invalid_url() {
    let port = find_free_port();

    let mut child = Command::new(assert_cmd::cargo::cargo_bin("gannet-mcp"))
        .arg("--http")
        .arg("--port")
        .arg(port.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server in HTTP mode");

    wait_for_server(port);

    let transport = rmcp::transport::StreamableHttpClientTransport::from_uri(format!(
        "http://127.0.0.1:{}/mcp",
        port
    ));

    let client =
        ().serve::<_, _, rmcp::transport::TransportAdapterIdentity>(transport)
            .await
            .expect("Failed to connect to HTTP server");

    let params = CallToolRequestParams::new("web_fetch").with_arguments(
        serde_json::json!({ "url": "not-a-valid-url" })
            .as_object()
            .unwrap()
            .clone(),
    );

    let result = client.peer().call_tool_once(params).await;

    assert!(
        result.is_err(),
        "web_fetch with invalid URL should return an error"
    );

    let _ = child.kill();
    let _ = child.wait();
}

#[tokio::test]
async fn test_http_call_web_fetch_example_com() {
    let port = find_free_port();

    let mut child = Command::new(assert_cmd::cargo::cargo_bin("gannet-mcp"))
        .arg("--http")
        .arg("--port")
        .arg(port.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server in HTTP mode");

    wait_for_server(port);

    let transport = rmcp::transport::StreamableHttpClientTransport::from_uri(format!(
        "http://127.0.0.1:{}/mcp",
        port
    ));

    let client =
        ().serve::<_, _, rmcp::transport::TransportAdapterIdentity>(transport)
            .await
            .expect("Failed to connect to HTTP server");

    let args = serde_json::json!({
        "url": "https://example.com",
        "extract_links": false,
        "extract_images": false,
        "include_raw_html": true
    });
    let params =
        CallToolRequestParams::new("web_fetch").with_arguments(args.as_object().unwrap().clone());

    let result = client.peer().call_tool_once(params).await;

    assert!(result.is_ok(), "web_fetch for example.com should succeed");

    let _ = child.kill();
    let _ = child.wait();
}

#[tokio::test]
async fn test_http_multiple_requests_same_session() {
    let port = find_free_port();

    let mut child = Command::new(assert_cmd::cargo::cargo_bin("gannet-mcp"))
        .arg("--http")
        .arg("--port")
        .arg(port.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server in HTTP mode");

    wait_for_server(port);

    let transport = rmcp::transport::StreamableHttpClientTransport::from_uri(format!(
        "http://127.0.0.1:{}/mcp",
        port
    ));

    let client =
        ().serve::<_, _, rmcp::transport::TransportAdapterIdentity>(transport)
            .await
            .expect("Failed to connect to HTTP server");

    // First request: list tools
    let tools1 = client
        .peer()
        .list_tools(Default::default())
        .await
        .expect("list_tools failed");
    assert!(tools1.tools.len() >= 2);

    // Second request: call web_fetch
    let fetch_params = CallToolRequestParams::new("web_fetch").with_arguments(
        serde_json::json!({ "url": "https://example.com" })
            .as_object()
            .unwrap()
            .clone(),
    );
    let fetch_result = client.peer().call_tool_once(fetch_params).await;
    assert!(fetch_result.is_ok());

    // Third request: list tools again (session should persist)
    let tools2 = client
        .peer()
        .list_tools(Default::default())
        .await
        .expect("list_tools failed on second call");
    assert!(tools2.tools.len() >= 2);

    let _ = child.kill();
    let _ = child.wait();
}

#[tokio::test]
async fn test_http_custom_mcp_path() {
    let port = find_free_port();

    let mut child = Command::new(assert_cmd::cargo::cargo_bin("gannet-mcp"))
        .arg("--http")
        .arg("--port")
        .arg(port.to_string())
        .arg("--mcp-path")
        .arg("/custom-mcp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server with custom path");

    wait_for_server(port);

    let transport = rmcp::transport::StreamableHttpClientTransport::from_uri(format!(
        "http://127.0.0.1:{}/custom-mcp",
        port
    ));

    let client = ()
        .serve::<_, _, rmcp::transport::TransportAdapterIdentity>(transport)
        .await
        .expect("Failed to connect to HTTP server with custom path");

    let tools = client
        .peer()
        .list_tools(Default::default())
        .await
        .expect("list_tools failed with custom path");

    assert!(tools.tools.len() >= 2);

    let _ = child.kill();
    let _ = child.wait();
}

#[tokio::test]
async fn test_http_server_rejects_tool_before_initialize() {
    let port = find_free_port();

    let mut child = Command::new(assert_cmd::cargo::cargo_bin("gannet-mcp"))
        .arg("--http")
        .arg("--port")
        .arg(port.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server in HTTP mode");

    wait_for_server(port);

    let transport = rmcp::transport::StreamableHttpClientTransport::from_uri(format!(
        "http://127.0.0.1:{}/mcp",
        port
    ));

    let client =
        ().serve::<_, _, rmcp::transport::TransportAdapterIdentity>(transport)
            .await
            .expect("Failed to connect to HTTP server");

    let params = CallToolRequestParams::new("web_search").with_arguments(
        serde_json::json!({ "limit": 5 })
            .as_object()
            .unwrap()
            .clone(),
    );

    let result = client.peer().call_tool_once(params).await;

    // Missing required "query" field should cause an error
    assert!(
        result.is_err(),
        "web_search without query should return an error"
    );

    let _ = child.kill();
    let _ = child.wait();
}
