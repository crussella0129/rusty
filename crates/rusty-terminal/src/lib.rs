//! `rusty-terminal` — ANSI/VT100 rendering of the embedded shell on egui.
//!
//! Phase 0 placeholder. The Phase-1 spike decides the implementation: try the
//! `egui_term` widget first, and if it is not mature enough against the current
//! egui, fall back to a thin VT100 renderer built on `vte` for escape-sequence
//! parsing plus egui text widgets for drawing. Either way, the PTY bytes it renders
//! are produced by `rusty-host`.
//!
//! Architectural contract (prompt §11): OS/terminal-specific code is allowed here;
//! the portable engine crates must never depend on it.

/// Crate identity marker, replaced by the VT100 renderer in Phase 1.
pub const CRATE_NAME: &str = "rusty-terminal";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crate_name() {
        assert_eq!(CRATE_NAME, "rusty-terminal");
    }
}
