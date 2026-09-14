//! `gannet-mcp mcp-config` — emit MCP client JSON snippets (Phase 1b).
//!
//! Prints the JSON snippet for one MCP client (STDIO by default, HTTP with
//! `--http`) and optionally writes it to the client's known config file.
//!
//! Secrets are conveyed via an `env` block carrying the `MCP__…` variables
//! copied from the current process environment — never via `argv`.
//!
//! Client → write-path table (used by `--write`):
//!
//! | Client            | Top-level JSON key | Write path (`--write`)              |
//! |-------------------|--------------------|---------------------------------------|
//! | Claude            | `mcpServers`       | `~/.claude.json` (all OSes)           |
//! | Cursor            | `mcpServers`       | `~/.cursor/mcp.json` (all OSes)       |
//! | Opencode          | `mcp`              | Linux: `$XDG_CONFIG_HOME/opencode/opencode.json` (fallback `~/.config/opencode/opencode.json`); macOS: `~/Library/Application Support/opencode/opencode.json`; Windows: `%APPDATA%/opencode/opencode.json` |
//! | Zed               | `context_servers`  | Linux: `$XDG_CONFIG_HOME/zed/settings.json` (fallback `~/.config/zed/settings.json`); macOS: `~/Library/Application Support/Zed/settings.json` (fallback `~/.config/zed/settings.json`); Windows: `%APPDATA%/Zed/settings.json` — merged under `context_servers`, other settings preserved |
//! | Continue          | `mcpServers`       | `~/.continue/config.json` (all OSes) — merged under `mcpServers`; pre-existing array-form `mcpServers` is merged by `name` |
//! | Copilot (VS Code) | `servers`          | User-level file **only if it already exists**, else fall back to `--print`: Linux: `$XDG_CONFIG_HOME/Code/User/mcp.json` (fallback `~/.config/Code/User/mcp.json`, then `~/.vscode/mcp.json`); macOS: `~/Library/Application Support/Code/User/mcp.json`; Windows: `%APPDATA%/Code/User/mcp.json`. A workspace `<workspace>/.vscode/mcp.json` is ambiguous from the CLI so it is never written automatically. |
//!
//! Schema compromises (kept pragmatic; each client's conventional top-level
//! key is used, but the server entry shape is uniform):
//!
//! - STDIO entries are always `{"command": <exe>, "args": [], "env": {...}}`.
//!   Native deviations, deliberately not followed so every client gets the
//!   same shape: opencode wants `"type": "local"` with `command` as an array
//!   and `environment` instead of `env`; Zed (recent versions) nests the
//!   command as `{"command": {"path": …, "args": …, "env": …}}`; Continue's
//!   classic form is an array of `{"name": …, "command": …}` objects.
//! - HTTP entries are `{"url": …, "env": {...}}`, plus a type discriminator
//!   where the client schema uses one: opencode gets `"type": "remote"`
//!   (its documented discriminator, not `"http"`), Copilot/VS Code gets
//!   `"type": "http"`. Claude/Cursor/Zed/Continue get no `type` key.
//! - Continue `--write` merges into either map-form or array-form
//!   `mcpServers`; newly created files use the map form.
//! - The binary path comes from `std::env::current_exe()`. On Windows it is
//!   emitted as-is (backslashes preserved, no separator normalisation);
//!   elsewhere it is emitted as the OS string. If `current_exe()` fails,
//!   the fallback command is the literal `gannet-mcp` (expected on `PATH`).

use anyhow::{Context, Result};
use clap::{Args, ValueEnum};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
#[derive(Args, Debug)]
pub struct McpConfigArgs {
    #[arg(long, value_enum)]
    pub client: Option<McpClient>,
    #[arg(long)]
    pub stdio: bool,
    #[arg(long)]
    pub http: bool,
    #[arg(long)]
    pub print: bool,
    #[arg(long)]
    pub write: bool,
}
#[derive(Clone, Debug, ValueEnum)]
pub enum McpClient {
    Claude,
    Cursor,
    Zed,
    Opencode,
    Continue,
    Copilot,
}

