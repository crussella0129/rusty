//! `rusty-curriculum` — the typed lesson/exercise model and content loader for Rusty.
//!
//! Phase 0 placeholder. The real `Lesson` / `Concept` / `Exercise` types and the
//! `lesson.toml` + Markdown loader land in Phase 2 (curriculum model + lesson 1).
//!
//! Portability contract: this crate must stay OS-portable. No `std::process`, no
//! raw filesystem access, no PTY — all of that lives in `rusty-host`.

/// Crate identity marker, replaced by the real curriculum types in Phase 2.
pub const CRATE_NAME: &str = "rusty-curriculum";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crate_name() {
        assert_eq!(CRATE_NAME, "rusty-curriculum");
    }
}
