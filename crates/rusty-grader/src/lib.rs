//! `rusty-grader` — exercise grading rules for Rusty.
//!
//! Phase 0 placeholder. The real grader (the `SuccessCriterion` evaluator over a
//! parsed `cargo test --message-format=json` result, plus optional `syn`-based AST
//! style hints) lands in Phase 3 (editor + grader + diagnostics).
//!
//! Portability contract: this crate grades *already-captured* cargo output passed in
//! by `rusty-host`; it never spawns `cargo` itself. No `std::process` here.

/// Crate identity marker, replaced by the real grading types in Phase 3.
pub const CRATE_NAME: &str = "rusty-grader";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crate_name() {
        assert_eq!(CRATE_NAME, "rusty-grader");
    }
}
