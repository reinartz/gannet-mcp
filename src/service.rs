// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Ole Reinartz
//!
//! OS service management for gannet-mcp (`gannet-mcp service ...`).
//!
//! - `run` is the hidden daemon entry point: it forces HTTP mode
//!   (`use_stdio = false`) and serves MCP over Streamable HTTP. All OS
//!   supervisors (systemd unit, launchd plist, Windows SCM) launch
//!   `gannet-mcp service run`.
//! - `install` writes the OS-native service definition pointing at
//!   `... service run`, then enables and starts it.
//! - Non-root / non-admin invocations print the exact elevation command
//!   instead of failing cryptically.

use anyhow::{Context, Result};
use clap::Subcommand;

/// Manage the gannet-mcp OS service (HTTP daemon).
#[derive(Subcommand, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceAction {
    /// Install and start the OS service (requires root / admin)
    Install,
    /// Stop and remove the OS service (requires root / admin)
    Uninstall,
    /// Start the OS service (requires root / admin)
    Start,
    /// Stop the OS service (requires root / admin)
    Stop,
    /// Restart the OS service (requires root / admin)
    Restart,
    /// Show the OS service status (no privileges required)
    Status,
    /// Run the HTTP daemon (used by the OS supervisor; hidden)
    #[command(hide = true)]
    Run,
}

/// Force HTTP-daemon mode on a loaded config (used by `service run`).
pub fn apply_run_overrides(config: &mut crate::Config) {
    config.server.use_stdio = false;
}

/// Load the daemon config from env/file sources and force HTTP mode.
///
/// Note: the binary's `service run` path instead reuses the CLI-aware
/// `load_config` from `main.rs` and then calls [`apply_run_overrides`];
/// this helper serves library callers and keeps the forcing logic testable.
pub fn load_service_config() -> Result<crate::Config> {
    let mut config = crate::Config::from_env().unwrap_or_default();
    apply_run_overrides(&mut config);
    Ok(config)
}

/// Shared pure helpers (cross-platform, unit-testable).
///
/// Platform `#[cfg]` modules reuse these so `cargo test` on any host covers
/// the rendering / command-building / elevation-detection logic. The helpers
/// perform no I/O and require no privileges.
pub(crate) const SYSTEMD_SERVICE_NAME: &str = "gannet-mcp";
pub(crate) const SYSTEMD_UNIT_PATH: &str = "/etc/systemd/system/gannet-mcp.service";
pub(crate) const SYSTEMD_SYSUSERS_PATH: &str = "/usr/lib/sysusers.d/gannet-mcp.conf";
pub(crate) const SYSTEMD_TMPFILES_PATH: &str = "/usr/lib/tmpfiles.d/gannet-mcp.conf";

/// systemd unit for the HTTP daemon (mirrors `systemd/gannet-mcp.service`).
pub(crate) const SYSTEMD_UNIT_FILE: &str = r#"[Unit]
Description=Web Search MCP Server (HTTP mode)
Documentation=https://github.com/reinartz/gannet-mcp
After=network.target

[Service]
Type=simple
User=gannet-mcp
Group=gannet-mcp
EnvironmentFile=-/etc/gannet-mcp.conf
ExecStart=/usr/bin/gannet-mcp service run
Restart=on-failure
RestartSec=5
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/log/gannet-mcp
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
"#;

/// Exact content of `systemd/gannet-mcp.sysusers` (reused verbatim for the
/// sysusers drop-in written by `service install`).
pub(crate) const SYSUSERS_CONTENT: &str = "# systemd-sysusers drop-in for gannet-mcp.\n# Installed to %{_sysusersdir}/gannet-mcp.conf by the RPM spec.\n# Creates the unprivileged user the gannet-mcp.service unit runs as.\nu gannet-mcp - \"gannet-mcp daemon\" - -\n";
/// Exact content of `systemd/gannet-mcp.tmpfiles` (reused verbatim for the
/// tmpfiles drop-in written by `service install`).
pub(crate) const TMPFILES_CONTENT: &str = "# systemd-tmpfiles entry for gannet-mcp.\n# Installed to %{_tmpfilesdir}/gannet-mcp.conf by the RPM spec.\n# Creates /var/log/gannet-mcp owned by the service user (see ReadWritePaths=\n# in gannet-mcp.service).\nd /var/log/gannet-mcp 0750 gannet-mcp gannet-mcp -\n";

