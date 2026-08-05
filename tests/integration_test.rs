// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Ole Reinartz
//
use std::env;
use std::io::{BufRead, Write};
use std::process::{Child, Command, Stdio};
use std::sync::LazyLock;

static ENV_TEST_MUTEX: LazyLock<std::sync::Mutex<()>> = LazyLock::new(|| std::sync::Mutex::new(()));

const MCP_PROTOCOL_VERSION: &str = "2025-11-25";

struct McpClient {
    process: Child,
}

impl McpClient {
    fn start() -> Self {
        let process = Command::new(assert_cmd::cargo::cargo_bin("gannet-mcp"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start server binary");

        Self { process }
    }

    fn send_request(&mut self, request: &serde_json::Value) -> serde_json::Value {
        let stdin = self.process.stdin.as_mut().expect("Failed to open stdin");
        let mut line = serde_json::to_string(request).expect("Failed to serialize request");
        line.push('\n');
        stdin
            .write_all(line.as_bytes())
            .expect("Failed to write to stdin");
        stdin.flush().expect("Failed to flush stdin");

        let stdout = self.process.stdout.as_mut().expect("Failed to open stdout");
        let mut reader = std::io::BufReader::new(stdout);
        let mut response_line = String::new();
        reader
            .read_line(&mut response_line)
            .expect("Failed to read response line");

        serde_json::from_str(&response_line).expect("Failed to parse response JSON")
    }

    fn initialize(&mut self) -> serde_json::Value {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": MCP_PROTOCOL_VERSION,
                "capabilities": {},
                "clientInfo": {
                    "name": "integration-test",
                    "version": "1.0.0"
                }
            }
        });
        self.send_request(&request)
    }

    fn send_initialized(&mut self) {
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });
        let stdin = self.process.stdin.as_mut().expect("Failed to open stdin");
        let mut line = serde_json::to_string(&notification).expect("Failed to serialize");
        line.push('\n');
        stdin
            .write_all(line.as_bytes())
            .expect("Failed to write to stdin");
        stdin.flush().expect("Failed to flush stdin");
    }

    fn list_tools(&mut self) -> serde_json::Value {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list"
        });
        self.send_request(&request)
    }

    fn call_tool(&mut self, name: &str, args: serde_json::Value) -> serde_json::Value {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": name,
                "arguments": args
            }
        });
        self.send_request(&request)
    }

    fn shutdown(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[tokio::test]
async fn test_binary_help_flag() {
    let output = Command::new(assert_cmd::cargo::cargo_bin("gannet-mcp"))
        .arg("--help")
        .output()
        .expect("Failed to run --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("gannet-mcp"));
    assert!(stdout.contains("--help"));
}

#[tokio::test]
async fn test_binary_version_flag() {
    let output = Command::new(assert_cmd::cargo::cargo_bin("gannet-mcp"))
        .arg("--version")
        .output()
        .expect("Failed to run --version");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty());
}

#[tokio::test]
async fn test_mcp_initialize_success() {
    let mut client = McpClient::start();

    // Send initialize request
    let response = client.initialize();

    // Verify response structure
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(
        response.get("result").is_some(),
        "Expected 'result' field in initialize response"
    );
    assert!(response["result"]["serverInfo"].is_object());
    assert!(response["result"]["capabilities"].is_object());
    assert_eq!(response["result"]["protocolVersion"], MCP_PROTOCOL_VERSION);
}

#[tokio::test]
async fn test_mcp_tools_list() {
    let mut client = McpClient::start();

    // Initialize
    let _ = client.initialize();
    client.send_initialized();

    // List tools
    let response = client.list_tools();

    // Verify response
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 2);
    let result = response.get("result").expect("Expected result field");
    let tools = result["tools"].as_array().expect("Expected tools array");
    assert!(tools.len() >= 2, "Expected at least 2 tools");

    let tool_names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
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

    // Verify tool schemas
    let search_tool = tools.iter().find(|t| t["name"] == "web_search").unwrap();
    assert!(search_tool["inputSchema"].is_object());
    assert!(search_tool["inputSchema"]["required"]
        .as_array()
        .map(|r| r.contains(&serde_json::Value::String("query".to_string())))
        .unwrap_or(false));

    let fetch_tool = tools.iter().find(|t| t["name"] == "web_fetch").unwrap();
    assert!(fetch_tool["inputSchema"].is_object());
    assert!(fetch_tool["inputSchema"]["required"]
        .as_array()
        .map(|r| r.contains(&serde_json::Value::String("url".to_string())))
        .unwrap_or(false));
}

