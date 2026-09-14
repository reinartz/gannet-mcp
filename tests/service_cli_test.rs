// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Ole Reinartz
//
// Phase 2 service CLI tests — non-privileged paths only.
// No test here requires root/admin, network access, or running services.

use std::process::Command;

/// Resolve the debug/test binary the same way `tests/integration_test.rs` does.
fn bin() -> std::path::PathBuf {
    assert_cmd::cargo::cargo_bin("gannet-mcp")
}

fn is_root() -> bool {
    Command::new("id")
        .arg("-u")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "0")
        .unwrap_or(false)
}

fn read_opt(path: &str) -> Option<String> {
    std::fs::read_to_string(path).ok()
}

#[test]
fn service_status_exits_zero_without_privileges() {
    // `service status` is informational: exit 0 even when nothing is installed
    // and without any privileges.
    let output = Command::new(bin())
        .args(["service", "status"])
        .output()
        .expect("failed to run `service status`");
    assert!(
        output.status.success(),
        "`service status` must exit 0, got {} (stdout: {}, stderr: {})",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn service_install_non_root_prints_sudo_hint_and_changes_nothing() {
    if is_root() {
        eprintln!("SKIP: running as root; non-privileged install path not applicable");
        return;
    }

    const UNIT: &str = "/etc/systemd/system/gannet-mcp.service";
    const SYSUSERS: &str = "/usr/lib/sysusers.d/gannet-mcp.conf";
    const TMPFILES: &str = "/usr/lib/tmpfiles.d/gannet-mcp.conf";
    const PLIST: &str = "/Library/LaunchDaemons/io.github.reinartz.gannet-mcp.plist";

    let before = [
        (UNIT, read_opt(UNIT)),
        (SYSUSERS, read_opt(SYSUSERS)),
        (TMPFILES, read_opt(TMPFILES)),
        (PLIST, read_opt(PLIST)),
    ];

    let output = Command::new(bin())
        .args(["service", "install"])
        .output()
        .expect("failed to run `service install`");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.contains("sudo gannet-mcp service install"),
        "non-root install must print the exact elevation command; stdout: {stdout:?}, stderr: {stderr:?}"
    );
    // Mutating without elevation must fail (exit non-zero) ...
    assert!(
        !output.status.success(),
        "non-root install must fail; stdout: {stdout:?}, stderr: {stderr:?}"
    );
    // ... and crucially must not have touched the filesystem.
    for (path, content_before) in before {
        let after = read_opt(path);
        assert_eq!(
            after, content_before,
            "non-root install must leave {path} unchanged"
        );
    }
}

#[test]
fn service_help_hides_run_subcommand() {
    // `run` is the hidden daemon entry point: it must parse but stay hidden.
    let output = Command::new(bin())
        .args(["service", "--help"])
        .output()
        .expect("failed to run `service --help`");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    for visible in ["install", "uninstall", "start", "stop", "restart", "status"] {
        assert!(
            stdout.contains(visible),
            "`service --help` should list `{visible}`, got:\n{stdout}"
        );
    }
    // No help line may advertise `run` as a subcommand (word-boundary check
    // so prose like "running" cannot trip the assertion).
    for line in stdout.lines() {
        let first_word = line.split_whitespace().next().unwrap_or_default();
        assert_ne!(
            first_word, "run",
            "`service --help` must hide the `run` subcommand, got line: {line:?}\nfull help:\n{stdout}"
        );
    }
}