// Unused on Linux/Windows builds; exercised by unit tests + the macOS module.
#[allow(dead_code)]
pub(crate) const LAUNCHD_LABEL: &str = "io.github.reinartz.gannet-mcp";
// Unused on Linux/Windows builds; exercised by unit tests + the macOS module.
#[allow(dead_code)]
pub(crate) const LAUNCHD_PLIST_PATH: &str =
    "/Library/LaunchDaemons/io.github.reinartz.gannet-mcp.plist";
// Unused on Linux/Windows builds; exercised by unit tests + the macOS module.
#[allow(dead_code)]
pub(crate) const LAUNCHD_LOG_PATH: &str = "/var/log/gannet-mcp.log";

// Unused on Linux/macOS builds; exercised by unit tests + the Windows module.
#[allow(dead_code)]
pub(crate) const WINDOWS_SERVICE_NAME: &str = "gannet-mcp";

/// Parse `id -u` output: root iff the trimmed UID is "0".
pub(crate) fn is_root_uid_text(output: &str) -> bool {
    output.trim() == "0"
}

/// Exact elevation hint printed for non-root invocations (Linux/macOS).
pub(crate) fn elevation_hint_text(verb: &str) -> String {
    format!(
        "The 'service {verb}' action requires root privileges.\nRe-run with elevation:\n  sudo gannet-mcp service {verb}"
    )
}

/// Exact elevation hint printed for non-admin invocations (Windows).
/// Always contains the required "Run as administrator" phrase.
/// Unused on Linux/macOS builds; exercised by unit tests + Windows module.
#[allow(dead_code)]
pub(crate) fn admin_hint_text(verb: &str) -> String {
    format!(
        "The 'service {verb}' action requires elevation.\nRun as administrator (elevated prompt) and retry:\n  gannet-mcp service {verb}"
    )
}

/// Render the launchd LaunchDaemon plist for `exe` (`... service run`).
/// Unused on Linux/Windows builds; exercised by unit tests + macOS module.
#[allow(dead_code)]
pub(crate) fn render_launchd_plist(exe: &str, label: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{exe}</string>
        <string>service</string>
        <string>run</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{LOG}</string>
    <key>StandardErrorPath</key>
    <string>{LOG}</string>
</dict>
</plist>
"#,
        LOG = LAUNCHD_LOG_PATH
    )
}

/// `launchctl` domain target for the system daemon (`system/<label>`).
/// Unused on Linux/Windows builds; exercised by unit tests + macOS module.
#[allow(dead_code)]
pub(crate) fn launchd_domain_target(label: &str) -> String {
    format!("system/{label}")
}

/// Binary arguments the Windows SCM `binPath` must carry (`service run`).
/// Unused on Linux/macOS builds; exercised by unit tests + Windows module.
#[allow(dead_code)]
pub(crate) fn windows_launch_args() -> [&'static str; 2] {
    ["service", "run"]
}

/// Build a `(program, args)` command tuple for tests / platform modules.
pub(crate) fn systemctl_command(action: &str, service: &str) -> (String, Vec<String>) {
    match action {
        "enable-now" => (
            "systemctl".to_string(),
            vec![
                "enable".to_string(),
                "--now".to_string(),
                service.to_string(),
            ],
        ),
        "disable-now" => (
            "systemctl".to_string(),
            vec![
                "disable".to_string(),
                "--now".to_string(),
                service.to_string(),
            ],
        ),
        "daemon-reload" => ("systemctl".to_string(), vec!["daemon-reload".to_string()]),
        other => (
            "systemctl".to_string(),
            vec![other.to_string(), service.to_string()],
        ),
    }
}

/// `systemctl --no-pager status <service>` (proxied by `service status`).
pub(crate) fn systemctl_status_command(service: &str) -> (String, Vec<String>) {
    (
        "systemctl".to_string(),
        vec![
            "--no-pager".to_string(),
            "status".to_string(),
            service.to_string(),
        ],
    )
}