#[tokio::test]
async fn test_mcp_call_unknown_tool_returns_error() {
    let mut client = McpClient::start();

    let _ = client.initialize();
    client.send_initialized();

    let response = client.call_tool("non_existent_tool", serde_json::json!({}));

    // Should return an error (isError: true or content with error)
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(response.get("error").is_some() || response["result"]["isError"] == true);
}

#[tokio::test]
async fn test_mcp_call_web_search_empty_query_returns_empty_results() {
    let mut client = McpClient::start();

    let _ = client.initialize();
    client.send_initialized();

    let response = client.call_tool("web_search", serde_json::json!({ "query": "" }));

    // Server accepts empty queries but returns empty results
    assert_eq!(response["jsonrpc"], "2.0");
    let result = response.get("result").expect("Expected result");
    if let Some(content) = result.get("content") {
        if let Some(text) = content.get(0).and_then(|c| c.get("text")) {
            let text_str = text.as_str().unwrap_or("");
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(text_str) {
                let results = parsed.get("results").and_then(|r| r.as_array());
                assert!(
                    results.is_some(),
                    "Expected 'results' field in search response"
                );
                // Results may be empty
                assert!(
                    results.unwrap().is_empty(),
                    "Expected empty results for empty query"
                );
            }
        }
    }
}

#[tokio::test]
async fn test_mcp_call_web_search_with_limit_too_high() {
    let mut client = McpClient::start();

    let _ = client.initialize();
    client.send_initialized();

    let response = client.call_tool(
        "web_search",
        serde_json::json!({ "query": "test", "limit": 999 }),
    );

    // Should either cap the limit or return an error
    assert_eq!(response["jsonrpc"], "2.0");
    let result = response.get("result");
    if let Some(r) = result {
        if let Some(is_error) = r.get("isError") {
            if is_error == true {
                return; // Error is acceptable
            }
        }
        // If no error, the server may have capped the limit - check response has query
        if let Some(content) = r.get("content") {
            if let Some(text) = content.get(0).and_then(|c| c.get("text")) {
                let text_str = text.as_str().unwrap_or("");
                // Should contain the query in some form
                assert!(
                    text_str.contains("test")
                        || text_str.contains("error")
                        || text_str.contains("limit")
                );
            }
        }
    } else {
        // Error at top level is also acceptable
        assert!(response.get("error").is_some(), "Expected result or error");
    }
}

#[tokio::test]
async fn test_mcp_call_web_fetch_invalid_url() {
    let mut client = McpClient::start();

    let _ = client.initialize();
    client.send_initialized();

    let response = client.call_tool("web_fetch", serde_json::json!({ "url": "not-a-valid-url" }));

    // Should return error for invalid URL
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(
        response.get("error").is_some() || response["result"]["isError"] == true,
        "Expected error for invalid URL, got: {}",
        response
    );
}

#[tokio::test]
async fn test_mcp_call_web_fetch_missing_url() {
    let mut client = McpClient::start();

    let _ = client.initialize();
    client.send_initialized();

    let response = client.call_tool("web_fetch", serde_json::json!({}));

    assert_eq!(response["jsonrpc"], "2.0");
    assert!(
        response.get("error").is_some() || response["result"]["isError"] == true,
        "Expected error for missing URL, got: {}",
        response
    );
}

