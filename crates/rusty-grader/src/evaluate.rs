//! Bridges the curriculum's [`SuccessCriterion`] to a grading [`Verdict`], given the
//! captured output of the cargo commands `rusty-host` ran.

use rusty_curriculum::SuccessCriterion;

use crate::verdict::{grade_cargo_test, grade_run_output, Verdict};

/// The cargo output captured by `rusty-host` for one grading run.
#[derive(Debug, Clone, Default)]
pub struct CargoOutcome {
    /// `cargo test --message-format=json` stdout.
    pub test_json: String,
    /// Whether `cargo test` exited successfully.
    pub test_exit_ok: bool,
    /// `cargo run` stdout (only needed for `CargoRunOutputMatches`).
    pub run_stdout: String,
}

/// Evaluate a `SuccessCriterion` against captured cargo output.
pub fn evaluate(criterion: &SuccessCriterion, outcome: &CargoOutcome) -> Verdict {
    match criterion {
        SuccessCriterion::CargoTestPasses => {
            grade_cargo_test(&outcome.test_json, outcome.test_exit_ok)
        }
        SuccessCriterion::CargoRunOutputMatches { expected } => {
            grade_run_output(&outcome.run_stdout, expected)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_cargo_test_passes() {
        let outcome = CargoOutcome {
            test_json: String::new(), // no compiler messages
            test_exit_ok: true,
            run_stdout: String::new(),
        };
        assert_eq!(
            evaluate(&SuccessCriterion::CargoTestPasses, &outcome),
            Verdict::Pass
        );
    }

    #[test]
    fn test_evaluate_run_matches() {
        let outcome = CargoOutcome {
            run_stdout: "hi\n".to_string(),
            ..Default::default()
        };
        let crit = SuccessCriterion::CargoRunOutputMatches {
            expected: "hi".to_string(),
        };
        assert_eq!(evaluate(&crit, &outcome), Verdict::Pass);

        let crit_bad = SuccessCriterion::CargoRunOutputMatches {
            expected: "bye".to_string(),
        };
        assert!(matches!(
            evaluate(&crit_bad, &outcome),
            Verdict::RunMismatch { .. }
        ));
    }
}
