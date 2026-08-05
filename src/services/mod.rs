// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Ole Reinartz
//!
//! Service modules for external integrations

pub mod fetch_service;
pub mod search_service;

pub use fetch_service::FetchService;
pub use search_service::SearchService;
