//! `rusty-terminal` — ANSI/VT100 rendering of the embedded shell on egui.
//!
//! Decided in the Phase-1 spike: roll our own renderer on `vte` (the published
//! `egui_term` 0.1.0 pins egui 0.31, incompatible with our 0.34). The pipeline is
//! PTY bytes → [`vte::Parser`] driving a [`Performer`] → a [`Grid`] of colored
//! [`Cell`]s → the [`terminal_ui`] egui widget. The grid/performer are pure and
//! unit-tested; only the painting needs a window.
//!
//! OS/terminal-specific code is allowed here (portability tripwire §11); the four
//! portable engine crates must never depend on it.

pub mod cell;
pub mod grid;
pub mod performer;
pub mod widget;

pub use cell::Cell;
pub use grid::Grid;
pub use performer::Performer;
pub use widget::{key_to_bytes, terminal_ui};
