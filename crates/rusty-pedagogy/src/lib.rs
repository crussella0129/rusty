//! `rusty-pedagogy` — research citations and helpers that keep Rusty's claims honest.
//!
//! Phase 0 placeholder. This crate eventually holds the typed `Reference` values
//! behind each lesson's `further_reading` and the citations that back the four
//! learning principles, cross-checked against `docs/PEDAGOGY.md`.
//!
//! Portability contract: pure data and helpers, OS-portable, no filesystem coupling.

/// Crate identity marker, replaced by the real citation types in later phases.
pub const CRATE_NAME: &str = "rusty-pedagogy";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crate_name() {
        assert_eq!(CRATE_NAME, "rusty-pedagogy");
    }
}