impl std::fmt::Display for McpClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(client_name(self))
    }
}

/// Short lowercase name used in log/status messages.
pub fn client_name(client: &McpClient) -> &'static str {
    match client {
        McpClient::Claude => "claude",
        McpClient::Cursor => "cursor",
        McpClient::Zed => "zed",
        McpClient::Opencode => "opencode",
        McpClient::Continue => "continue",
        McpClient::Copilot => "copilot",
    }
}

/// Conventional top-level JSON key for each client's config file.
pub fn top_key(client: &McpClient) -> &'static str {
    match client {
        McpClient::Claude | McpClient::Cursor | McpClient::Continue => "mcpServers",
        McpClient::Opencode => "mcp",
        McpClient::Zed => "context_servers",
        McpClient::Copilot => "servers",
    }
}

/// Default URL used for `--http` snippets.
pub fn default_http_url() -> String {
    "http://127.0.0.1:8080/mcp".to_string()
}

/// Client selection: explicit `--client`, otherwise Opencode.
pub fn resolve_client(args: &McpConfigArgs) -> McpClient {
    args.client.clone().unwrap_or(McpClient::Opencode)
}

/// Transport selection: `--http` alone means HTTP; both/neither means STDIO.
pub fn resolve_use_http(args: &McpConfigArgs) -> bool {
    args.http && !args.stdio
}

/// `--print` is the default when `--write` is absent; both flags mean both.
pub fn should_print(args: &McpConfigArgs) -> bool {
    args.print || !args.write
}

/// Whether `--write` was requested.
pub fn should_write(args: &McpConfigArgs) -> bool {
    args.write
}

/// Keep only `MCP__…` variables (pure helper; hermetic under test).
pub fn filter_mcp_env(
    vars: impl IntoIterator<Item = (String, String)>,
) -> BTreeMap<String, String> {
    vars.into_iter()
        .filter(|(k, _)| k.starts_with("MCP__"))
        .collect()
}

/// Copy `MCP__…` keys from the current process environment.
pub fn collect_mcp_env() -> BTreeMap<String, String> {
    filter_mcp_env(std::env::vars())
}

/// Binary path for snippets: `current_exe()` as-is (Windows backslashes are
/// preserved — no separator normalisation anywhere), else the OS string.
/// Falls back to `gannet-mcp` when `current_exe()` fails.
pub fn current_exe_string() -> String {
    match std::env::current_exe() {
        Ok(p) => p.to_string_lossy().into_owned(),
        Err(_) => "gannet-mcp".to_string(),
    }
}

/// Render the full client config document as pretty JSON (no trailing
/// newline; callers add one via `println!` / file write).
pub fn render(
    client: &McpClient,
    use_http: bool,
    exe_path: &str,
    env_vars: &BTreeMap<String, String>,
    http_url: &str,
) -> String {
    let env_obj: serde_json::Map<String, serde_json::Value> = env_vars
        .iter()
        .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
        .collect();
    let entry = if use_http {
        match client {
            McpClient::Opencode => {
                serde_json::json!({"type": "remote", "url": http_url, "env": env_obj})
            }
            McpClient::Copilot => {
                serde_json::json!({"type": "http", "url": http_url, "env": env_obj})
            }
            _ => serde_json::json!({"url": http_url, "env": env_obj}),
        }
    } else {
        serde_json::json!({"command": exe_path, "args": [], "env": env_obj})
    };
    let mut servers = serde_json::Map::new();
    servers.insert("gannet-mcp".to_string(), entry);
    let mut top = serde_json::Map::new();
    top.insert(
        top_key(client).to_string(),
        serde_json::Value::Object(servers),
    );
    serde_json::to_string_pretty(&serde_json::Value::Object(top))
        .expect("JSON serialization of a Value cannot fail")
}

