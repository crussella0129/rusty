//! End-to-end grading of lesson 1's authored exercises (T-405): the Faded exercise
//! (`cargo test`) and the Open exercise (`cargo run` output) are graded against real
//! copies of the lesson's `starter/` and `solution/` projects. Also verifies the
//! C-002 live-link path: the *unfilled* Faded starter yields E0425 → a concept link to
//! the loaded lesson. Real `cargo` runs — a few seconds each.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use rusty_curriculum::SuccessCriterion;
use rusty_grader::annotate;
use rusty_host::{grade, Verdict};

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// The lesson directory, located relative to this crate's manifest.
fn lesson_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content/lessons/foundations-01-hello")
}

/// Copy the lesson's `which` project (`starter` or `solution`) into a fresh temp dir,
/// skipping any `target/`. Returns the sandbox path.
fn sandbox_from(which: &str, tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dst = std::env::temp_dir().join(format!("rusty_s4_{tag}_{nanos}_{n}"));
    copy_dir(&lesson_dir().join(which), &dst);
    dst
}

fn copy_dir(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        if entry.file_name() == "target" {
            continue;
        }
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir(&from, &to);
        } else {
            std::fs::copy(&from, &to).unwrap();
        }
    }
}

const OPEN_EXPECTED: &str = "I compiled my first Rust program!";

#[test]
fn test_lesson1_faded_solution_grades_pass() {
    let sandbox = sandbox_from("solution", "sol_test");
    let verdict = grade(&sandbox, &SuccessCriterion::CargoTestPasses).unwrap();
    assert_eq!(verdict, Verdict::Pass);
    std::fs::remove_dir_all(&sandbox).ok();
}

#[test]
fn test_lesson1_faded_starter_grades_not_pass() {
    // The unfilled starter calls `greeting()` (undefined) from its test → E0425.
    let sandbox = sandbox_from("starter", "starter_test");
    match grade(&sandbox, &SuccessCriterion::CargoTestPasses).unwrap() {
        Verdict::CompileError(diags) => assert!(
            diags
                .iter()
                .any(|d| matches!(d.code.as_deref(), Some("E0425") | Some("E0433"))),
            "expected an unresolved-name error, got {diags:?}"
        ),
        other => panic!("expected CompileError, got {other:?}"),
    }
    std::fs::remove_dir_all(&sandbox).ok();
}

#[test]
fn test_lesson1_open_solution_output_matches() {
    let sandbox = sandbox_from("solution", "open_test");
    let verdict = grade(
        &sandbox,
        &SuccessCriterion::CargoRunOutputMatches {
            expected: OPEN_EXPECTED.to_string(),
        },
    )
    .unwrap();
    assert_eq!(verdict, Verdict::Pass);
    std::fs::remove_dir_all(&sandbox).ok();
}

#[test]
fn test_lesson1_open_starter_output_mismatches() {
    // The starter still prints the welcome line, not the expected one → RunMismatch.
    let sandbox = sandbox_from("starter", "open_mismatch");
    let verdict = grade(
        &sandbox,
        &SuccessCriterion::CargoRunOutputMatches {
            expected: OPEN_EXPECTED.to_string(),
        },
    )
    .unwrap();
    assert!(matches!(verdict, Verdict::RunMismatch { .. }));
    std::fs::remove_dir_all(&sandbox).ok();
}

#[test]
fn test_faded_starter_annotates_live_link() {
    // C-002: grade the unfilled starter → CompileError(E0425) → annotate → a concept
    // link to the *authored* lesson 1 (so the live-link UI path is reachable now).
    let sandbox = sandbox_from("starter", "live_link");
    let verdict = grade(&sandbox, &SuccessCriterion::CargoTestPasses).unwrap();
    let annotation = annotate(&verdict);
    assert!(
        annotation
            .links
            .iter()
            .any(|l| l.lesson_id == "foundations-01-hello"),
        "the unresolved-name error should link to lesson 1, got {:?}",
        annotation.links
    );
    std::fs::remove_dir_all(&sandbox).ok();
}
