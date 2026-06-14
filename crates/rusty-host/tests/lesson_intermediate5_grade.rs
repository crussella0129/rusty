//! End-to-end grading of intermediate lesson 5 (Serialization & Parsing).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use rusty_curriculum::SuccessCriterion;
use rusty_host::{grade, Verdict};

static COUNTER: AtomicU32 = AtomicU32::new(0);

fn lesson_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content/lessons/intermediate-05-parsing")
}

fn sandbox_from(which: &str, tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dst = std::env::temp_dir().join(format!("rusty_s26_{tag}_{nanos}_{n}"));
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

#[test]
fn test_intermediate5_starter_fails() {
    let sandbox = sandbox_from("starter", "starter_fail");
    let verdict = grade(
        &sandbox,
        &SuccessCriterion::CargoTestPasses,
    )
    .unwrap();
    assert!(
        matches!(verdict, Verdict::TestsFailed | Verdict::CompileError { .. }),
        "expected failure for starter, got {:?}",
        verdict
    );
    std::fs::remove_dir_all(&sandbox).ok();
}

#[test]
fn test_intermediate5_step2_faded_solution_passes() {
    let sandbox = sandbox_from("solution", "sol_step2");
    let verdict = grade(
        &sandbox,
        &SuccessCriterion::CargoTestPasses,
    )
    .unwrap();
    assert_eq!(verdict, Verdict::Pass);
    std::fs::remove_dir_all(&sandbox).ok();
}

#[test]
fn test_intermediate5_step3_open_solution_passes() {
    let sandbox = sandbox_from("solution", "sol_step3");
    let verdict = grade(
        &sandbox,
        &SuccessCriterion::CargoTestPasses,
    )
    .unwrap();
    assert_eq!(verdict, Verdict::Pass);
    std::fs::remove_dir_all(&sandbox).ok();
}
