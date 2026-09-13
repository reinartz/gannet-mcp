# Gannet-MCP Pre-Release Checklist

Tasks to complete before uploading to GitHub and crates.io.



## Publishing

- [x] **GitHub**
  - [x] Create GitHub repository (https://github.com/reinartz/gannet-mcp)
  - [x] Push to GitHub
  - [x] Create initial release tag (v0.1.0)
  - [x] Add release notes (https://github.com/reinartz/gannet-mcp/releases/tag/v0.1.0)
- [x] **crates.io** (gannet-mcp 0.1.0 live since 2026-08-05)
  - [x] Run `cargo publish --dry-run` to verify
  - [x] Publish to crates.io with `cargo publish`
  - [x] Verify crate page on crates.io

## Post-Publish (Optional)

- [ ] Add badges to README.md (crates.io version, downloads, CI status, license)
- [ ] Set up GitHub Actions CI/CD badges
- [ ] Create GitHub Wiki for extended documentation
- [ ] Add project to GitHub Topics (rust, mcp, web-search, etc.)

---

## Notes

- **License**: Dual-licensed MIT OR Apache-2.0 (already configured)
- **Tests**: 358 unit/integration tests (all passing)
- **MVP Features**: web_search, web_fetch tools with multiple search providers
- **Target Platforms**: Linux, macOS, Windows
