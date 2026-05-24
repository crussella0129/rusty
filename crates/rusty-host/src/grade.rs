//! Process #2 of the three-process model: run `cargo` in a lesson sandbox and grade
//! the result. This is a separate, invisible subprocess — not the learner's PTY shell.
//! The OS spawn lives here; the verdict logic lives in the portable `rusty-grader`.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use rusty_curriculum::SuccessCriterion;
use rusty_grader::{evaluate, CargoOutcome, Verdict};

/// Resolve the sandbox's `Cargo.toml` path. We pass this via `--manifest-path` on every
/// grader cargo call so cargo cannot **escalate** up the directory tree to find an
/// ancestor workspace's manifest (the s7 Reveal-Pass bug: an empty sandbox let
/// `cargo test` build the outer Rusty workspace's tests, all of which passed, and the
/// grader saw a clean exit and reported `Verdict::Pass` for the wrong code). With
/// `--manifest-path`, a missing manifest is a hard cargo error surfaced as a grade
/// `Err` instead of a silent Pass.
fn manifest_path(sandbox: &Path) -> PathBuf {
    sandbox.join("Cargo.toml")
}

/// Fail fast (as a grade `Err`, not a misleading `Verdict::TestsFailed`) if the sandbox
/// manifest is missing — a regressed sandbox prep would otherwise produce zero-diags +
/// non-zero exit, which maps to `TestsFailed` and hides the real cause.
fn require_manifest(manifest: &Path) -> Result<()> {
    anyhow::ensure!(
        manifest.is_file(),
        "sandbox manifest missing — expected {} (sandbox prep regressed, see s7 ADR)",
        manifest.display()
    );
    Ok(())
}

/// Run `cargo test --message-format=json` against the sandbox manifest; return
/// (json stdout, exit-ok).
pub fn run_cargo_test(sandbox: &Path) -> Result<(String, bool)> {
    let manifest = manifest_path(sandbox);
    require_manifest(&manifest)?;
    let output = Command::new("cargo")
        .args(["test", "--message-format=json", "--manifest-path"])
        .arg(&manifest)
        .current_dir(sandbox)
        // Build in the sandbox's own `target/`, never an inherited CARGO_TARGET_DIR
        // (which would let concurrent grades collide and leak artifacts).
        .env_remove("CARGO_TARGET_DIR")
        .output()
        .with_context(|| format!("running cargo test in {}", sandbox.display()))?;
    let json = String::from_utf8_lossy(&output.stdout).into_owned();
    Ok((json, output.status.success()))
}

/// Run `cargo run -q` against the sandbox manifest; return its captured stdout.
pub fn run_cargo_run(sandbox: &Path) -> Result<String> {
    let manifest = manifest_path(sandbox);
    require_manifest(&manifest)?;
    let output = Command::new("cargo")
        .args(["run", "-q", "--manifest-path"])
        .arg(&manifest)
        .current_dir(sandbox)
        .env_remove("CARGO_TARGET_DIR")
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
