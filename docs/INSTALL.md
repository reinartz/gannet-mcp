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

## Coming later / roadmap (do NOT exist yet)

The following channels are planned (see `RELEASE_PLAN.md` Phases 3–5) but **do
not exist for v0.2.0** — there is nothing to install from them yet:

- **Homebrew tap** (`reinartz/homebrew-formulae`) — does not exist yet.
- **winget** package — does not exist yet.
- **apt / OBS repository** (`sources.list.d` + signing key) — does not exist yet.
- **Native packages**: `.deb`, `.pkg`, `.msi` — not shipped; the v0.2.0 release
  carries only the portable `.tar.xz` / `.zip` archives listed above.