#[tokio::test]
async fn test_consecutive_requests_maintain_session() {
    let mut client = McpClient::start();

    let _ = client.initialize();
    client.send_initialized();

    // Send multiple requests and verify session is maintained
    let resp1 = client.list_tools();
    assert_eq!(resp1["id"], 2);

    let resp2 = client.call_tool("web_search", serde_json::json!({ "query": "" }));
    assert_eq!(resp2["id"], 3);

    let resp3 = client.call_tool("web_fetch", serde_json::json!({ "url": "invalid" }));
    assert_eq!(resp3["id"], 3);
}

#[tokio::test]
async fn test_web_search_with_optional_params() {
    let mut client = McpClient::start();

    let _ = client.initialize();
    client.send_initialized();

    let response = client.call_tool(
        "web_search",
        serde_json::json!({
            "query": "rust programming language",
            "language": "en",
            "safe_search": "moderate",
            "region": "us-en"
        }),
    );

    assert_eq!(response["jsonrpc"], "2.0");
    assert!(
        response.get("result").is_some() || response.get("error").is_some(),
        "Expected result or error field"
    );

    if let Some(result) = response.get("result") {
        if let Some(content) = result.get("content") {
            if let Some(text) = content.get(0).and_then(|c| c.get("text")) {
                let text_str = text.as_str().unwrap_or("");
                let parsed: serde_json::Result<serde_json::Value> = serde_json::from_str(text_str);
                assert!(parsed.is_ok(), "Response should be valid JSON: {text_str}");
            }
        }
    }
}

#[tokio::test]
async fn test_web_search_with_safe_search_strict() {
    let mut client = McpClient::start();

    let _ = client.initialize();
    client.send_initialized();

    let response = client.call_tool(
        "web_search",
        serde_json::json!({
            "query": "test",
            "safe_search": "strict"
        }),
    );

    assert_eq!(response["jsonrpc"], "2.0");
    assert!(
        response.get("result").is_some() || response.get("error").is_some(),
        "Expected result or error field"
    );
}

#[tokio::test]
async fn test_web_search_with_safe_search_off() {
    let mut client = McpClient::start();

    let _ = client.initialize();
    client.send_initialized();

    let response = client.call_tool(
        "web_search",
        serde_json::json!({
            "query": "test",
            "safe_search": "off"
        }),
    );

    assert_eq!(response["jsonrpc"], "2.0");
    assert!(
        response.get("result").is_some() || response.get("error").is_some(),
        "Expected result or error field"
    );
}

#[tokio::test]
async fn test_web_fetch_include_raw_html() {
    let mut client = McpClient::start();

    let _ = client.initialize();
    client.send_initialized();

    let response = client.call_tool(
        "web_fetch",
        serde_json::json!({
            "url": "https://example.com",
            "include_raw_html": true
        }),
    );

    assert_eq!(response["jsonrpc"], "2.0");
    let result = response.get("result").expect("Expected result field");
    assert!(
        result["isError"] == false,
        "Expected successful result, got error: {}",
        result
    );

    if let Some(content) = result.get("content") {
        if let Some(text) = content.get(0).and_then(|c| c.get("text")) {
            let text_str = text.as_str().unwrap_or("");
            let parsed: serde_json::Result<serde_json::Value> = serde_json::from_str(text_str);
            assert!(parsed.is_ok(), "Response should be valid JSON");

            if let Ok(parsed_val) = parsed {
                assert!(
                    parsed_val.get("raw_html").is_some(),
                    "Expected raw_html field when include_raw_html=true"
                );
            }
        }
    }
}

