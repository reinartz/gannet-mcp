// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Ole Reinartz
//!
//! MCP protocol handlers

pub mod fetch_handler;
pub mod search_handler;

pub use fetch_handler::{fetch_tool_definition, FetchHandler};
pub use search_handler::{search_tool_definition, SearchHandler};
