//! End-to-end grading of intermediate lesson 6 (Advanced Diffing Climax).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use rusty_curriculum::SuccessCriterion;
use rusty_host::{grade, Verdict};

static COUNTER: AtomicU32 = AtomicU32::new(0);

fn lesson_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content/lessons/intermediate-06-diffing")
}

fn sandbox_from(which: &str, tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dst = std::env::temp_dir().join(format!("rusty_s27_{tag}_{nanos}_{n}"));
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
fn test_intermediate6_starter_fails() {
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
fn test_intermediate6_step2_faded_solution_passes() {
    let sandbox = sandbox_from("solution", "sol_step2");
    
    // We comment out the Step 3 test to ensure we only grade Step 2.
    let main_path = sandbox.join("src/main.rs");
    let content = std::fs::read_to_string(&main_path).unwrap();
    let pruned = comment_out_from(&content, "    #[test]\n    fn test_compute_diff()");
    std::fs::write(&main_path, pruned).unwrap();

    let verdict = grade(
        &sandbox,
        &SuccessCriterion::CargoTestPasses,
    )
    .unwrap();
    assert_eq!(verdict, Verdict::Pass);
    std::fs::remove_dir_all(&sandbox).ok();
}

#[test]
fn test_intermediate6_step3_open_solution_passes() {
    let sandbox = sandbox_from("solution", "sol_step3");
    let verdict = grade(
        &sandbox,
        &SuccessCriterion::CargoTestPasses,
    )
    .unwrap();
    assert_eq!(verdict, Verdict::Pass);
    std::fs::remove_dir_all(&sandbox).ok();
}
