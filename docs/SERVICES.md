# gannet-mcp as an OS service (HTTP daemon)

This page covers enabling and verifying the gannet-mcp HTTP service via
`gannet-mcp service ...` on Linux (systemd), macOS (launchd), and Windows
(Service Control Manager).

All three platforms supervise the same daemon entry point:
`gannet-mcp service run` (see [service run](#service-run-hidden-daemon-entry)).

> Version note: this documents the behavior implemented in `src/service.rs`
> as shipped in **0.2.0** (`Cargo.toml`, `gannet-mcp --version` → `0.2.0`).
> The `RELEASE_PLAN.md` Phase 2 / Phase 6 text describes the same design;
> where the plan and the code differ, **the code wins** and the difference
> is called out below.

## Subcommand overview

```
gannet-mcp service install|uninstall|start|stop|restart|status|run
```

- `install`, `uninstall`, `start`, `stop`, `restart` require elevation
  (root via `sudo` on Linux/macOS; an elevated prompt on Windows).
  Without elevation they print the exact elevation command and exit
  non-zero **before touching anything** (elevation gate is checked first
  in every platform module).
- `status` is informational only and **never requires privileges**
  (`src/service.rs:31-32`, and none of the three `status()` implementations
  check for root/admin).
- `run` is hidden from `--help` (`#[command(hide = true)]`,
  `src/service.rs:34-35`) but still parses: `gannet-mcp service run`.
  Users never invoke it directly; OS supervisors do.

Global flags (`--config`, `--log-level`, `--log-format`, …) are parsed for
all subcommands, including `service run` (see below).

---

## 1. Linux (systemd)

**Unit name:** `gannet-mcp` (`SYSTEMD_SERVICE_NAME`, `src/service.rs:59`).

### What `install` does

`sudo gannet-mcp service install` (`src/service.rs:374-394`):

1. Checks root via `id -u` == `0` (`src/service.rs:337-343`). If not root,
   prints the elevation hint and fails before writing anything:
   `sudo gannet-mcp service install` (`src/service.rs:116-120`).
2. Writes three files with content baked into the binary:
   - `/etc/systemd/system/gannet-mcp.service` (`SYSTEMD_UNIT_PATH`,
     `src/service.rs:60`) — content is `SYSTEMD_UNIT_FILE`
     (`src/service.rs:65-86`), which mirrors `systemd/gannet-mcp.service`:
     `ExecStart=/usr/bin/gannet-mcp service run`, `User=gannet-mcp`,
     `Group=gannet-mcp`, `EnvironmentFile=-/etc/gannet-mcp.conf`,
     `Restart=on-failure`, hardening (`NoNewPrivileges`,
     `ProtectSystem=strict`, `ProtectHome=true`,
     `ReadWritePaths=/var/log/gannet-mcp`), `WantedBy=multi-user.target`.
   - `/usr/lib/sysusers.d/gannet-mcp.conf` (`SYSTEMD_SYSUSERS_PATH`,
     `src/service.rs:61`) — content `SYSUSERS_CONTENT`
     (`src/service.rs:90`): `u gannet-mcp - "gannet-mcp daemon" - -`
     (verbatim copy of `systemd/gannet-mcp.sysusers`).
   - `/usr/lib/tmpfiles.d/gannet-mcp.conf` (`SYSTEMD_TMPFILES_PATH`,
     `src/service.rs:62`) — content `TMPFILES_CONTENT`
     (`src/service.rs:93`): `d /var/log/gannet-mcp 0750 gannet-mcp gannet-mcp -`
     (verbatim copy of `systemd/gannet-mcp.tmpfiles`).
3. Runs `systemd-sysusers` and `systemd-tmpfiles --create` on a
   **best-effort** basis (failures ignored — covers non-systemd systems,
   `src/service.rs:386-387`).
4. Runs `systemctl daemon-reload`, then `systemctl enable --now gannet-mcp`
   (`src/service.rs:388-391`), i.e. install both enables (start at boot)
   and starts the service immediately.
5. Prints `gannet-mcp service installed and started.`

Note: install does **not** create `/etc/gannet-mcp.conf` itself; the unit
references it as optional (`EnvironmentFile=-...`, the `-` prefix means
"missing file is OK"). That file comes from a package install, not from
`service install` (see ambiguity note below).

### Start / stop / restart

Require root (`sudo`); each proxies one `systemctl` verb
(`src/service.rs:415-443`):

```
sudo gannet-mcp service start    # systemctl start gannet-mcp
sudo gannet-mcp service stop     # systemctl stop gannet-mcp
sudo gannet-mcp service restart  # systemctl restart gannet-mcp
```

### Verify

`service status` needs no privileges. It proxies
`systemctl --no-pager status gannet-mcp` (`src/service.rs:445-460`,
command built at `src/service.rs:206-215`):

```
gannet-mcp service status
systemctl status gannet-mcp
```

Caveat: the proxy **ignores systemctl's own exit code**
(`src/service.rs:447-448`) — `service status` exits 0 even for an unknown
or stopped unit. Read the printed output; do not rely on the exit code in
scripts. If the `systemctl` binary itself is missing it prints
`gannet-mcp service status unavailable: ...` plus
`Is systemd installed on this machine?` and still exits 0.

### Logs

The daemon logs to **stderr**, which systemd captures into the journal —
read them with:

```
journalctl -u gannet-mcp
journalctl -u gannet-mcp -f   # follow
```

(`init_daemon_logging`, `src/service.rs:291-321`, attaches a
tracing-subscriber fmt layer with `std::io::stderr` as writer.)
`/var/log/gannet-mcp` exists (tmpfiles drop-in + `ReadWritePaths=`) but
`service run` itself does not write log files there — see ambiguity note.

### Uninstall

`sudo gannet-mcp service uninstall` (`src/service.rs:396-413`):

1. Root gate (same hint pattern).
2. `systemctl disable --now gannet-mcp` on a best-effort basis
   (may fail if the unit was never installed; ignored).
3. Deletes the three files (`/etc/systemd/system/gannet-mcp.service`,
   `/usr/lib/sysusers.d/gannet-mcp.conf`,
   `/usr/lib/tmpfiles.d/gannet-mcp.conf`); missing files are ignored.
4. `systemctl daemon-reload` (best effort).
5. Prints `gannet-mcp service uninstalled.`

Uninstall does not delete `/var/log/gannet-mcp` or the `gannet-mcp` user —
remove those manually if desired (`userdel gannet-mcp`,
`rm -rf /var/log/gannet-mcp`).

---

## 2. macOS (launchd)

**Label:** `io.github.reinartz.gannet-mcp` (`LAUNCHD_LABEL`,
`src/service.rs:97`).
**Plist path:** `/Library/LaunchDaemons/io.github.reinartz.gannet-mcp.plist`
(`LAUNCHD_PLIST_PATH`, `src/service.rs:100-101`) — a system-wide
LaunchDaemon, so installation requires root.

### What `install` does

`sudo gannet-mcp service install` (`src/service.rs:513-526`):

1. Root gate via `id -u` (`src/service.rs:473-479`); non-root prints
   `sudo gannet-mcp service install` and fails before writing anything.
2. Renders the plist with `render_launchd_plist` (`src/service.rs:135-162`):
   - `Label` = `io.github.reinartz.gannet-mcp`
   - `ProgramArguments` = `[<exe>, service, run]`, where `<exe>` is
     `std::env::current_exe()` at install time, falling back to
     `/usr/local/bin/gannet-mcp` only if the current-exe lookup fails
     (`src/service.rs:518-520`).
   - `RunAtLoad` = true, `KeepAlive` = true.
   - `StandardOutPath` and `StandardErrorPath` **both** point at
     `/var/log/gannet-mcp.log` (`LAUNCHD_LOG_PATH`, `src/service.rs:104`).
3. Writes the plist to
   `/Library/LaunchDaemons/io.github.reinartz.gannet-mcp.plist`.
4. Runs `launchctl bootstrap system <plist>` (`src/service.rs:523`).
   With `RunAtLoad`, bootstrap loads and starts the daemon.
5. Prints `gannet-mcp service installed and started.`

### Start / stop / restart

Require root (`sudo`). Implemented with `launchctl` against the system
domain target `system/io.github.reinartz.gannet-mcp`
(`launchd_domain_target`, `src/service.rs:167-169`):

```
sudo gannet-mcp service start    # launchctl kickstart -k system/io.github.reinartz.gannet-mcp
sudo gannet-mcp service stop     # launchctl bootout  system/io.github.reinartz.gannet-mcp
sudo gannet-mcp service restart  # bootout (best effort) then kickstart -k
```

(`src/service.rs:541-567`.)

### Verify

```
sudo gannet-mcp service status                        # no root actually required
launchctl print system/io.github.reinartz.gannet-mcp  # what status() runs
```

`service status` runs `launchctl print system/<label>`
(`src/service.rs:569-583`) and never fails hard: if `launchctl` itself
cannot be executed it prints
`gannet-mcp service status unavailable: ...` and exits 0.

### Logs

stdout and stderr both go to `/var/log/gannet-mcp.log`:

```
tail -f /var/log/gannet-mcp.log
```

### Uninstall

`sudo gannet-mcp service uninstall` (`src/service.rs:528-539`):

1. Root gate.
2. `launchctl bootout system/io.github.reinartz.gannet-mcp` (best effort).
3. Deletes the plist (missing file ignored).
4. Prints `gannet-mcp service uninstalled.`

---

## 3. Windows (Service Control Manager)

**Service name:** `gannet-mcp` (`WINDOWS_SERVICE_NAME`,
`src/service.rs:108`).
**Display name:** `Gannet MCP Server` (`src/service.rs:640`).
Implementation uses the `windows-service` crate (`Cargo.toml:99-100`);
all SCM access goes through `ServiceManager::local_computer`
(`src/service.rs:621-624`).

### Privileges

Every mutating action probes elevation by attempting to open the SCM with
`ServiceManagerAccess::CREATE_SERVICE` (`is_elevated`,
`src/service.rs:605-607`). Without elevation it prints the admin hint and
fails before touching the SCM:

```
The 'service <verb>' action requires elevation.
Run as administrator (elevated prompt) and retry:
  gannet-mcp service <verb>
```

(`admin_hint_text`, `src/service.rs:126-130`; always contains the phrase
**"Run as administrator"**.) So: open an **elevated ("Run as
administrator") prompt** first.

### What `install` does

From the elevated prompt, `gannet-mcp service install`
(`src/service.rs:626-655`):

1. Elevation gate (`not_elevated("install")`).
2. Registers the SCM service with:
   - `binPath` = `<current_exe> service run` — i.e. the installing
     binary's own path plus the launch args `["service", "run"]`
     (`windows_launch_args`, `src/service.rs:174-176`;
     wired into `ServiceInfo` at `src/service.rs:633-649`).
   - `ServiceType::OWN_PROCESS`, `ServiceStartType::AutoStart` (start at
     boot), `ServiceErrorControl::Normal`.
   - No explicit service account (`account_name: None`), i.e. the code
     does not select an account — see ambiguity note.
3. Starts the service immediately (`service.start`, `src/service.rs:652`).
4. Prints `gannet-mcp service installed and started.`

### Start / stop / restart

From the elevated prompt (`src/service.rs:673-714`):

```
gannet-mcp service start    # SCM start
gannet-mcp service stop     # SCM stop
gannet-mcp service restart  # SCM stop (best effort) then start
```

### Verify

```
gannet-mcp service status
```

`status()` (`src/service.rs:716-746`) connects with
`ServiceManagerAccess::CONNECT`, opens the service with
`QUERY_STATUS`, and prints:

```
gannet-mcp service state: <ServiceState>
```

e.g. `Running` / `Stopped`. It never fails hard: an uninstalled service
or query failure prints `gannet-mcp service status unavailable: ...` and
exits 0; access-denied (Win32 error 5) prints the "Run as administrator"
hint and exits 0. (Non-admin users can usually query, so `status` normally
works from a non-elevated prompt.)

### Logs

On Windows the daemon path is the SCM dispatcher
(`run_via_scm_or_foreground`, `src/service.rs:799-812`); there is no
separate log file wired by `service install`. Daemon stderr/stdout handling
is determined by how the service host captures it — check
**Event Viewer → Windows Logs → Application / System** for service
lifecycle entries, and run the foreground equivalent
(`gannet-mcp service run`, or `gannet-mcp --http`) in a console to see
log output interactively while diagnosing config problems. Fatal service
errors are printed as `gannet-mcp service error: ...`
(`src/service.rs:750-755`).

### Uninstall

From the elevated prompt, `gannet-mcp service uninstall`
(`src/service.rs:657-671`): opens the service with
`STOP | QUERY_STATUS | DELETE`, stops it (best effort), deletes the SCM
entry, prints `gannet-mcp service uninstalled.`

---

## 4. Verify it's serving (all platforms)

After `service install` + start, the daemon serves MCP over
**Streamable HTTP**. Defaults (`src/config.rs:112-118`,
`src/config.rs:59-61`):

| Setting | Default |
|---------|---------|
| Host    | `127.0.0.1` |
| Port    | `8080` |
| Path    | `/mcp` |

So the default endpoint is:

```
http://127.0.0.1:8080/mcp
```

The server logs the concrete address at startup
(`src/server.rs:103-106`):
`MCP Streamable HTTP server listening on http://<host>:<port><path> ...`.
Host/port/path can be changed via config file, env (`MCP__…`), or the
global `--host` / `--port` / `--mcp-path` flags — and `service run`
honors those global flags (see next section). `server.http.path` defaults
to `/mcp` and the router nests the Streamable-HTTP service at that path
(`src/server.rs:136`).

A quick smoke check from the machine itself (Streamable HTTP is not a
plain GET page — expect an MCP-protocol response, a 4xx method error, or
similar rather than a website; the point is that **something answers**):

```
curl -s -o /dev/null -w "%{http_code}\n" http://127.0.0.1:8080/mcp
```

For a real end-to-end check, point an MCP client at the URL or run
`web_search` against it.

### Localhost / auth caveat (0.2.0)

- **There is no bearer-token auth on the HTTP transport in 0.2.0.**
  `Server::run_http` (`src/server.rs:70-139`) sets CORS, Host-header
  allow-listing, SSE keep-alive, JSON-response mode, and legacy session
  mode — but no `Authorization` / token check. (The plan already flagged
  this: `RELEASE_PLAN.md:46-47`, "HTTP transport has no auth".)
- The default bind (`127.0.0.1`) only exposes the loopback interface,
  which is the safe default — keep it unless you have a reason not to.
- **Warning about `0.0.0.0`:** binding `0.0.0.0` (e.g. `--host 0.0.0.0`)
  exposes the unauthenticated MCP endpoint on every network interface.
  Only do this behind a **reverse proxy with TLS and authentication**
  in front, and consider also setting `--allowed-hosts` (Host-header
  validation; defaults to `localhost,127.0.0.1` when unset,
  `src/server.rs:78-82`) and a restrictive `--cors-origin`
  (default `*`, `src/config.rs:55-57`).

---

## service run (hidden daemon entry)

- `gannet-mcp service run` is the **hidden daemon entry point**: it serves
  MCP over Streamable HTTP and **never STDIO**. Hiding is only cosmetic
  (`#[command(hide = true)]`, `src/service.rs:34-35`); parsing is unchanged
  (`src/main.rs:891-900` tests).
- Forcing is a one-liner: `apply_run_overrides` sets
  `config.server.use_stdio = false` (`src/service.rs:39-41`), and it wins
  even over an explicit `--stdio` flag
  (`src/main.rs:912-920` test: `--stdio service run` still ends up HTTP).
- **Users never invoke it directly.** All three supervisors launch it:
  systemd `ExecStart=/usr/bin/gannet-mcp service run`, the launchd plist
  `ProgramArguments` (`… service run`), and the Windows SCM `binPath`
  (`… service run`).
- Config handling differs subtly between the two `run` code paths:
  - The **binary** path (`src/main.rs:177-183`) loads the CLI-aware config
    (`load_config`: config file, env, **and** global flags like `--config`,
    `--host`, `--port`, `--mcp-path`) and then applies the
    HTTP-forcing override.
  - The **library** helper `load_service_config`
    (`src/service.rs:48-52`) loads only `Config::from_env()` (env/file,
    no CLI flags) and forces HTTP — it exists for library callers and
    testability, and is what the non-Windows `run_daemon` and the
    Windows interactive fallback use (`src/service.rs:267-272`,
    `src/service.rs:799-812`).
- Non-Windows `run_daemon` then calls `run_http_blocking`, which builds a
  fresh multi-thread Tokio runtime and serves `Server::run_http()`
  (`src/service.rs:278-289`).
- On Windows, `run` goes through the SCM dispatcher
  (`StartServiceCtrlDispatcher`): it reports `StartPending` during init,
  then `Running` (accepting `STOP`/`SHUTDOWN`), and serves the same HTTP
  daemon (`src/service.rs:757-792`). Stop/Shutdown currently exits the
  process promptly — "no graceful-shutdown plumbing yet (Phase 2)"
  (`src/service.rs:762-765`). If `run` is launched interactively instead
  of by the SCM, the dispatcher handshake fails with
  `ERROR_FAILED_SERVICE_CONTROLLER_CONNECT` (1063) and it falls back to a
  plain foreground HTTP run (`src/service.rs:794-812`).