#[tokio::test]
async fn test_web_fetch_extract_images() {
    let mut client = McpClient::start();

    let _ = client.initialize();
    client.send_initialized();

    let response = client.call_tool(
        "web_fetch",
        serde_json::json!({
            "url": "https://example.com",
            "extract_images": true
        }),
    );

    assert_eq!(response["jsonrpc"], "2.0");
    let result = response.get("result").expect("Expected result field");
    assert!(
        result["isError"] == false,
        "Expected successful result, got error: {}",
        result
    );

    if let Some(content) = result.get("content") {
        if let Some(text) = content.get(0).and_then(|c| c.get("text")) {
            let text_str = text.as_str().unwrap_or("");
            let parsed: serde_json::Result<serde_json::Value> = serde_json::from_str(text_str);
            assert!(parsed.is_ok(), "Response should be valid JSON");

            if let Ok(parsed_val) = parsed {
                assert!(
                    parsed_val.get("images").is_some(),
                    "Expected images field when extract_images=true"
                );
            }
        }
    }
}

#[tokio::test]
async fn test_web_fetch_extract_links_false() {
    let mut client = McpClient::start();

    let _ = client.initialize();
    client.send_initialized();

    let response = client.call_tool(
        "web_fetch",
        serde_json::json!({
            "url": "https://example.com",
            "extract_links": false
        }),
    );

    assert_eq!(response["jsonrpc"], "2.0");
    let result = response.get("result").expect("Expected result field");
    assert!(
        result["isError"] == false,
        "Expected successful result, got error: {}",
        result
    );

    if let Some(content) = result.get("content") {
        if let Some(text) = content.get(0).and_then(|c| c.get("text")) {
            let text_str = text.as_str().unwrap_or("");
            let parsed: serde_json::Result<serde_json::Value> = serde_json::from_str(text_str);
            assert!(parsed.is_ok(), "Response should be valid JSON");

            if let Ok(parsed_val) = parsed {
                // extract_links=false should omit the links field
                assert!(
                    parsed_val.get("links").is_none(),
                    "Expected no links field when extract_links=false"
                );
            }
        }
    }
}

#[tokio::test]
async fn test_web_fetch_with_user_agent() {
    let mut client = McpClient::start();

    let _ = client.initialize();
    client.send_initialized();

    let response = client.call_tool(
        "web_fetch",
        serde_json::json!({
            "url": "https://example.com",
            "user_agent": "IntegrationTest/1.0"
        }),
    );

    assert_eq!(response["jsonrpc"], "2.0");
    let result = response.get("result").expect("Expected result field");
    assert!(
        result["isError"] == false,
        "Expected successful result, got error: {}",
        result
    );
}

#[tokio::test]
async fn test_server_capabilities_in_initialize() {
    let mut client = McpClient::start();

    let response = client.initialize();

    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);

    let result = response.get("result").expect("Expected result field");

    // Server must advertise tools capability
    assert!(
        result["capabilities"].get("tools").is_some(),
        "Expected tools capability in initialize response"
    );

    // Verify serverInfo is present
    assert!(result["serverInfo"].get("name").is_some());
    assert!(result["serverInfo"].get("version").is_some());
}

#[tokio::test]
async fn test_json_rpc_error_format() {
    let mut client = McpClient::start();

    let _ = client.initialize();
    client.send_initialized();

    // Call a non-existent tool to trigger a JSON-RPC error
    let response = client.call_tool("tool_that_does_not_exist", serde_json::json!({}));

    // JSON-RPC error objects must have: jsonrpc, error (with code, message), optional id
    assert_eq!(response["jsonrpc"], "2.0");
    assert!(
        response.get("error").is_some(),
        "Expected 'error' field in JSON-RPC error response"
    );

    let error_obj = response["error"]
        .as_object()
        .expect("Expected error to be an object");
    assert!(
        error_obj.contains_key("code"),
        "Expected 'code' field in error object"
    );
    assert!(
        error_obj.contains_key("message"),
        "Expected 'message' field in error object"
    );

    let error_code: i64 = error_obj["code"]
        .as_i64()
        .expect("Error code should be an integer");
    assert!(
        error_code < 0,
        "JSON-RPC error codes must be negative, got {}",
        error_code
    );
}

