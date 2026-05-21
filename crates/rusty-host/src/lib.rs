//! `rusty-host` — Rusty's single OS boundary.
//!
//! This crate owns every OS-dependent interaction so the four portable engine crates
//! never have to: the PTY-attached sandbox shell ([`pty`], `portable-pty`), the
//! `cargo test --message-format=json` grading subprocess (Phase 3), the
//! `rust-analyzer` LSP subprocess (Phase 4), and all sandboxed filesystem operations.
//!
//! Architectural contract (prompt §11): all `std::process`, raw filesystem, and
//! platform-specific code lives here (or in `rusty-terminal`) — nowhere else.

pub mod pty;
pub mod sandbox;

pub use pty::PtySession;
pub use sandbox::{default_shell, resolve_cd, CdOutcome};

/// Crate identity marker (kept for the original skeleton test; harmless).
pub const CRATE_NAME: &str = "rusty-host";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crate_name() {
        assert_eq!(CRATE_NAME, "rusty-host");
    }
}
