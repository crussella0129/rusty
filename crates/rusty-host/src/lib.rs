//! `rusty-host` — Rusty's single OS boundary.
//!
//! Phase 0 placeholder. This crate owns every OS-dependent interaction so the four
//! portable engine crates never have to: the PTY-attached sandbox shell
//! (`portable-pty`, Phase 1), the `cargo test --message-format=json` grading
//! subprocess (`cargo_metadata`, Phase 3), the `rust-analyzer` LSP subprocess
//! (`lsp-types`, Phase 4), and all sandboxed filesystem operations.
//!
//! Architectural contract (prompt §11): all `std::process`, raw filesystem, and
//! platform-specific code lives here (or in `rusty-terminal`) — nowhere else.

/// Crate identity marker, replaced by the PTY/subprocess/LSP host in Phases 1–4.
pub const CRATE_NAME: &str = "rusty-host";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crate_name() {
        assert_eq!(CRATE_NAME, "rusty-host");
    }
}