#[tokio::test]
async fn test_session_persistence_multiple_tool_calls() {
    let mut client = McpClient::start();

    let _ = client.initialize();
    client.send_initialized();

    // Perform multiple tool calls sequentially to verify session is maintained
    let resp1 = client.call_tool("web_search", serde_json::json!({ "query": "" }));
    assert_eq!(resp1["id"], 3);

    let resp2 = client.list_tools();
    assert_eq!(resp2["id"], 2);

    let resp3 = client.call_tool("web_search", serde_json::json!({ "query": "" }));
    assert_eq!(resp3["id"], 3);

    let resp4 = client.call_tool("web_fetch", serde_json::json!({ "url": "invalid" }));
    assert_eq!(resp4["id"], 3);

    // All responses should be valid JSON-RPC
    for resp in [&resp1, &resp2, &resp3, &resp4] {
        assert_eq!(
            resp["jsonrpc"], "2.0",
            "All responses must have jsonrpc field"
        );
    }
}

#[tokio::test]
async fn test_tools_before_initialize_rejection() {
    let mut child = Command::new(assert_cmd::cargo::cargo_bin("gannet-mcp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server");

    // Do NOT call initialize() - try to call tools directly
    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    let req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "web_search",
            "arguments": { "query": "test" }
        }
    });
    stdin
        .write_all(format!("{}\n", serde_json::to_string(&req).unwrap()).as_bytes())
        .expect("Failed to write");
    stdin.flush().expect("Failed to flush");

    // The server should reject pre-initialize tool calls:
    // either by sending a JSON-RPC error response, or by closing the connection
    let stdout = child.stdout.as_mut().expect("Failed to open stdout");
    let mut reader = std::io::BufReader::new(stdout);
    let mut response_line = String::new();

    match reader.read_line(&mut response_line) {
        Ok(0) => {
            // Server closed connection - this is an acceptable rejection
        }
        Ok(_) => {
            // Server sent a response - it should be a JSON-RPC error
            let response: serde_json::Value =
                serde_json::from_str(&response_line).expect("Failed to parse response");
            assert_eq!(response["jsonrpc"], "2.0");
            assert!(
                response.get("error").is_some() || response["result"]["isError"] == true,
                "Expected rejection for tool call before initialize"
            );
        }
        Err(_) => {
            // Read error - acceptable as server may have closed
        }
    }

    let _ = child.kill();
    let _ = child.wait();
}

// --- Step 10: Config loading integration tests ---

#[tokio::test]
async fn test_binary_with_yaml_config_file() {
    let _guard = ENV_TEST_MUTEX.lock().unwrap();
    let tmp_dir = assert_fs::TempDir::new().expect("Failed to create temp dir");
    let config_path = tmp_dir.join("config.yaml");
    std::fs::write(
        &config_path,
        "search:\n  provider: brightdata\n  api_key: yaml-file-key\n  search_engine_id: yaml-engine-id\n  default_limit: 7\nserver:\n  port: 9090\n",
    )
    .expect("Failed to write YAML config");

    let mut child = Command::new(assert_cmd::cargo::cargo_bin("gannet-mcp"))
        .arg("--config")
        .arg(config_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server with YAML config");

    // Verify server starts by sending MCP initialize
    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    let init_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": MCP_PROTOCOL_VERSION,
            "capabilities": {},
            "clientInfo": { "name": "integration-test", "version": "1.0.0" }
        }
    });
    stdin
        .write_all(format!("{}\n", serde_json::to_string(&init_req).unwrap()).as_bytes())
        .expect("Failed to write");
    stdin.flush().expect("Failed to flush");

    let stdout = child.stdout.as_mut().expect("Failed to open stdout");
    let mut reader = std::io::BufReader::new(stdout);
    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .expect("Failed to read response");
    let response: serde_json::Value =
        serde_json::from_str(&response_line).expect("Failed to parse response");
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["result"]["protocolVersion"], MCP_PROTOCOL_VERSION);

    // Kill child before reading stderr to trigger EOF on the pipe
    let _ = child.kill();
    let stderr = child.stderr.as_mut().expect("Failed to open stderr");
    let mut err_reader = std::io::BufReader::new(stderr);
    let mut err_lines = Vec::new();
    let mut err_line = String::new();
    while err_reader.read_line(&mut err_line).unwrap() > 0 {
        err_lines.push(err_line.clone());
        err_line.clear();
    }
    let err_text: String = err_lines.join("\n");
    assert!(
        err_text.contains("brightdata"),
        "Expected 'brightdata' in stderr logs, got: {}",
        err_text
    );

    let _ = child.wait();
}

