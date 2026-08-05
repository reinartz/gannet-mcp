# Contributing to gannet-mcp

Thank you for your interest in contributing to gannet-mcp! This document outlines the guidelines for contributing.

## Getting Started

### Prerequisites

- Rust 1.70+ (with Cargo)
- OpenSSL development headers (for TLS support)
- A GitHub account

### Setup

```bash
# Fork and clone the repository
git clone https://github.com/reinartz/gannet-mcp.git
cd gannet-mcp

# Build in release mode
cargo build --release

# Run the test suite
cargo test
```

## Development Workflow

### Code Style

- Run `cargo fmt` before submitting changes
- Run `cargo clippy -- -D warnings` to check for lint issues
- All code must pass formatting and linting checks

### Testing

- Write unit tests alongside source files
- Write integration tests in the `tests/` directory
- Run all tests before submitting: `cargo test -- --nocapture`

### Commit Messages

Follow conventional commit format:

```
feat: add support for SearXNG search provider
fix: handle timeout errors in fetch service
docs: update configuration reference
test: add integration test for HTTP mode
chore: update dependencies
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

## Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Make your changes
4. Run `cargo test`, `cargo clippy -- -D warnings`, and `cargo fmt -- --check`
5. Commit your changes with clear messages
6. Push to your fork and open a Pull Request

### PR Checklist

- [ ] Tests pass (`cargo test`)
- [ ] No clippy warnings (`cargo clippy -- -D warnings`)
- [ ] Code is formatted (`cargo fmt -- --check`)
- [ ] Release build succeeds (`cargo build --release`)
- [ ] Documentation updated (if applicable)
- [ ] CHANGELOG.md updated (if applicable)

## Architecture Overview

The codebase is organized into:

- **`src/models/`** - Pure data structures with serialization
- **`src/services/`** - Business logic and external API integrations
- **`src/handlers/`** - MCP protocol request handling
- **`src/`** - Server core, configuration, and error types

## Adding a New Search Provider

1. Implement the `SearchProvider` trait in `src/services/`
2. Add configuration support in `src/config.rs`
3. Update the handler in `src/handlers/` if needed
4. Add integration tests in `tests/search_provider_test.rs`
5. Update `README.md` and `docs/CONFIGURATION.md`

## Reporting Issues

Please use the GitHub issue templates:

- [Bug Report](.github/ISSUE_TEMPLATE/bug_report.md)
- [Feature Request](.github/ISSUE_TEMPLATE/feature_request.md)

## License

By contributing, you agree that your contributions will be licensed under:

- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

## Getting Help

- Open an issue for bugs or feature requests
- Check existing issues before creating new ones
- Be respectful and follow the code of conduct