/// Resolve the `--write` destination for a client.
///
/// Returns `None` when no sensible path exists: for Copilot when no
/// user-level file exists yet (workspace path is ambiguous from the CLI),
/// or when the home directory cannot be determined.
pub fn config_path_for(client: &McpClient) -> Option<PathBuf> {
    let home = resolve_home();
    let home_ref = home.as_deref();
    let xdg = resolve_xdg_config(home_ref);
    let appdata = resolve_appdata(home_ref);
    config_path_from_parts(client, home_ref, xdg.as_deref(), appdata.as_deref(), &|p| {
        p.exists()
    })
}

fn resolve_home() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("USERPROFILE").map(PathBuf::from))
}

fn resolve_xdg_config(home: Option<&Path>) -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from) {
        if !dir.as_os_str().is_empty() {
            return Some(dir);
        }
    }
    home.map(|h| h.join(".config"))
}

fn resolve_appdata(home: Option<&Path>) -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("APPDATA").map(PathBuf::from) {
        if !dir.as_os_str().is_empty() {
            return Some(dir);
        }
    }
    home.map(|h| h.join("AppData").join("Roaming"))
}

/// Explicit-input core of [`config_path_for`] so unit tests can pass fake
/// base directories and a stub existence check instead of touching `$HOME`.
fn config_path_from_parts(
    client: &McpClient,
    home: Option<&Path>,
    xdg_config: Option<&Path>,
    appdata: Option<&Path>,
    exists: &dyn Fn(&Path) -> bool,
) -> Option<PathBuf> {
    match client {
        McpClient::Claude => home.map(|h| h.join(".claude.json")),
        McpClient::Cursor => home.map(|h| h.join(".cursor").join("mcp.json")),
        McpClient::Opencode => {
            if cfg!(windows) {
                appdata.map(|a| a.join("opencode").join("opencode.json"))
            } else if cfg!(target_os = "macos") {
                home.map(|h| {
                    h.join("Library")
                        .join("Application Support")
                        .join("opencode")
                        .join("opencode.json")
                })
            } else {
                xdg_config
                    .map(|x| x.join("opencode").join("opencode.json"))
                    .or_else(|| {
                        home.map(|h| h.join(".config").join("opencode").join("opencode.json"))
                    })
            }
        }
        McpClient::Zed => {
            if cfg!(windows) {
                appdata.map(|a| a.join("Zed").join("settings.json"))
            } else if cfg!(target_os = "macos") {
                // Prefer the app-support location; fall back to ~/.config/zed.
                home.map(|h| {
                    h.join("Library")
                        .join("Application Support")
                        .join("Zed")
                        .join("settings.json")
                })
            } else {
                xdg_config
                    .map(|x| x.join("zed").join("settings.json"))
                    .or_else(|| home.map(|h| h.join(".config").join("zed").join("settings.json")))
            }
        }
        McpClient::Continue => home.map(|h| h.join(".continue").join("config.json")),
        McpClient::Copilot => {
            let mut candidates: Vec<PathBuf> = Vec::new();
            if cfg!(windows) {
                if let Some(a) = appdata {
                    candidates.push(a.join("Code").join("User").join("mcp.json"));
                }
            } else if cfg!(target_os = "macos") {
                if let Some(h) = home {
                    candidates.push(
                        h.join("Library")
                            .join("Application Support")
                            .join("Code")
                            .join("User")
                            .join("mcp.json"),
                    );
                }
            } else {
                if let Some(x) = xdg_config {
                    candidates.push(x.join("Code").join("User").join("mcp.json"));
                }
                if let Some(h) = home {
                    let fallback = h.join(".config").join("Code").join("User").join("mcp.json");
                    if !candidates.contains(&fallback) {
                        candidates.push(fallback);
                    }
                    candidates.push(h.join(".vscode").join("mcp.json"));
                }
            }
            candidates.into_iter().find(|p| exists(p))
        }
    }
}

/// Convert a map-form server entry into Continue's classic array item form.
fn entry_to_continue_item(entry: &serde_json::Value) -> serde_json::Value {
    let mut item = serde_json::Map::new();
    item.insert(
        "name".to_string(),
        serde_json::Value::String("gannet-mcp".to_string()),
    );
    if let Some(obj) = entry.as_object() {
        for (k, v) in obj {
            item.insert(k.clone(), v.clone());
        }
    }
    serde_json::Value::Object(item)
}

