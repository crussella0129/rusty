//! End-to-end grading of lesson 8's authored exercises (Collections & File I/O).
//! Verifies that solution files grade as Pass and starter files grade as CompileError or RunMismatch.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use rusty_curriculum::SuccessCriterion;
use rusty_host::{grade, Verdict};

static COUNTER: AtomicU32 = AtomicU32::new(0);

fn lesson_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content/lessons/foundations-08-collections")
}

fn sandbox_from(which: &str, tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dst = std::env::temp_dir().join(format!("rusty_s16_{tag}_{nanos}_{n}"));
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

const STEP2_EXPECTED: &str = "New items: Purple";
const STEP3_EXPECTED: &str =
    "New items: Purple\nCombined uniques: Blue~Green~Orange~Purple~Red~Yellow";
const STEP4_EXPECTED: &str = "New items: Purple\nCombined uniques: Blue~Green~Orange~Purple~Red~Yellow\nFull diff: +Purple~-Orange";
const STEP5_EXPECTED: &str = "New items: Purple\nCombined uniques: Blue~Green~Orange~Purple~Red~Yellow\nFull diff: +Purple~-Orange\nPositional diff: +Purple@3~-Orange@1~>Green@3->2~>Yellow@2->1";

fn comment_out_from(content: &str, tag: &str) -> String {
    if let Some(pos) = content.find(tag) {
        let mut new_content = content[..pos].to_string();
        new_content.push_str(&format!("/* {tag}\n"));
        let rest = &content[pos + tag.len()..];
        if let Some(brace_pos) = rest.rfind('}') {
            new_content.push_str(&rest[..brace_pos]);
            new_content.push_str("*/\n}");
        } else {
            new_content.push_str(rest);
        }
        new_content
    } else {
        content.to_string()
    }
}

#[test]
fn test_lesson8_step2_faded_solution_passes() {
    let sandbox = sandbox_from("solution", "sol_step2");
    let main_path = sandbox.join("src/main.rs");
    let content = std::fs::read_to_string(&main_path).unwrap();

    // Comment out everything from Step 3 onwards
    let pruned = comment_out_from(&content, "// (Open - Step 3)");
    std::fs::write(&main_path, pruned).unwrap();

    let verdict = grade(
        &sandbox,
        &SuccessCriterion::CargoRunOutputMatches {
            expected: STEP2_EXPECTED.to_string(),
        },
    )
    .unwrap();
    assert_eq!(verdict, Verdict::Pass);
    std::fs::remove_dir_all(&sandbox).ok();
}

#[test]
fn test_lesson8_step3_open_solution_passes() {
    let sandbox = sandbox_from("solution", "sol_step3");
    let main_path = sandbox.join("src/main.rs");
    let content = std::fs::read_to_string(&main_path).unwrap();

    // Comment out everything from Step 4 onwards
    let pruned = comment_out_from(&content, "// (Faded - Step 4)");
    std::fs::write(&main_path, pruned).unwrap();

    let verdict = grade(
        &sandbox,
        &SuccessCriterion::CargoRunOutputMatches {
            expected: STEP3_EXPECTED.to_string(),
        },
    )
    .unwrap();
    assert_eq!(verdict, Verdict::Pass);
    std::fs::remove_dir_all(&sandbox).ok();
}

#[test]
fn test_lesson8_step4_faded_solution_passes() {
    let sandbox = sandbox_from("solution", "sol_step4");
    let main_path = sandbox.join("src/main.rs");
    let content = std::fs::read_to_string(&main_path).unwrap();

    // Comment out everything from Step 5 onwards
    let pruned = comment_out_from(&content, "// (Open - Step 5)");
    std::fs::write(&main_path, pruned).unwrap();

    let verdict = grade(
        &sandbox,
        &SuccessCriterion::CargoRunOutputMatches {
            expected: STEP4_EXPECTED.to_string(),
        },
    )
    .unwrap();
    assert_eq!(verdict, Verdict::Pass);
    std::fs::remove_dir_all(&sandbox).ok();
}

#[test]
fn test_lesson8_step5_open_solution_passes() {
    let sandbox = sandbox_from("solution", "sol_step5");
    let verdict = grade(
        &sandbox,
        &SuccessCriterion::CargoRunOutputMatches {
            expected: STEP5_EXPECTED.to_string(),
        },
    )
    .unwrap();
    assert_eq!(verdict, Verdict::Pass);
    std::fs::remove_dir_all(&sandbox).ok();
}

#[test]
fn test_lesson8_starter_fails() {
    let sandbox = sandbox_from("starter", "starter_fail");

    // Test that the unmodified starter fails Step 2
    let verdict = grade(
        &sandbox,
        &SuccessCriterion::CargoRunOutputMatches {
            expected: STEP2_EXPECTED.to_string(),
        },
    )
    .unwrap();
    assert!(
        matches!(verdict, Verdict::RunMismatch { .. }),
        "expected RunMismatch for starter, got {:?}",
        verdict
    );
    std::fs::remove_dir_all(&sandbox).ok();
}
