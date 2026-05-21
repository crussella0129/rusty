//! `rusty-scheduler` — the SM-2-lite spaced-repetition scheduler for Rusty.
//!
//! Phase 0 placeholder. The real per-concept `ease` / `interval_days` / `due_at`
//! state and the SM-2 update rule land in Phase 5 (recall, scheduling, persistence).
//!
//! Portability contract: this crate must stay OS-portable. No clock or filesystem
//! coupling baked in — time is passed in by the caller so the scheduler stays a
//! pure function of its inputs.

/// Crate identity marker, replaced by the real scheduler types in Phase 5.
pub const CRATE_NAME: &str = "rusty-scheduler";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crate_name() {
        assert_eq!(CRATE_NAME, "rusty-scheduler");
    }
}