#[tokio::test]
async fn test_binary_with_json_config_file() {
    let _guard = ENV_TEST_MUTEX.lock().unwrap();
    let tmp_dir = assert_fs::TempDir::new().expect("Failed to create temp dir");
    let config_path = tmp_dir.join("config.json");
    std::fs::write(
        &config_path,
        r#"{"search":{"provider":"brightdata","api_key":"json-file-key","search_engine_id":"serp_api1","max_limit":30},"server":{"port":4000}}"#,
    )
    .expect("Failed to write JSON config");

    let mut child = Command::new(assert_cmd::cargo::cargo_bin("gannet-mcp"))
        .arg("--config")
        .arg(config_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server with JSON config");

    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    let init_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": MCP_PROTOCOL_VERSION,
            "capabilities": {},
            "clientInfo": { "name": "integration-test", "version": "1.0.0" }
        }
    });
    stdin
        .write_all(format!("{}\n", serde_json::to_string(&init_req).unwrap()).as_bytes())
        .expect("Failed to write");
    stdin.flush().expect("Failed to flush");

    let stdout = child.stdout.as_mut().expect("Failed to open stdout");
    let mut reader = std::io::BufReader::new(stdout);
    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .expect("Failed to read response");
    let response: serde_json::Value =
        serde_json::from_str(&response_line).expect("Failed to parse response");
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["result"]["protocolVersion"], MCP_PROTOCOL_VERSION);

    // Kill child before reading stderr to trigger EOF on the pipe
    let _ = child.kill();
    let stderr = child.stderr.as_mut().expect("Failed to open stderr");
    let mut err_reader = std::io::BufReader::new(stderr);
    let mut err_lines = Vec::new();
    let mut err_line = String::new();
    while err_reader.read_line(&mut err_line).unwrap() > 0 {
        err_lines.push(err_line.clone());
        err_line.clear();
    }
    let err_text: String = err_lines.join("\n");
    assert!(
        err_text.contains("brightdata"),
        "Expected 'brightdata' in stderr logs, got: {}",
        err_text
    );

    let _ = child.wait();
}

