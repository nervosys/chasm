// Copyright (c) 2024-2026 Nervosys LLC
// SPDX-License-Identifier: Apache-2.0
//! TUI (Text User Interface) module for interactive browsing of chat sessions
//!
//! Provides color-coded tables and interactive navigation for VS Code Copilot Chat sessions.

mod app;
mod events;
mod ui;

pub use events::run_tui;
