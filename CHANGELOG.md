# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-08-05

### Added
- Web search tool with support for multiple providers:
   - DuckDuckGo (default, no API key required)
   - Serper (requires `api_key`)
   - SearXNG (self-hosted, defaults to `http://127.0.0.1:8888`)
   - BrightData (requires `api_key` and `search_engine_id`)
- Webpage fetch tool with HTML parsing and content extraction
- Link and image extraction from fetched pages
- Metadata extraction (title, description)
- STDIO transport mode (default)
- Streamable HTTP transport mode
- Rate limiting with configurable limits
- Configurable timeouts
- URL validation and safe search options
- Content size limits
- Dual licensing (MIT OR Apache-2.0)
- RPM packaging support
- Comprehensive test suite (358+ tests)