/// Merge the `gannet-mcp` entry from `snippet` into `existing` (which must be
/// a JSON object). Returns `true` when a wrongly-typed section was replaced
/// (caller warns on stderr).
fn merge_snippet(
    existing: &mut serde_json::Value,
    client: &McpClient,
    snippet: &serde_json::Value,
) -> bool {
    let key = top_key(client);
    let Some(entry) = snippet.get(key).and_then(|v| v.get("gannet-mcp")).cloned() else {
        return false;
    };
    let Some(obj) = existing.as_object_mut() else {
        return false;
    };
    if matches!(client, McpClient::Continue) {
        match obj.get_mut(key) {
            Some(serde_json::Value::Array(arr)) => {
                let item = entry_to_continue_item(&entry);
                if let Some(pos) = arr
                    .iter()
                    .position(|e| e.get("name").and_then(|n| n.as_str()) == Some("gannet-mcp"))
                {
                    arr[pos] = item;
                } else {
                    arr.push(item);
                }
                false
            }
            Some(serde_json::Value::Object(map)) => {
                map.insert("gannet-mcp".to_string(), entry);
                false
            }
            None => {
                obj.insert(key.to_string(), snippet[key].clone());
                false
            }
            Some(_) => {
                obj.insert(key.to_string(), snippet[key].clone());
                true
            }
        }
    } else {
        match obj.get_mut(key) {
            Some(serde_json::Value::Object(map)) => {
                map.insert("gannet-mcp".to_string(), entry);
                false
            }
            None => {
                obj.insert(key.to_string(), snippet[key].clone());
                false
            }
            Some(_) => {
                obj.insert(key.to_string(), snippet[key].clone());
                true
            }
        }
    }
}

/// Write `snippet` to an explicit `path`, creating parent directories.
///
/// Existing files that parse as JSON objects are merged (the `gannet-mcp`
/// entry is inserted under the client's top-level key, everything else is
/// preserved); files that don't exist, don't parse, or aren't objects are
/// overwritten (with a stderr warning in the latter two cases).
pub fn write_to(path: &Path, client: &McpClient, snippet: &str) -> Result<()> {
    let snippet_value: serde_json::Value =
        serde_json::from_str(snippet).context("rendered snippet is not valid JSON")?;
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create directory {}", parent.display()))?;
        }
    }
    let body = format!("{}\n", snippet.trim_end());
    if !path.exists() {
        std::fs::write(path, body)
            .with_context(|| format!("failed to write {}", path.display()))?;
        return Ok(());
    }
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let mut existing: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(_) => {
            eprintln!(
                "gannet-mcp: warning: existing config at {} did not parse as JSON; overwriting",
                path.display()
            );
            std::fs::write(path, body)
                .with_context(|| format!("failed to write {}", path.display()))?;
            return Ok(());
        }
    };
    if !existing.is_object() {
        eprintln!(
            "gannet-mcp: warning: existing config at {} is not a JSON object; overwriting",
            path.display()
        );
        std::fs::write(path, body)
            .with_context(|| format!("failed to write {}", path.display()))?;
        return Ok(());
    }
    if merge_snippet(&mut existing, client, &snippet_value) {
        eprintln!(
                "gannet-mcp: warning: existing {:?} section at {} had an unexpected shape; replacing it",
            top_key(client),
            path.display()
        );
    }
    let out = serde_json::to_string_pretty(&existing).expect("JSON serialization cannot fail");
    std::fs::write(path, format!("{out}\n"))
        .with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

