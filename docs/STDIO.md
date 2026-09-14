# STDIO setup

This page covers STDIO transport for `gannet-mcp`: when to use it instead of
HTTP, and per-client setup via `gannet-mcp mcp-config`.

Source of truth for snippet shapes and write paths: `src/mcp_config.rs`
(module header docs). Release context: `RELEASE_PLAN.md` Phase 1
(`mcp-config`) and Phase 6 (this doc).

## 1. STDIO vs HTTP

| | STDIO | HTTP |
|---|---|---|
| Use when | Local inference engines / coding agents running **on the same machine** as `gannet-mcp` (Claude, Cursor, Zed, Opencode, Continue, Copilot) | Service mode: `gannet-mcp` runs as a daemon and serves remote or local HTTP clients |
| Default? | **Yes** — `use_stdio` defaults to `true`; bare `gannet-mcp` runs STDIO | Opt-in via `--http` (or `MCP__SERVER__USE_STDIO=false`) |
| Auth | None needed — local pipe between the client and the subprocess it spawns | See the HTTP-auth caveat below |

The two transports **coexist**: it is the same binary, just a different
transport selected at startup.

- STDIO (default): `gannet-mcp` or `gannet-mcp --stdio`
- HTTP: `gannet-mcp --http`
- Service (daemon) mode always runs HTTP: OS supervisors launch
  `gannet-mcp service run`, which forces `use_stdio = false`
  (see `src/service.rs`) regardless of CLI flags.

Note: if both `--stdio` and `--http` are passed to the **server**, `--http`
wins (`src/main.rs` applies `--stdio` first, then `--http`). For
`mcp-config` snippet rendering the rule differs — both flags render STDIO
(see §3 and Deviations).

## 2. Per-client STDIO setup

Every snippet below was generated with the actual binary — **not
hand-written**. Build once, then print per client:

```bash
cargo build
target/debug/gannet-mcp mcp-config --client <name> --print
```

The only edit applied to the output is replacing the local absolute binary
path with a `<path-to>/gannet-mcp` placeholder. Everything else (keys, key
order, whitespace) is byte-identical to what the binary printed. Each
snippet was validated by piping real binary output through
`python3 -c "import json,sys; json.load(sys.stdin)"`.

Replace `<path-to>` with the directory containing your `gannet-mcp` binary
(e.g. `/usr/bin`, `~/.local/bin`, or the `target/debug` path from your
build). Restart the client after editing its config so it respawns the
server subprocess.

### Claude

Generated with:

```bash
target/debug/gannet-mcp mcp-config --client claude --print   # valid JSON: yes
```

```json
{
  "mcpServers": {
    "gannet-mcp": {
      "args": [],
      "command": "<path-to>/gannet-mcp",
      "env": {}
    }
  }
}
```

### Cursor

Generated with:

```bash
target/debug/gannet-mcp mcp-config --client cursor --print   # valid JSON: yes
```

```json
{
  "mcpServers": {
    "gannet-mcp": {
      "args": [],
      "command": "<path-to>/gannet-mcp",
      "env": {}
    }
  }
}
```

### Zed

Generated with:

```bash
target/debug/gannet-mcp mcp-config --client zed --print   # valid JSON: yes
```

```json
{
  "context_servers": {
    "gannet-mcp": {
      "args": [],
      "command": "<path-to>/gannet-mcp",
      "env": {}
    }
  }
}
```

### Opencode

Generated with:

```bash
target/debug/gannet-mcp mcp-config --client opencode --print   # valid JSON: yes
```

```json
{
  "mcp": {
    "gannet-mcp": {
      "args": [],
      "command": "<path-to>/gannet-mcp",
      "env": {}
    }
  }
}
```

### Continue

Generated with:

```bash
target/debug/gannet-mcp mcp-config --client continue --print   # valid JSON: yes
```

```json
{
  "mcpServers": {
    "gannet-mcp": {
      "args": [],
      "command": "<path-to>/gannet-mcp",
      "env": {}
    }
  }
}
```

### Copilot (VS Code)

Generated with:

```bash
target/debug/gannet-mcp mcp-config --client copilot --print   # valid JSON: yes
```

```json
{
  "servers": {
    "gannet-mcp": {
      "args": [],
      "command": "<path-to>/gannet-mcp",
      "env": {}
    }
  }
}
```

### Adding secrets

Secrets are never passed as `argv`. Export the `MCP__*` variables in the
environment where you run `mcp-config` and they are copied into the
snippet's `env` block. Example (real binary output, key order as printed):

```bash
MCP__SEARCH__PROVIDER=serper MCP__SEARCH__API_KEY=s3cret \
  target/debug/gannet-mcp mcp-config --client claude --print
```