#[tokio::test]
async fn test_binary_with_env_var_search_provider() {
    let _guard = ENV_TEST_MUTEX.lock().unwrap();
    // Clear any existing MCP__ env vars to avoid interference
    for var in &[
        "MCP__SEARCH__PROVIDER",
        "MCP__SEARCH__API_KEY",
        "MCP__SERVER__HOST",
        "MCP__SERVER__PORT",
    ] {
        env::remove_var(var);
    }
    env::set_var("MCP__SEARCH__PROVIDER", "duckduckgo");

    let mut child = Command::new(assert_cmd::cargo::cargo_bin("gannet-mcp"))
        .env("MCP__SEARCH__PROVIDER", "duckduckgo")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server with env var config");

    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    let init_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": MCP_PROTOCOL_VERSION,
            "capabilities": {},
            "clientInfo": { "name": "integration-test", "version": "1.0.0" }
        }
    });
    stdin
        .write_all(format!("{}\n", serde_json::to_string(&init_req).unwrap()).as_bytes())
        .expect("Failed to write");
    stdin.flush().expect("Failed to flush");

    let stdout = child.stdout.as_mut().expect("Failed to open stdout");
    let mut reader = std::io::BufReader::new(stdout);
    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .expect("Failed to read response");
    let response: serde_json::Value =
        serde_json::from_str(&response_line).expect("Failed to parse response");
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["result"]["protocolVersion"], MCP_PROTOCOL_VERSION);

    // Kill child before reading stderr to trigger EOF on the pipe
    let _ = child.kill();
    let stderr = child.stderr.as_mut().expect("Failed to open stderr");
    let mut err_reader = std::io::BufReader::new(stderr);
    let mut err_lines = Vec::new();
    let mut err_line = String::new();
    while err_reader.read_line(&mut err_line).unwrap() > 0 {
        err_lines.push(err_line.clone());
        err_line.clear();
    }
    // Server should log startup info containing the duckduckgo provider
    let err_text: String = err_lines.join("\n");
    assert!(
        err_text.contains("duckduckgo"),
        "Expected 'duckduckgo' in stderr logs, got: {}",
        err_text
    );

    env::remove_var("MCP__SEARCH__PROVIDER");
    let _ = child.wait();
}

#[tokio::test]
async fn test_binary_with_cli_provider_and_api_key_overrides() {
    let _guard = ENV_TEST_MUTEX.lock().unwrap();
    for var in &[
        "MCP__SEARCH__PROVIDER",
        "MCP__SEARCH__API_KEY",
        "MCP__SERVER__HOST",
        "MCP__SERVER__PORT",
    ] {
        env::remove_var(var);
    }

    let mut child = Command::new(assert_cmd::cargo::cargo_bin("gannet-mcp"))
        .args(["--provider", "duckduckgo"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server with CLI overrides");

    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    let init_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": MCP_PROTOCOL_VERSION,
            "capabilities": {},
            "clientInfo": { "name": "integration-test", "version": "1.0.0" }
        }
    });
    stdin
        .write_all(format!("{}\n", serde_json::to_string(&init_req).unwrap()).as_bytes())
        .expect("Failed to write");
    stdin.flush().expect("Failed to flush");

    let stdout = child.stdout.as_mut().expect("Failed to open stdout");
    let mut reader = std::io::BufReader::new(stdout);
    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .expect("Failed to read response");
    let response: serde_json::Value =
        serde_json::from_str(&response_line).expect("Failed to parse response");
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["result"]["protocolVersion"], MCP_PROTOCOL_VERSION);

    // Kill child before reading stderr to trigger EOF on the pipe
    let _ = child.kill();
    let stderr = child.stderr.as_mut().expect("Failed to open stderr");
    let mut err_reader = std::io::BufReader::new(stderr);
    let mut err_lines = Vec::new();
    let mut err_line = String::new();
    while err_reader.read_line(&mut err_line).unwrap() > 0 {
        err_lines.push(err_line.clone());
        err_line.clear();
    }
    let err_text: String = err_lines.join("\n");
    assert!(
        err_text.contains("duckduckgo"),
        "Expected 'duckduckgo' in stderr logs, got: {}",
        err_text
    );

    env::remove_var("MCP__SEARCH__PROVIDER");
    env::remove_var("MCP__SEARCH__API_KEY");
    let _ = child.wait();
}