/// Build a `(program, args)` launchctl tuple (`bootstrap`/`bootout`/etc.).
/// Unused on Linux/Windows builds; exercised by unit tests + macOS module.
#[allow(dead_code)]
pub(crate) fn launchctl_command(action: &str, target_or_plist: &str) -> (String, Vec<String>) {
    match action {
        "bootstrap" => (
            "launchctl".to_string(),
            vec![
                "bootstrap".to_string(),
                "system".to_string(),
                target_or_plist.to_string(),
            ],
        ),
        "bootout" => (
            "launchctl".to_string(),
            vec!["bootout".to_string(), target_or_plist.to_string()],
        ),
        "kickstart" => (
            "launchctl".to_string(),
            vec![
                "kickstart".to_string(),
                "-k".to_string(),
                target_or_plist.to_string(),
            ],
        ),
        "print" => (
            "launchctl".to_string(),
            vec!["print".to_string(), target_or_plist.to_string()],
        ),
        other => (
            "launchctl".to_string(),
            vec![other.to_string(), target_or_plist.to_string()],
        ),
    }
}

/// Dispatch an OS service action.
pub fn run_service(action: &ServiceAction) -> Result<()> {
    match action {
        ServiceAction::Run => run_daemon(),
        ServiceAction::Install => platform::install(),
        ServiceAction::Uninstall => platform::uninstall(),
        ServiceAction::Start => platform::start(),
        ServiceAction::Stop => platform::stop(),
        ServiceAction::Restart => platform::restart(),
        ServiceAction::Status => platform::status(),
    }
}

/// Hidden daemon entry: serve MCP over Streamable HTTP (never STDIO).
#[cfg(not(windows))]
fn run_daemon() -> Result<()> {
    let config = load_service_config()?;
    init_daemon_logging(&config.logging);
    run_http_blocking(config)
}

/// Serve `config` over Streamable HTTP on a fresh Tokio runtime.
///
/// `pub` so the binary can reuse it for the CLI-aware `service run` path
/// (same behavior, but honoring global flags like `--config` / `--port`).
pub fn run_http_blocking(config: crate::Config) -> Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("Failed to create Tokio runtime")?;
    runtime.block_on(async {
        let server = crate::Server::new(config)
            .await
            .context("Failed to initialize server")?;
        server.run_http().await
    })
}

fn init_daemon_logging(logging: &crate::LoggingConfig) {
    use tracing::Level;
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};
    let level = match logging.level.as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("gannet_mcp={}", level)));
    let subscriber = tracing_subscriber::registry().with(env_filter);
    match logging.format.as_str() {
        "json" => {
            subscriber
                .with(fmt::layer().with_writer(std::io::stderr).json())
                .init();
        }
        "compact" => {
            subscriber
                .with(fmt::layer().with_writer(std::io::stderr).compact())
                .init();
        }
        _ => {
            subscriber
                .with(fmt::layer().with_writer(std::io::stderr).pretty())
                .init();
        }
    }
}

// ---------------------------------------------------------------------------
// Linux (systemd)
// ---------------------------------------------------------------------------
#[cfg(all(unix, not(target_os = "macos")))]
mod platform {
    use super::*;
    use std::process::Command;

    use super::{
        SYSTEMD_SERVICE_NAME as SERVICE_NAME, SYSTEMD_SYSUSERS_PATH as SYSUSERS_DROP_IN,
        SYSTEMD_TMPFILES_PATH as TMPFILES_DROP_IN, SYSTEMD_UNIT_FILE as UNIT_FILE,
        SYSTEMD_UNIT_PATH as UNIT_PATH, SYSUSERS_CONTENT, TMPFILES_CONTENT,
    };

    fn is_root() -> bool {
        Command::new("id")
            .arg("-u")
            .output()
            .map(|o| super::is_root_uid_text(&String::from_utf8_lossy(&o.stdout)))
            .unwrap_or(false)
    }

    fn elevation_hint(verb: &str) {
        println!("{}", super::elevation_hint_text(verb));
    }

    /// Print the elevation hint, then fail without touching the filesystem.
    fn not_elevated(verb: &str) -> anyhow::Error {
        elevation_hint(verb);
        anyhow::anyhow!(
            "service {verb} requires root privileges (re-run with: sudo gannet-mcp service {verb})"
        )
    }

    fn run_cmd(prog: &str, args: &[&str]) -> Result<()> {
        let status = Command::new(prog)
            .args(args)
            .status()
            .with_context(|| format!("failed to execute '{prog}'"))?;
        if status.success() {
            Ok(())
        } else {
            anyhow::bail!("'{prog} {}' exited with status {status}", args.join(" "))
        }
    }

    fn run_tuple(prog: &str, args: &[String]) -> Result<()> {
        let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();
        run_cmd(prog, &args_ref)
    }

