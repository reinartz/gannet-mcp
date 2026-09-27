# Install gannet-mcp (v0.2.0)

All links below point at the [v0.2.0 release](https://github.com/reinartz/gannet-mcp/releases/tag/v0.2.0).
Verify with `gannet-mcp --version` → `gannet-mcp 0.2.0`.

## Option 1 — cargo install (any OS with Rust)

```bash
cargo install gannet-mcp
```

Uses [crates.io/crates/gannet-mcp](https://crates.io/crates/gannet-mcp) (version 0.2.0 published).
Requires Rust 1.70+.

## Option 2 — Installer script (recommended for prebuilt binaries)

### Linux / macOS

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/reinartz/gannet-mcp/releases/download/v0.2.0/gannet-mcp-installer.sh | sh
```

### Windows (PowerShell)

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/reinartz/gannet-mcp/releases/download/v0.2.0/gannet-mcp-installer.ps1 | iex"
```

The installer detects your platform, downloads the matching archive from the
v0.2.0 release, and installs `gannet-mcp` into `$HOME/.cargo/bin`
(`$env:CARGO_HOME/bin` on Windows), adding it to `PATH`.

## Option 3 — Manual download (per-platform archives)

Download the archive for your platform from the
[v0.2.0 release page](https://github.com/reinartz/gannet-mcp/releases/tag/v0.2.0),
unpack it, and place `gannet-mcp` somewhere on your `PATH`.

| OS | Arch | Archive |
|----|------|---------|
| Linux | x86_64 | [gannet-mcp-x86_64-unknown-linux-gnu.tar.xz](https://github.com/reinartz/gannet-mcp/releases/download/v0.2.0/gannet-mcp-x86_64-unknown-linux-gnu.tar.xz) |
| Linux | aarch64 | [gannet-mcp-aarch64-unknown-linux-gnu.tar.xz](https://github.com/reinartz/gannet-mcp/releases/download/v0.2.0/gannet-mcp-aarch64-unknown-linux-gnu.tar.xz) |
| macOS | x86_64 (Intel) | [gannet-mcp-x86_64-apple-darwin.tar.xz](https://github.com/reinartz/gannet-mcp/releases/download/v0.2.0/gannet-mcp-x86_64-apple-darwin.tar.xz) |
| macOS | aarch64 (Apple Silicon) | [gannet-mcp-aarch64-apple-darwin.tar.xz](https://github.com/reinartz/gannet-mcp/releases/download/v0.2.0/gannet-mcp-aarch64-apple-darwin.tar.xz) |
| Windows | x86_64 | [gannet-mcp-x86_64-pc-windows-msvc.zip](https://github.com/reinartz/gannet-mcp/releases/download/v0.2.0/gannet-mcp-x86_64-pc-windows-msvc.zip) |

Each archive has a matching `.sha256` checksum file alongside it on the release page
(e.g. `gannet-mcp-x86_64-unknown-linux-gnu.tar.xz.sha256`).

### Linux / macOS

```bash
# Example: Linux x86_64 (swap in your archive name from the table above)
curl -LO https://github.com/reinartz/gannet-mcp/releases/download/v0.2.0/gannet-mcp-x86_64-unknown-linux-gnu.tar.xz
tar -xJf gannet-mcp-x86_64-unknown-linux-gnu.tar.xz
sudo install -m 755 gannet-mcp /usr/local/bin/gannet-mcp
gannet-mcp --version
```

### Windows (PowerShell)

```powershell
# Example: Windows x86_64
Invoke-WebRequest -Uri https://github.com/reinartz/gannet-mcp/releases/download/v0.2.0/gannet-mcp-x86_64-pc-windows-msvc.zip -OutFile gannet-mcp.zip
Expand-Archive gannet-mcp.zip -DestinationPath $env:USERPROFILE\bin
& "$env:USERPROFILE\bin\gannet-mcp.exe" --version
```

## Option 4 — Build from source (any OS)

```bash
git clone https://github.com/reinartz/gannet-mcp.git
cd gannet-mcp
cargo build --release
# Binary at target/release/gannet-mcp
```

## Option 5 — RPM, the hard way (Fedora/RHEL, local build)

Builds an RPM locally from the spec file (requires `rpm-build`, Rust, Cargo;
see `make install-rpm-deps`):

```bash
make rpm
```

Details live in `gannet-mcp.spec` and the `Makefile`. This is a local build, not
a hosted RPM repository.

## Option 6 — Homebrew (macOS / Linux)

```bash
brew install reinartz/formulae/gannet-mcp
```

Formula lives in the [`reinartz/homebrew-formulae`](https://github.com/reinartz/homebrew-formulae) tap (binary bottles for all four macOS/Linux targets, auto-bumped on future releases).

### Maintainer check (run on a Mac before announcing a release)

```bash
# Lint the formula (strict, as a new-formula review would)
brew audit --strict --new reinartz/formulae/gannet-mcp

# Test-install the bottle on this Mac (repeat on Intel + ARM if possible)
brew install reinartz/formulae/gannet-mcp
gannet-mcp --version          # must print the released version, exit 0
gannet-mcp mcp-config --client opencode --print | python3 -c "import json,sys; json.load(sys.stdin); print('JSON_OK')"

# Run the formula's own test block
brew test reinartz/formulae/gannet-mcp

# Clean up afterwards (optional)
brew uninstall gannet-mcp
```

All four commands must succeed. If `audit` flags anything, fix it in [`reinartz/homebrew-formulae`](https://github.com/reinartz/homebrew-formulae) (`Formula/gannet-mcp.rb`) — never in this repo.

Status (v0.2.0): verified 2026-09-14 on macOS (arm64) — `audit`, bottle
install plus `--version` / `mcp-config` JSON check, and `brew test` all pass.
Note: on Homebrew 7.x run `brew tap reinartz/formulae` and
`brew trust reinartz/formulae` first, otherwise `audit`/`install` refuse to
resolve the formula as untrusted.

## Coming later / roadmap

- **winget** package — manifest PR [microsoft/winget-pkgs#434439](https://github.com/microsoft/winget-pkgs/pull/434439) is open and passed automated validation; pending Microsoft reviewer merge. Not installable via `winget` yet.
- **apt / OBS repository** — deliberately skipped (no hosted repos; release assets + Homebrew + winget cover installation).
- **Windows MSI installer** (new in v0.2.1): download
  [gannet-mcp-x86_64-pc-windows-msvc.msi](https://github.com/reinartz/gannet-mcp/releases/download/v0.2.1/gannet-mcp-x86_64-pc-windows-msvc.msi)
  from the [v0.2.1 release page](https://github.com/reinartz/gannet-mcp/releases/tag/v0.2.1)
  and run it. It installs `gannet-mcp.exe` (default `%ProgramFiles%\gannet-mcp\`)
  and adds it to `PATH`. The MSI is unsigned (signing is deferred), so expect a
  Windows SmartScreen warning. Service mode is not set up by the installer: after
  installing, open an elevated (Administrator) prompt and run
  `gannet-mcp service install` (see `docs/SERVICES.md`).
