//! End-to-end grading of lesson 7's authored exercises (Error Handling).
//! Verifies that solution files grade as Pass and starter files grade as CompileError or RunMismatch.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use rusty_curriculum::SuccessCriterion;
use rusty_host::{grade, Verdict};

static COUNTER: AtomicU32 = AtomicU32::new(0);

fn lesson_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content/lessons/foundations-07-errors")
}

fn sandbox_from(which: &str, tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dst = std::env::temp_dir().join(format!("rusty_s14_{tag}_{nanos}_{n}"));
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

const FADED_EXPECTED: &str = "Result: 50\nFailed to parse: invalid digit found in string";
const OPEN_EXPECTED: &str = "Result: 50\nFailed to parse: invalid digit found in string\nNew balance: 50\nTransaction failed: Invalid amount format\nTransaction failed: Amount must be positive\nTransaction failed: Insufficient funds";

#[test]
fn test_lesson7_faded_solution_passes() {
    let sandbox = sandbox_from("solution", "sol_faded");
    let main_path = sandbox.join("src/main.rs");
    let content = std::fs::read_to_string(&main_path).unwrap();
    
    // Comment out the Open challenge part in the solution to isolate Faded output
    if let Some(pos) = content.rfind("// (Open)") {
        let mut new_content = content[..pos].to_string();
        new_content.push_str("/* // (Open)\n");
        let rest = &content[pos..];
        if let Some(brace_pos) = rest.rfind('}') {
            new_content.push_str(&rest[..brace_pos]);
            new_content.push_str("*/\n}");
        } else {
            new_content.push_str(rest);
        }
        std::fs::write(&main_path, new_content).unwrap();
    }

    let verdict = grade(
        &sandbox,
        &SuccessCriterion::CargoRunOutputMatches {
            expected: FADED_EXPECTED.to_string(),
        },
    )
    .unwrap();
    assert_eq!(verdict, Verdict::Pass);
    std::fs::remove_dir_all(&sandbox).ok();
}

#[test]
fn test_lesson7_faded_starter_fails_compile() {
    let sandbox = sandbox_from("starter", "starter_faded");
    // Since the starter is broken (no `?` in Faded), running tests should fail compilation.
    let verdict = grade(&sandbox, &SuccessCriterion::CargoTestPasses).unwrap();
    assert!(
        matches!(verdict, Verdict::CompileError(_)),
        "expected CompileError, got {:?}",
        verdict
    );
    std::fs::remove_dir_all(&sandbox).ok();
}

#[test]
fn test_lesson7_open_solution_passes() {
    let sandbox = sandbox_from("solution", "sol_open");
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