    pub fn install() -> Result<()> {
        // Elevation gate FIRST: fail before touching the filesystem.
        if !is_root() {
            return Err(not_elevated("install"));
        }
        std::fs::write(UNIT_PATH, UNIT_FILE)
            .with_context(|| format!("failed to write {UNIT_PATH}"))?;
        std::fs::write(SYSUSERS_DROP_IN, SYSUSERS_CONTENT)
            .with_context(|| format!("failed to write {SYSUSERS_DROP_IN}"))?;
        std::fs::write(TMPFILES_DROP_IN, TMPFILES_CONTENT)
            .with_context(|| format!("failed to write {TMPFILES_DROP_IN}"))?;
        // Best effort: helpers may be absent on non-systemd systems; report clearly.
        let _ = run_cmd("systemd-sysusers", &[]);
        let _ = run_cmd("systemd-tmpfiles", &["--create"]);
        let (prog, args) = super::systemctl_command("daemon-reload", SERVICE_NAME);
        run_tuple(&prog, &args)?;
        let (prog, args) = super::systemctl_command("enable-now", SERVICE_NAME);
        run_tuple(&prog, &args)?;
        println!("gannet-mcp service installed and started.");
        Ok(())
    }

    pub fn uninstall() -> Result<()> {
        // Elevation gate FIRST: fail before touching the filesystem.
        if !is_root() {
            return Err(not_elevated("uninstall"));
        }
        let (prog, args) = super::systemctl_command("disable-now", SERVICE_NAME);
        // disable --now may fail when the unit was never installed; best effort.
        let _ = run_tuple(&prog, &args);
        for path in [UNIT_PATH, SYSUSERS_DROP_IN, TMPFILES_DROP_IN] {
            match std::fs::remove_file(path) {
                Ok(()) | Err(_) => {}
            }
        }
        let (prog, args) = super::systemctl_command("daemon-reload", SERVICE_NAME);
        let _ = run_tuple(&prog, &args);
        println!("gannet-mcp service uninstalled.");
        Ok(())
    }

    pub fn start() -> Result<()> {
        if !is_root() {
            return Err(not_elevated("start"));
        }
        let (prog, args) = super::systemctl_command("start", SERVICE_NAME);
        run_tuple(&prog, &args)?;
        println!("gannet-mcp service started.");
        Ok(())
    }

    pub fn stop() -> Result<()> {
        if !is_root() {
            return Err(not_elevated("stop"));
        }
        let (prog, args) = super::systemctl_command("stop", SERVICE_NAME);
        run_tuple(&prog, &args)?;
        println!("gannet-mcp service stopped.");
        Ok(())
    }

    pub fn restart() -> Result<()> {
        if !is_root() {
            return Err(not_elevated("restart"));
        }
        let (prog, args) = super::systemctl_command("restart", SERVICE_NAME);
        run_tuple(&prog, &args)?;
        println!("gannet-mcp service restarted.");
        Ok(())
    }

    pub fn status() -> Result<()> {
        // Informational only: never requires root, never fails hard.
        // Proxies `systemctl --no-pager status gannet-mcp`; the exit code of
        // systemctl itself is intentionally ignored (unknown unit still exits 0 here).
        let (prog, args) = super::systemctl_status_command(SERVICE_NAME);
        match Command::new(&prog).args(&args).status() {
            Ok(_) => Ok(()),
            Err(e) => {
                println!(
                    "gannet-mcp service status unavailable: could not execute systemctl ({e})."
                );
                println!("Is systemd installed on this machine?");
                Ok(())
            }
        }
    }
}

// ---------------------------------------------------------------------------
// macOS (launchd LaunchDaemon)
// ---------------------------------------------------------------------------
#[cfg(target_os = "macos")]
mod platform {
    use super::*;
    use std::process::Command;

    use super::{LAUNCHD_LABEL as LABEL, LAUNCHD_PLIST_PATH as PLIST_PATH};

    fn is_root() -> bool {
        Command::new("id")
            .arg("-u")
            .output()
            .map(|o| super::is_root_uid_text(&String::from_utf8_lossy(&o.stdout)))
            .unwrap_or(false)
    }

    fn elevation_hint(verb: &str) {
        println!("{}", super::elevation_hint_text(verb));
    }

    /// Print the elevation hint, then fail without touching the filesystem.
    fn not_elevated(verb: &str) -> anyhow::Error {
        elevation_hint(verb);
        anyhow::anyhow!(
            "service {verb} requires root privileges (re-run with: sudo gannet-mcp service {verb})"
        )
    }

