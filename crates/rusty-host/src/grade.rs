//! Process #2 of the three-process model: run `cargo` in a lesson sandbox and grade
//! the result. This is a separate, invisible subprocess — not the learner's PTY shell.
//! The OS spawn lives here; the verdict logic lives in the portable `rusty-grader`.

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use rusty_curriculum::SuccessCriterion;
use rusty_grader::{evaluate, CargoOutcome, Verdict};

/// Run `cargo test --message-format=json` in `sandbox`; return (json stdout, exit-ok).
pub fn run_cargo_test(sandbox: &Path) -> Result<(String, bool)> {
    let output = Command::new("cargo")
        .args(["test", "--message-format=json"])
        .current_dir(sandbox)
        .output()
        .with_context(|| format!("running cargo test in {}", sandbox.display()))?;
    let json = String::from_utf8_lossy(&output.stdout).into_owned();
    Ok((json, output.status.success()))
}

/// Run `cargo run -q` in `sandbox`; return its captured stdout.
pub fn run_cargo_run(sandbox: &Path) -> Result<String> {
    let output = Command::new("cargo")
        .args(["run", "-q"])
        .current_dir(sandbox)
        .output()
        .with_context(|| format!("running cargo run in {}", sandbox.display()))?;
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Grade the sandbox against a success criterion, running only the cargo command the
/// criterion needs.
pub fn grade(sandbox: &Path, criterion: &SuccessCriterion) -> Result<Verdict> {
    let outcome = match criterion {
        SuccessCriterion::CargoTestPasses => {
            let (test_json, test_exit_ok) = run_cargo_test(sandbox)?;
            CargoOutcome {
                test_json,
                test_exit_ok,
                run_stdout: String::new(),
            }
        }
        SuccessCriterion::CargoRunOutputMatches { .. } => {
            let run_stdout = run_cargo_run(sandbox)?;
            CargoOutcome {
                run_stdout,
                ..Default::default()
            }
        }
    };
    Ok(evaluate(criterion, &outcome))
}