#[tokio::test]
async fn test_binary_with_no_args_uses_defaults() {
    let _guard = ENV_TEST_MUTEX.lock().unwrap();
    for var in &[
        "MCP__SEARCH__PROVIDER",
        "MCP__SEARCH__API_KEY",
        "MCP__SERVER__HOST",
        "MCP__SERVER__PORT",
    ] {
        env::remove_var(var);
    }

    let mut child = Command::new(assert_cmd::cargo::cargo_bin("gannet-mcp"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server with no args");

    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    let init_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": MCP_PROTOCOL_VERSION,
            "capabilities": {},
            "clientInfo": { "name": "integration-test", "version": "1.0.0" }
        }
    });
    stdin
        .write_all(format!("{}\n", serde_json::to_string(&init_req).unwrap()).as_bytes())
        .expect("Failed to write");
    stdin.flush().expect("Failed to flush");

    let stdout = child.stdout.as_mut().expect("Failed to open stdout");
    let mut reader = std::io::BufReader::new(stdout);
    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .expect("Failed to read response");
    let response: serde_json::Value =
        serde_json::from_str(&response_line).expect("Failed to parse response");
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["result"]["protocolVersion"], MCP_PROTOCOL_VERSION);

    // Kill child before reading stderr to trigger EOF on the pipe
    let _ = child.kill();
    let stderr = child.stderr.as_mut().expect("Failed to open stderr");
    let mut err_reader = std::io::BufReader::new(stderr);
    let mut err_lines = Vec::new();
    let mut err_line = String::new();
    while err_reader.read_line(&mut err_line).unwrap() > 0 {
        err_lines.push(err_line.clone());
        err_line.clear();
    }
    let err_text: String = err_lines.join("\n");
    assert!(
        err_text.contains("duckduckgo"),
        "Expected default provider 'duckduckgo' in stderr logs, got: {}",
        err_text
    );

    env::remove_var("MCP__SEARCH__PROVIDER");
    env::remove_var("MCP__SEARCH__API_KEY");
    let _ = child.wait();
}

#[tokio::test]
async fn test_binary_with_log_level_debug_log_format_json() {
    let _guard = ENV_TEST_MUTEX.lock().unwrap();
    for var in &[
        "MCP__SEARCH__PROVIDER",
        "MCP__SEARCH__API_KEY",
        "MCP__SERVER__HOST",
        "MCP__SERVER__PORT",
    ] {
        env::remove_var(var);
    }

    let mut child = Command::new(assert_cmd::cargo::cargo_bin("gannet-mcp"))
        .args(["--log-level", "debug", "--log-format", "json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start server with debug/json logging");

    // Verify server starts without error
    let stdin = child.stdin.as_mut().expect("Failed to open stdin");
    let init_req = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": MCP_PROTOCOL_VERSION,
            "capabilities": {},
            "clientInfo": { "name": "integration-test", "version": "1.0.0" }
        }
    });
    stdin
        .write_all(format!("{}\n", serde_json::to_string(&init_req).unwrap()).as_bytes())
        .expect("Failed to write");
    stdin.flush().expect("Failed to flush");

    let stdout = child.stdout.as_mut().expect("Failed to open stdout");
    let mut reader = std::io::BufReader::new(stdout);
    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .expect("Failed to read response");
    let response: serde_json::Value =
        serde_json::from_str(&response_line).expect("Failed to parse response");
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["result"]["protocolVersion"], MCP_PROTOCOL_VERSION);

    // Kill child before reading stderr to trigger EOF on the pipe
    let _ = child.kill();
    let stderr = child.stderr.as_mut().expect("Failed to open stderr");
    let mut err_reader = std::io::BufReader::new(stderr);
    let mut err_lines = Vec::new();
    let mut err_line = String::new();
    while err_reader.read_line(&mut err_line).unwrap() > 0 {
        err_lines.push(err_line.clone());
        err_line.clear();
    }
    let err_text: String = err_lines.join("\n");
    // JSON log format should produce lines starting with '{'
    assert!(
        err_text.contains("{"),
        "Expected JSON-formatted log lines (starting with '{{') in stderr, got: {}",
        err_text
    );

    env::remove_var("MCP__SEARCH__PROVIDER");
    env::remove_var("MCP__SEARCH__API_KEY");
    let _ = child.wait();
}