    fn run_cmd(prog: &str, args: &[&str]) -> Result<()> {
        let status = Command::new(prog)
            .args(args)
            .status()
            .with_context(|| format!("failed to execute '{prog}'"))?;
        if status.success() {
            Ok(())
        } else {
            anyhow::bail!("'{prog} {}' exited with status {status}", args.join(" "))
        }
    }

    fn plist_content(exe: &str) -> String {
        super::render_launchd_plist(exe, LABEL)
    }

    fn domain_target() -> String {
        super::launchd_domain_target(LABEL)
    }

    pub fn install() -> Result<()> {
        // Elevation gate FIRST: fail before touching the filesystem.
        if !is_root() {
            return Err(not_elevated("install"));
        }
        let exe = std::env::current_exe()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| "/usr/local/bin/gannet-mcp".to_string());
        std::fs::write(PLIST_PATH, plist_content(&exe))
            .with_context(|| format!("failed to write {PLIST_PATH}"))?;
        run_cmd("launchctl", &["bootstrap", "system", PLIST_PATH])?;
        println!("gannet-mcp service installed and started.");
        Ok(())
    }

    pub fn uninstall() -> Result<()> {
        // Elevation gate FIRST: fail before touching the filesystem.
        if !is_root() {
            return Err(not_elevated("uninstall"));
        }
        let _ = run_cmd("launchctl", &["bootout", &domain_target()]);
        match std::fs::remove_file(PLIST_PATH) {
            Ok(()) | Err(_) => {}
        }
        println!("gannet-mcp service uninstalled.");
        Ok(())
    }

    pub fn start() -> Result<()> {
        if !is_root() {
            return Err(not_elevated("start"));
        }
        run_cmd("launchctl", &["kickstart", "-k", &domain_target()])?;
        println!("gannet-mcp service started.");
        Ok(())
    }

    pub fn stop() -> Result<()> {
        if !is_root() {
            return Err(not_elevated("stop"));
        }
        run_cmd("launchctl", &["bootout", &domain_target()])?;
        println!("gannet-mcp service stopped.");
        Ok(())
    }

    pub fn restart() -> Result<()> {
        if !is_root() {
            return Err(not_elevated("restart"));
        }
        let _ = run_cmd("launchctl", &["bootout", &domain_target()]);
        run_cmd("launchctl", &["kickstart", "-k", &domain_target()])?;
        println!("gannet-mcp service restarted.");
        Ok(())
    }

    pub fn status() -> Result<()> {
        match Command::new("launchctl")
            .arg("print")
            .arg(domain_target())
            .status()
        {
            Ok(_) => Ok(()),
            Err(e) => {
                println!(
                    "gannet-mcp service status unavailable: could not execute launchctl ({e})."
                );
                Ok(())
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Windows (Service Control Manager via `windows-service`)
// ---------------------------------------------------------------------------
#[cfg(windows)]
mod platform {
    use super::*;
    use std::ffi::OsString;
    use std::time::Duration;
    use windows_service::service::{
        ServiceAccess, ServiceControl, ServiceControlAccept, ServiceErrorControl, ServiceExitCode,
        ServiceInfo, ServiceStartType, ServiceState, ServiceStatus, ServiceType,
    };
    use windows_service::service_control_handler;
    use windows_service::service_dispatcher;
    use windows_service::service_manager::{ServiceManager, ServiceManagerAccess};
    use windows_service::{define_windows_service, Error as ServiceError};

    use super::WINDOWS_SERVICE_NAME as SERVICE_NAME;

    fn is_elevated() -> bool {
        ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CREATE_SERVICE).is_ok()
    }

    fn admin_hint(verb: &str) {
        println!("{}", super::admin_hint_text(verb));
    }

    /// Print the admin hint, then fail without touching the SCM.
    fn not_elevated(verb: &str) -> anyhow::Error {
        admin_hint(verb);
        anyhow::anyhow!(
            "service {verb} requires elevation (re-run from an elevated prompt: Run as administrator)"
        )
    }

    fn connect(access: ServiceManagerAccess) -> Result<ServiceManager> {
        ServiceManager::local_computer(None::<&str>, access)
            .context("failed to connect to the Service Control Manager")
    }

    pub fn install() -> Result<()> {
        // Elevation gate FIRST: fail before touching the SCM.
        if !is_elevated() {
            return Err(not_elevated("install"));
        }
        let manager = connect(ServiceManagerAccess::CREATE_SERVICE)?;
        let exe = std::env::current_exe().context("failed to locate current executable")?;
        // SCM binPath = <current_exe> service run (launch args carry the subcommand).
        let launch_args: Vec<OsString> = super::windows_launch_args()
            .iter()
            .map(OsString::from)
            .collect();
        let info = ServiceInfo {
            name: OsString::from(SERVICE_NAME),
            display_name: OsString::from("Gannet MCP Server"),
            service_type: ServiceType::OWN_PROCESS,
            start_type: ServiceStartType::AutoStart,
            error_control: ServiceErrorControl::Normal,
            executable_path: exe,
            launch_arguments: launch_args,
            dependencies: vec![],
            account_name: None,
            account_password: None,
        };
        let service =
            manager.create_service(&info, ServiceAccess::QUERY_STATUS | ServiceAccess::START)?;
        service.start::<OsString>(&[])?;
        println!("gannet-mcp service installed and started.");
        Ok(())
    }

    pub fn uninstall() -> Result<()> {
        // Elevation gate FIRST: fail before touching the SCM.
        if !is_elevated() {
            return Err(not_elevated("uninstall"));
        }
        let manager = connect(ServiceManagerAccess::CONNECT)?;
        let service = manager.open_service(
            SERVICE_NAME,
            ServiceAccess::STOP | ServiceAccess::QUERY_STATUS | ServiceAccess::DELETE,
        )?;
        let _ = service.stop();
        service.delete()?;
        println!("gannet-mcp service uninstalled.");
        Ok(())
    }

    pub fn start() -> Result<()> {
        if !is_elevated() {
            return Err(not_elevated("start"));
        }
        let manager = connect(ServiceManagerAccess::CONNECT)?;
        let service = manager.open_service(
            SERVICE_NAME,
            ServiceAccess::START | ServiceAccess::QUERY_STATUS,
        )?;
        service.start::<OsString>(&[])?;
        println!("gannet-mcp service started.");
        Ok(())
    }

    pub fn stop() -> Result<()> {
        if !is_elevated() {
            return Err(not_elevated("stop"));
        }
        let manager = connect(ServiceManagerAccess::CONNECT)?;
        let service = manager.open_service(
            SERVICE_NAME,
            ServiceAccess::STOP | ServiceAccess::QUERY_STATUS,
        )?;
        service.stop()?;
        println!("gannet-mcp service stopped.");
        Ok(())
    }

    pub fn restart() -> Result<()> {
        if !is_elevated() {
            return Err(not_elevated("restart"));
        }
        let manager = connect(ServiceManagerAccess::CONNECT)?;
        let service = manager.open_service(
            SERVICE_NAME,
            ServiceAccess::START | ServiceAccess::STOP | ServiceAccess::QUERY_STATUS,
        )?;
        let _ = service.stop();
        service.start::<OsString>(&[])?;
        println!("gannet-mcp service restarted.");
        Ok(())
    }

    pub fn status() -> Result<()> {
        // Informational only: never fail hard; non-admins can usually query.
        let manager =
            match ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT) {
                Ok(manager) => manager,
                Err(e) => {
                    if matches!(&e, ServiceError::Winapi(io) if io.raw_os_error() == Some(5)) {
                        admin_hint("status");
                        return Ok(());
                    }
                    println!("gannet-mcp service status unavailable: {e}.");
                    return Ok(());
                }
            };
        match manager.open_service(SERVICE_NAME, ServiceAccess::QUERY_STATUS) {
            Ok(service) => match service.query_status() {
                Ok(status) => {
                    println!("gannet-mcp service state: {:?}", status.current_state);
                    Ok(())
                }
                Err(e) => {
                    println!("gannet-mcp service status unavailable: {e}.");
                    Ok(())
                }
            },
            Err(e) => {
                println!("gannet-mcp service status unavailable: {e}.");
                Ok(())
            }
        }
    }

    define_windows_service!(owned_service_main, service_main_fn);

    fn service_main_fn(_args: Vec<OsString>) {
        if let Err(e) = service_main_inner() {
            eprintln!("gannet-mcp service error: {e:#}");
            std::process::exit(1);
        }
    }

    fn service_main_inner() -> Result<()> {
        let status_handle = service_control_handler::register(SERVICE_NAME, |control| {
            use windows_service::service_control_handler::ServiceControlHandlerResult;
            match control {
                ServiceControl::Stop | ServiceControl::Shutdown => {
                    // No graceful-shutdown plumbing yet (Phase 2); exit promptly
                    // so the SCM does not mark the stop as hung. The SCM
                    // observes the process exit and marks the service Stopped.
                    std::process::exit(0);
                }
                ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
                _ => ServiceControlHandlerResult::NotImplemented,
            }
        })?;
        // Report StartPending during init so the SCM does not time out,
        // then Running once the HTTP daemon is about to serve.
        status_handle.set_service_status(ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::StartPending,
            controls_accepted: ServiceControlAccept::empty(),
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::from_secs(30),
            process_id: None,
        })?;
        status_handle.set_service_status(ServiceStatus {
            service_type: ServiceType::OWN_PROCESS,
            current_state: ServiceState::Running,
            controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
            exit_code: ServiceExitCode::Win32(0),
            checkpoint: 0,
            wait_hint: Duration::default(),
            process_id: None,
        })?;
        super::run_http_blocking(super::load_service_config()?)
    }

    /// SCM entry for `service run` on Windows.
    ///
    /// When launched interactively (not by the SCM), the dispatcher handshake
    /// fails with `ERROR_FAILED_SERVICE_CONTROLLER_CONNECT` (1063); fall back
    /// to a plain foreground HTTP run.
    pub fn run_via_scm_or_foreground() -> Result<()> {
        match service_dispatcher::start(SERVICE_NAME, owned_service_main) {
            Ok(()) => Ok(()),
            Err(e) => {
                let failed_controller_connect =
                    matches!(&e, ServiceError::Winapi(io) if io.raw_os_error() == Some(1063));
                if failed_controller_connect {
                    super::run_http_blocking(super::load_service_config()?)
                } else {
                    Err(e).context("service dispatcher failed")
                }
            }
        }
    }
}