pub fn run_mcp_config(args: &McpConfigArgs) -> Result<()> {
    let client = resolve_client(args);
    let use_http = resolve_use_http(args);
    let exe = current_exe_string();
    let env = collect_mcp_env();
    let url = default_http_url();
    let snippet = render(&client, use_http, &exe, &env, &url);

    let do_print = should_print(args);
    let do_write = should_write(args);

    if do_print {
        println!("{snippet}");
    }
    if do_write {
        match config_path_for(&client) {
            Some(path) => {
                write_to(&path, &client, &snippet)?;
                eprintln!("gannet-mcp: wrote {client} config to {}", path.display());
            }
            None => {
                if matches!(client, McpClient::Copilot) {
                    eprintln!(
                        "gannet-mcp: no existing Copilot/VS Code user-level mcp.json found; \
                         a workspace .vscode/mcp.json is ambiguous from the CLI, \
                         so printing the snippet instead"
                    );
                } else {
                    eprintln!(
                        "gannet-mcp: could not determine config path for {client}; printing instead"
                    );
                }
                if !do_print {
                    println!("{snippet}");
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn args_for(
        client: Option<McpClient>,
        stdio: bool,
        http: bool,
        print: bool,
        write: bool,
    ) -> McpConfigArgs {
        McpConfigArgs {
            client,
            stdio,
            http,
            print,
            write,
        }
    }

    fn sample_env() -> BTreeMap<String, String> {
        BTreeMap::from([
            ("MCP__SEARCH__PROVIDER".to_string(), "serper".to_string()),
            ("MCP__SEARCH__API_KEY".to_string(), "s3cret".to_string()),
        ])
    }

    /// Unique scratch dir under the OS temp dir (never `$HOME`, no registry).
    fn temp_dir_unique(tag: &str) -> PathBuf {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let n = NEXT.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("gannet-mcp-{tag}-{}-{n}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn stdio_claude_shape() {
        let out = render(
            &McpClient::Claude,
            false,
            "/usr/bin/gannet-mcp",
            &sample_env(),
            &default_http_url(),
        );
        let v: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        let entry = &v["mcpServers"]["gannet-mcp"];
        assert_eq!(entry["command"], "/usr/bin/gannet-mcp");
        assert_eq!(entry["args"], serde_json::json!([]));
        assert_eq!(entry["env"]["MCP__SEARCH__PROVIDER"], "serper");
        assert_eq!(entry["env"]["MCP__SEARCH__API_KEY"], "s3cret");
        assert!(entry.get("url").is_none());
        // Secrets live only in env, never in argv.
        assert!(entry["args"].as_array().unwrap().is_empty());
    }

    #[test]
    fn http_claude_shape() {
        let url = default_http_url();
        let out = render(
            &McpClient::Claude,
            true,
            "/usr/bin/gannet-mcp",
            &sample_env(),
            &url,
        );
        let v: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        let entry = &v["mcpServers"]["gannet-mcp"];
        assert_eq!(entry["url"], url);
        assert!(entry.get("command").is_none());
        assert_eq!(entry["env"]["MCP__SEARCH__API_KEY"], "s3cret");
    }

    #[test]
    fn stdio_opencode_shape_uses_mcp_key() {
        let out = render(
            &McpClient::Opencode,
            false,
            "/usr/bin/gannet-mcp",
            &sample_env(),
            &default_http_url(),
        );
        let v: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        assert!(v.get("mcpServers").is_none());
        let entry = &v["mcp"]["gannet-mcp"];
        assert_eq!(entry["command"], "/usr/bin/gannet-mcp");
        assert_eq!(entry["args"], serde_json::json!([]));
        assert_eq!(entry["env"]["MCP__SEARCH__PROVIDER"], "serper");
    }

    #[test]
    fn http_opencode_shape_uses_remote_type() {
        let url = default_http_url();
        let out = render(
            &McpClient::Opencode,
            true,
            "/usr/bin/gannet-mcp",
            &sample_env(),
            &url,
        );
        let v: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        let entry = &v["mcp"]["gannet-mcp"];
        assert_eq!(entry["url"], url);
        assert_eq!(entry["type"], "remote");
    }

    #[test]
    fn http_copilot_shape_uses_http_type_and_servers_key() {
        let url = default_http_url();
        let out = render(
            &McpClient::Copilot,
            true,
            "/usr/bin/gannet-mcp",
            &sample_env(),
            &url,
        );
        let v: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        let entry = &v["servers"]["gannet-mcp"];
        assert_eq!(entry["url"], url);
        assert_eq!(entry["type"], "http");
    }

    #[test]
    fn stdio_cursor_and_zed_keys() {
        let env = sample_env();
        let url = default_http_url();
        let cursor: serde_json::Value =
            serde_json::from_str(&render(&McpClient::Cursor, false, "/bin/g", &env, &url)).unwrap();
        assert!(cursor["mcpServers"]["gannet-mcp"].is_object());
        let zed: serde_json::Value =
            serde_json::from_str(&render(&McpClient::Zed, false, "/bin/g", &env, &url)).unwrap();
        assert!(zed["context_servers"]["gannet-mcp"].is_object());
    }

    #[test]
    fn transport_defaulting() {
        // Neither flag → STDIO.
        assert!(!resolve_use_http(&args_for(
            None, false, false, false, false
        )));
        // Both flags → STDIO.
        assert!(!resolve_use_http(&args_for(None, true, true, false, false)));
        // --http alone → HTTP.
        assert!(resolve_use_http(&args_for(None, false, true, false, false)));
        // --stdio alone → STDIO.
        assert!(!resolve_use_http(&args_for(
            None, true, false, false, false
        )));
    }

    #[test]
    fn client_defaulting_and_print_write_routing() {
        assert!(matches!(
            resolve_client(&args_for(None, false, false, false, false)),
            McpClient::Opencode
        ));
        assert!(matches!(
            resolve_client(&args_for(
                Some(McpClient::Claude),
                false,
                false,
                false,
                false
            )),
            McpClient::Claude
        ));
        // --print default when --write absent.
        assert!(should_print(&args_for(None, false, false, false, false)));
        assert!(!should_write(&args_for(None, false, false, false, false)));
        // --write alone: no print.
        assert!(!should_print(&args_for(None, false, false, false, true)));
        assert!(should_write(&args_for(None, false, false, false, true)));
        // Both: do both.
        assert!(should_print(&args_for(None, false, false, true, true)));
        assert!(should_write(&args_for(None, false, false, true, true)));
    }

    #[test]
    fn env_filter_keeps_only_mcp_prefix() {
        let got = filter_mcp_env([
            ("MCP__A".to_string(), "1".to_string()),
            ("OTHER".to_string(), "2".to_string()),
            ("MCP__B__C".to_string(), "3".to_string()),
        ]);
        assert_eq!(got.len(), 2);
        assert_eq!(got["MCP__A"], "1");
        assert_eq!(got["MCP__B__C"], "3");
    }

    #[test]
    fn write_new_file_creates_parents() {
        let dir = temp_dir_unique("new");
        let path = dir.join("sub").join("dir").join("mcp.json");
        let snippet = render(
            &McpClient::Cursor,
            false,
            "/usr/bin/gannet-mcp",
            &sample_env(),
            &default_http_url(),
        );
        write_to(&path, &McpClient::Cursor, &snippet).unwrap();
        let raw = std::fs::read_to_string(&path).unwrap();
        let v: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(
            v["mcpServers"]["gannet-mcp"]["command"],
            "/usr/bin/gannet-mcp"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn write_merges_preserving_other_servers_and_keys() {
        let dir = temp_dir_unique("merge");
        let path = dir.join("mcp.json");
        std::fs::write(
            &path,
            r#"{"mcpServers": {"other": {"command": "x", "args": [], "env": {}}}, "keep": 1}"#,
        )
        .unwrap();
        let snippet = render(
            &McpClient::Cursor,
            false,
            "/usr/bin/gannet-mcp",
            &sample_env(),
            &default_http_url(),
        );
        write_to(&path, &McpClient::Cursor, &snippet).unwrap();
        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(v["keep"], 1);
        assert_eq!(v["mcpServers"]["other"]["command"], "x");
        assert_eq!(
            v["mcpServers"]["gannet-mcp"]["command"],
            "/usr/bin/gannet-mcp"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn write_overwrites_invalid_json() {
        let dir = temp_dir_unique("invalid");
        let path = dir.join("mcp.json");
        std::fs::write(&path, "not json {{{").unwrap();
        let snippet = render(
            &McpClient::Claude,
            false,
            "/usr/bin/gannet-mcp",
            &sample_env(),
            &default_http_url(),
        );
        write_to(&path, &McpClient::Claude, &snippet).unwrap();
        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(
            v["mcpServers"]["gannet-mcp"]["command"],
            "/usr/bin/gannet-mcp"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn write_zed_preserves_unrelated_settings() {
        let dir = temp_dir_unique("zed");
        let path = dir.join("settings.json");
        std::fs::write(&path, r#"{"theme": "dark", "context_servers": {}}"#).unwrap();
        let snippet = render(
            &McpClient::Zed,
            false,
            "/usr/bin/gannet-mcp",
            &sample_env(),
            &default_http_url(),
        );
        write_to(&path, &McpClient::Zed, &snippet).unwrap();
        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(v["theme"], "dark");
        assert_eq!(
            v["context_servers"]["gannet-mcp"]["command"],
            "/usr/bin/gannet-mcp"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn write_continue_merges_array_form_by_name() {
        let dir = temp_dir_unique("continue");
        let path = dir.join("config.json");
        std::fs::write(
            &path,
            r#"{"mcpServers": [{"name": "other", "command": "x"}, {"name": "gannet-mcp", "command": "old"}]}"#,
        )
        .unwrap();
        let snippet = render(
            &McpClient::Continue,
            false,
            "/usr/bin/gannet-mcp",
            &sample_env(),
            &default_http_url(),
        );
        write_to(&path, &McpClient::Continue, &snippet).unwrap();
        let v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        let arr = v["mcpServers"].as_array().unwrap();
        assert_eq!(arr.len(), 2);
        let ours = arr.iter().find(|e| e["name"] == "gannet-mcp").unwrap();
        assert_eq!(ours["command"], "/usr/bin/gannet-mcp");
        assert!(arr.iter().any(|e| e["name"] == "other"));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn copilot_path_used_only_when_it_exists() {
        let dir = temp_dir_unique("copilot");
        let home = dir.join("home");
        let xdg = dir.join("xdg");
        std::fs::create_dir_all(&home).unwrap();
        std::fs::create_dir_all(&xdg).unwrap();
        let target = xdg.join("Code").join("User").join("mcp.json");
        // Nothing exists → None (caller falls back to --print).
        assert!(config_path_from_parts(
            &McpClient::Copilot,
            Some(&home),
            Some(&xdg),
            None,
            &|_| false
        )
        .is_none());
        // Pretend the user-level file exists → that path is returned.
        let got =
            config_path_from_parts(&McpClient::Copilot, Some(&home), Some(&xdg), None, &|p| {
                p == target
            })
            .expect("existing copilot path");
        assert_eq!(got, target);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn static_paths_have_expected_suffixes() {
        let dir = temp_dir_unique("suffix");
        let home = dir.join("home");
        let xdg = dir.join("xdg");
        let no = &|_: &Path| false;
        let claude =
            config_path_from_parts(&McpClient::Claude, Some(&home), Some(&xdg), None, no).unwrap();
        assert!(claude.ends_with(".claude.json"));
        let cursor =
            config_path_from_parts(&McpClient::Cursor, Some(&home), Some(&xdg), None, no).unwrap();
        assert_eq!(
            cursor.strip_prefix(&home).unwrap(),
            Path::new(".cursor/mcp.json")
        );
        let cont = config_path_from_parts(&McpClient::Continue, Some(&home), Some(&xdg), None, no)
            .unwrap();
        assert_eq!(
            cont.strip_prefix(&home).unwrap(),
            Path::new(".continue/config.json")
        );
        std::fs::remove_dir_all(&dir).ok();
    }
}
