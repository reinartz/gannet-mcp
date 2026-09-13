# Gannet-MCP Pre-Release Checklist

Tasks to complete before uploading to GitHub and crates.io.



## Publishing

- [x] **GitHub**
  - [x] Create GitHub repository (https://github.com/reinartz/gannet-mcp)
  - [x] Push to GitHub
  - [ ] Create initial release tag (v0.1.0)
  - [ ] Add release notes
- [ ] **crates.io**
  - [ ] Run `cargo publish --dry-run` to verify
  - [ ] Publish to crates.io with `cargo publish`
  - [ ] Verify crate page on crates.io

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