/// Windows `Run` goes through the SCM dispatcher (with interactive fallback);
/// other platforms run the HTTP daemon directly.
#[cfg(windows)]
fn run_daemon() -> Result<()> {
    platform::run_via_scm_or_foreground()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_run_overrides_forces_http() {
        let mut config = crate::Config::default();
        config.server.use_stdio = true;
        apply_run_overrides(&mut config);
        assert!(!config.server.use_stdio);
    }

    #[test]
    fn test_load_service_config_forces_http() {
        let config = load_service_config().unwrap();
        assert!(
            !config.server.use_stdio,
            "service run must always use HTTP transport"
        );
    }

    #[test]
    fn test_service_action_debug() {
        let action = ServiceAction::Status;
        assert!(format!("{action:?}").contains("Status"));
    }

    #[test]
    fn test_is_root_uid_text() {
        assert!(is_root_uid_text("0\n"));
        assert!(is_root_uid_text("0"));
        assert!(is_root_uid_text("  0  \n"));
        assert!(!is_root_uid_text("1000\n"));
        assert!(!is_root_uid_text(""));
        assert!(!is_root_uid_text("root\n"));
    }

    #[test]
    fn test_elevation_hint_text_exact_command() {
        for verb in ["install", "uninstall", "start", "stop", "restart"] {
            let hint = elevation_hint_text(verb);
            assert!(
                hint.contains(&format!("sudo gannet-mcp service {verb}")),
                "hint must print the exact elevation command, got: {hint}"
            );
        }
    }

    #[test]
    fn test_admin_hint_text_requires_run_as_admin() {
        for verb in ["install", "uninstall", "start", "stop", "restart"] {
            let hint = admin_hint_text(verb);
            assert!(
                hint.contains("Run as administrator"),
                "windows hint must contain 'Run as administrator', got: {hint}"
            );
            assert!(
                hint.contains(&format!("gannet-mcp service {verb}")),
                "windows hint must name the action, got: {hint}"
            );
        }
    }

    #[test]
    fn test_systemd_unit_exec_start() {
        assert!(
            SYSTEMD_UNIT_FILE.contains("ExecStart=/usr/bin/gannet-mcp service run"),
            "unit must launch `... service run`"
        );
        assert!(SYSTEMD_UNIT_FILE.contains("User=gannet-mcp"));
        assert!(SYSTEMD_UNIT_PATH.ends_with("gannet-mcp.service"));
        assert_eq!(SYSTEMD_SERVICE_NAME, "gannet-mcp");
    }

    #[test]
    fn test_sysusers_content_matches_drop_in() {
        // Must match systemd/gannet-mcp.sysusers verbatim (incl. header).
        let expected = "# systemd-sysusers drop-in for gannet-mcp.\n# Installed to %{_sysusersdir}/gannet-mcp.conf by the RPM spec.\n# Creates the unprivileged user the gannet-mcp.service unit runs as.\nu gannet-mcp - \"gannet-mcp daemon\" - -\n";
        assert_eq!(SYSUSERS_CONTENT, expected);
    }

    #[test]
    fn test_tmpfiles_content_matches_drop_in() {
        // Must match systemd/gannet-mcp.tmpfiles verbatim (incl. header).
        let expected = "# systemd-tmpfiles entry for gannet-mcp.\n# Installed to %{_tmpfilesdir}/gannet-mcp.conf by the RPM spec.\n# Creates /var/log/gannet-mcp owned by the service user (see ReadWritePaths=\n# in gannet-mcp.service).\nd /var/log/gannet-mcp 0750 gannet-mcp gannet-mcp -\n";
        assert_eq!(TMPFILES_CONTENT, expected);
    }

    #[test]
    fn test_render_launchd_plist() {
        let plist = render_launchd_plist("/usr/local/bin/gannet-mcp", LAUNCHD_LABEL);
        assert!(plist.contains(LAUNCHD_LABEL));
        assert!(plist.contains("/usr/local/bin/gannet-mcp"));
        assert!(plist.contains("<string>service</string>"));
        assert!(plist.contains("<string>run</string>"));
        assert!(plist.contains("<key>RunAtLoad</key>"));
        assert!(plist.contains("<key>KeepAlive</key>"));
        assert!(plist.contains(LAUNCHD_LOG_PATH));
        // stdout + stderr both go to the same log file.
        assert_eq!(plist.matches(LAUNCHD_LOG_PATH).count(), 2);
        assert_eq!(
            LAUNCHD_PLIST_PATH,
            "/Library/LaunchDaemons/io.github.reinartz.gannet-mcp.plist"
        );
    }

    #[test]
    fn test_launchd_domain_target() {
        assert_eq!(
            launchd_domain_target(LAUNCHD_LABEL),
            format!("system/{LAUNCHD_LABEL}")
        );
    }

    #[test]
    fn test_windows_launch_args() {
        assert_eq!(windows_launch_args(), ["service", "run"]);
        assert_eq!(WINDOWS_SERVICE_NAME, "gannet-mcp");
    }

    #[test]
    fn test_systemctl_command_builders() {
        let (prog, args) = systemctl_status_command("gannet-mcp");
        assert_eq!(prog, "systemctl");
        assert_eq!(args, vec!["--no-pager", "status", "gannet-mcp"]);

        let (prog, args) = systemctl_command("daemon-reload", "gannet-mcp");
        assert_eq!(prog, "systemctl");
        assert_eq!(args, vec!["daemon-reload"]);

        let (prog, args) = systemctl_command("enable-now", "gannet-mcp");
        assert_eq!(prog, "systemctl");
        assert_eq!(args, vec!["enable", "--now", "gannet-mcp"]);

        let (prog, args) = systemctl_command("disable-now", "gannet-mcp");
        assert_eq!(prog, "systemctl");
        assert_eq!(args, vec!["disable", "--now", "gannet-mcp"]);

        let (prog, args) = systemctl_command("start", "gannet-mcp");
        assert_eq!(prog, "systemctl");
        assert_eq!(args, vec!["start", "gannet-mcp"]);
    }

    #[test]
    fn test_launchctl_command_builders() {
        let (prog, args) = launchctl_command("bootstrap", LAUNCHD_PLIST_PATH);
        assert_eq!(prog, "launchctl");
        assert_eq!(args, vec!["bootstrap", "system", LAUNCHD_PLIST_PATH]);

        let target = launchd_domain_target(LAUNCHD_LABEL);
        let (prog, args) = launchctl_command("bootout", &target);
        assert_eq!(prog, "launchctl");
        assert_eq!(args[0], "bootout");
        assert_eq!(args[1], target);

        let (prog, args) = launchctl_command("print", &target);
        assert_eq!(prog, "launchctl");
        assert_eq!(args, vec!["print".to_string(), target.clone()]);

        let (prog, args) = launchctl_command("kickstart", &target);
        assert_eq!(prog, "launchctl");
        assert_eq!(
            args,
            vec!["kickstart".to_string(), "-k".to_string(), target]
        );
    }
}