```json
{
  "mcpServers": {
    "gannet-mcp": {
      "args": [],
      "command": "<path-to>/gannet-mcp",
      "env": {
        "MCP__SEARCH__API_KEY": "s3cret",
        "MCP__SEARCH__PROVIDER": "serper"
      }
    }
  }
}
```

## 3. `mcp-config` usage

```bash
gannet-mcp mcp-config [--client claude|cursor|zed|opencode|continue|copilot] [--stdio|--http] [--print|--write]
```

- `--client`: which client's schema to emit. Accepted values are lowercase
  (`claude`, `cursor`, `zed`, `opencode`, `continue`, `copilot`); capitalised
  forms such as `Cursor` are rejected by clap. Omitted `--client` defaults
  to `opencode`.
- `--stdio` / `--http`: transport selection. STDIO is the default (neither
  flag, or `--stdio` alone, or **both** flags → STDIO); `--http` alone
  renders the HTTP form (`{"url": "http://127.0.0.1:8080/mcp", ...}` plus a
  `type` discriminator for opencode/copilot).
- `--print` (default) vs `--write`: `--print` writes the snippet to stdout
  and is the default whenever `--write` is absent. `--write` merges the
  `gannet-mcp` entry into the client's config file (creating parent
  directories; preserving other servers/settings), and `--print --write`
  together do both.
- Secrets: `MCP__*` environment variables are copied into the snippet's
  `env` block — never into `args`. Only `MCP__`-prefixed variables are
  picked up; everything else is ignored.

### Per-client write paths (`--write`)

From the `src/mcp_config.rs` module docs:

| Client | Top-level JSON key | Write path (`--write`) |
|--------|--------------------|------------------------|
| Claude | `mcpServers` | `~/.claude.json` (all OSes) |
| Cursor | `mcpServers` | `~/.cursor/mcp.json` (all OSes) |
| Opencode | `mcp` | Linux: `$XDG_CONFIG_HOME/opencode/opencode.json` (fallback `~/.config/opencode/opencode.json`); macOS: `~/Library/Application Support/opencode/opencode.json`; Windows: `%APPDATA%/opencode/opencode.json` |
| Zed | `context_servers` | Linux: `$XDG_CONFIG_HOME/zed/settings.json` (fallback `~/.config/zed/settings.json`); macOS: `~/Library/Application Support/Zed/settings.json` (fallback `~/.config/zed/settings.json`); Windows: `%APPDATA%/Zed/settings.json` — merged under `context_servers`, other settings preserved |
| Continue | `mcpServers` | `~/.continue/config.json` (all OSes) — merged under `mcpServers`; pre-existing array-form `mcpServers` is merged by `name` |
| Copilot (VS Code) | `servers` | User-level file **only if it already exists**, else falls back to `--print`: Linux: `$XDG_CONFIG_HOME/Code/User/mcp.json` (fallback `~/.config/Code/User/mcp.json`, then `~/.vscode/mcp.json`); macOS: `~/Library/Application Support/Code/User/mcp.json`; Windows: `%APPDATA%/Code/User/mcp.json`. A workspace `<workspace>/.vscode/mcp.json` is ambiguous from the CLI so it is never written automatically. |

## 4. HTTP-auth caveat

STDIO needs no auth: the client spawns `gannet-mcp` as a local subprocess
over a pipe, so there is nothing network-exposed to authenticate.

HTTP mode is different: it has **no bearer auth in 0.2.0**. Bind localhost
(the default) and expose it via a reverse proxy with TLS if remote access
is needed; avoid binding `0.0.0.0` on an untrusted network. Full warning in
`docs/USAGE.md` ("Security note" under HTTP Mode); bearer-token auth is a
planned follow-up (see `RELEASE_PLAN.md` Phase 0).

## 5. Known snippet-shape compromises

From the `src/mcp_config.rs` module header — the snippets above are
deliberately uniform rather than matching every client's native idiom, so
don't be surprised if they differ from a client's own docs:

- STDIO entries are always `{"command": <exe>, "args": [], "env": {...}}`.
  Native deviations, deliberately not followed:
  - opencode wants `"type": "local"` with `command` as an array and
    `environment` instead of `env`;
  - Zed (recent versions) nests the command as
    `{"command": {"path": …, "args": …, "env": …}}`;
  - Continue's classic form is an array of `{"name": …, "command": …}`
    objects (note: `--print` still emits the map form; only `--write`
    merges into an existing array-form file by `name`, and newly created
    files use the map form).
- HTTP entries are `{"url": …, "env": {...}}`, plus a type discriminator
  where the client schema uses one: opencode gets `"type": "remote"`,
  Copilot/VS Code gets `"type": "http"`; Claude/Cursor/Zed/Continue get no
  `type` key.
- The binary path comes from `std::env::current_exe()` (backslashes
  preserved on Windows, no separator normalisation); if `current_exe()`
  fails, the fallback command is the literal `gannet-mcp` (expected on
  `PATH`).
