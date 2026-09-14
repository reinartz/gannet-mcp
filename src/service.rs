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

    const SERVICE_NAME: &str = "gannet-mcp";
    const UNIT_PATH: &str = "/etc/systemd/system/gannet-mcp.service";
    const SYSUSERS_DROP_IN: &str = "/usr/lib/sysusers.d/gannet-mcp.conf";
    const TMPFILES_DROP_IN: &str = "/usr/lib/tmpfiles.d/gannet-mcp.conf";

    /// systemd unit for the HTTP daemon (mirrors `systemd/gannet-mcp.service`).
    const UNIT_FILE: &str = r#"[Unit]
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

    /// Matches `systemd/gannet-mcp.sysusers`.
    const SYSUSERS_CONTENT: &str = "u gannet-mcp - \"gannet-mcp daemon\" - -\n";
    /// Matches `systemd/gannet-mcp.tmpfiles`.
    const TMPFILES_CONTENT: &str = "d /var/log/gannet-mcp 0750 gannet-mcp gannet-mcp -\n";

    fn is_root() -> bool {
        Command::new("id")
            .arg("-u")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "0")
            .unwrap_or(false)
    }

    fn elevation_hint(verb: &str) {
        println!("The 'service {verb}' action requires root privileges.");
        println!("Re-run with elevation:");
        println!("  sudo gannet-mcp service {verb}");
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

    pub fn install() -> Result<()> {
        if !is_root() {
            elevation_hint("install");
            return Ok(());
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
        run_cmd("systemctl", &["daemon-reload"])?;
        run_cmd("systemctl", &["enable", "--now", SERVICE_NAME])?;
        println!("gannet-mcp service installed and started.");
        Ok(())
    }

    pub fn uninstall() -> Result<()> {
        if !is_root() {
            elevation_hint("uninstall");
            return Ok(());
        }
        let _ = run_cmd("systemctl", &["disable", "--now", SERVICE_NAME]);
        for path in [UNIT_PATH, SYSUSERS_DROP_IN, TMPFILES_DROP_IN] {
            match std::fs::remove_file(path) {
                Ok(()) | Err(_) => {}
            }
        }
        let _ = run_cmd("systemctl", &["daemon-reload"]);
        println!("gannet-mcp service uninstalled.");
        Ok(())
    }

    pub fn start() -> Result<()> {
        if !is_root() {
            elevation_hint("start");
            return Ok(());
        }
        run_cmd("systemctl", &["start", SERVICE_NAME])?;
        println!("gannet-mcp service started.");
        Ok(())
    }

    pub fn stop() -> Result<()> {
        if !is_root() {
            elevation_hint("stop");
            return Ok(());
        }
        run_cmd("systemctl", &["stop", SERVICE_NAME])?;
        println!("gannet-mcp service stopped.");
        Ok(())
    }

    pub fn restart() -> Result<()> {
        if !is_root() {
            elevation_hint("restart");
            return Ok(());
        }
        run_cmd("systemctl", &["restart", SERVICE_NAME])?;
        println!("gannet-mcp service restarted.");
        Ok(())
    }

    pub fn status() -> Result<()> {
        // Informational only: never requires root, never fails hard.
        match Command::new("systemctl")
            .arg("--no-pager")
            .arg("status")
            .arg(SERVICE_NAME)
            .status()
        {
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

    const LABEL: &str = "io.github.reinartz.gannet-mcp";
    const PLIST_PATH: &str = "/Library/LaunchDaemons/io.github.reinartz.gannet-mcp.plist";

    fn is_root() -> bool {
        Command::new("id")
            .arg("-u")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "0")
            .unwrap_or(false)
    }

    fn elevation_hint(verb: &str) {
        println!("The 'service {verb}' action requires root privileges.");
        println!("Re-run with elevation:");
        println!("  sudo gannet-mcp service {verb}");
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
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{LABEL}</string>
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
    <string>/var/log/gannet-mcp.log</string>
    <key>StandardErrorPath</key>
    <string>/var/log/gannet-mcp.log</string>
</dict>
</plist>
"#
        )
    }

    fn domain_target() -> String {
        format!("system/{LABEL}")
    }

    pub fn install() -> Result<()> {
        if !is_root() {
            elevation_hint("install");
            return Ok(());
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
        if !is_root() {
            elevation_hint("uninstall");
            return Ok(());
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
            elevation_hint("start");
            return Ok(());
        }
        run_cmd("launchctl", &["kickstart", "-k", &domain_target()])?;
        println!("gannet-mcp service started.");
        Ok(())
    }

    pub fn stop() -> Result<()> {
        if !is_root() {
            elevation_hint("stop");
            return Ok(());
        }
        run_cmd("launchctl", &["bootout", &domain_target()])?;
        println!("gannet-mcp service stopped.");
        Ok(())
    }

    pub fn restart() -> Result<()> {
        if !is_root() {
            elevation_hint("restart");
            return Ok(());
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

    const SERVICE_NAME: &str = "gannet-mcp";

    fn is_elevated() -> bool {
        ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CREATE_SERVICE).is_ok()
    }

    fn admin_hint(verb: &str) {
        println!("The 'service {verb}' action requires elevation.");
        println!("Run as administrator (elevated prompt) and retry:");
        println!("  gannet-mcp service {verb}");
    }

    fn connect(access: ServiceManagerAccess) -> Result<ServiceManager> {
        ServiceManager::local_computer(None::<&str>, access)
            .context("failed to connect to the Service Control Manager")
    }

    pub fn install() -> Result<()> {
        if !is_elevated() {
            admin_hint("install");
            return Ok(());
        }
        let manager = connect(ServiceManagerAccess::CREATE_SERVICE)?;
        let exe = std::env::current_exe().context("failed to locate current executable")?;
        let info = ServiceInfo {
            name: OsString::from(SERVICE_NAME),
            display_name: OsString::from("Gannet MCP Server"),
            service_type: ServiceType::OWN_PROCESS,
            start_type: ServiceStartType::AutoStart,
            error_control: ServiceErrorControl::Normal,
            executable_path: exe,
            launch_arguments: vec![OsString::from("service"), OsString::from("run")],
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
        if !is_elevated() {
            admin_hint("uninstall");
            return Ok(());
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
            admin_hint("start");
            return Ok(());
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
            admin_hint("stop");
            return Ok(());
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
            admin_hint("restart");
            return Ok(());
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
                    // so the SCM does not mark the stop as hung.
                    std::process::exit(0);
                }
                ServiceControl::Interrogate => ServiceControlHandlerResult::NoError,
                _ => ServiceControlHandlerResult::NotImplemented,
            }
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
}
