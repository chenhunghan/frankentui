#![forbid(unsafe_code)]

//! FrankenTUI Runtime
//!
//! This crate provides the runtime components that tie together the core,
//! render, and layout crates into a complete terminal application framework.
//!
//! # Key Components
//!
//! - [`TerminalWriter`] - Unified terminal output coordinator with inline mode support
//! - [`LogSink`] - Line-buffered writer for sanitized log output
//! - [`Program`] - Bubbletea/Elm-style runtime for terminal applications
//! - [`Model`] - Trait for application state and behavior
//! - [`Cmd`] - Commands for side effects

pub mod log_sink;
pub mod program;
pub mod terminal_writer;

pub use log_sink::LogSink;
pub use program::{App, AppBuilder, Cmd, Model, Program, ProgramConfig};
pub use terminal_writer::{ScreenMode, TerminalWriter, UiAnchor};
