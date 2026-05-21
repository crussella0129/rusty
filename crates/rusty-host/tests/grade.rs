//! Integration tests for the cargo grade runner — real `cargo` invocations against
//! minimal temp projects (each its own `[workspace]`, in OS temp outside the repo).
//! Each test spawns a cold `cargo` build, so they are a few seconds each.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use rusty_curriculum::SuccessCriterion;
use rusty_host::{grade, Verdict};

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// Create a temp cargo project (in OS temp, with its own `[workspace]`) from a list
/// of (relative path, contents) files plus a default `Cargo.toml`.
fn temp_project(tag: &str, name: &str, files: &[(&str, &str)]) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("rusty_s3_{tag}_{nanos}_{n}"));
    write(
        &dir.join("Cargo.toml"),
        &format!("[workspace]\n[package]\nname=\"{name}\"\nversion=\"0.0.0\"\nedition=\"2021\"\n"),
    );
    for (rel, contents) in files {
        write(&dir.join(rel), contents);
    }
    dir
}

fn write(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, contents).unwrap();
}

#[test]
fn test_grade_passing_test() {
    let dir = temp_project(
        "pass",
        "passproj",
        &[(
            "src/lib.rs",
            "pub fn add(a: i32, b: i32) -> i32 { a + b }\n\
             #[cfg(test)] mod t { use super::*; #[test] fn ok() { assert_eq!(add(2,2), 4); } }",
        )],
    );
    let verdict = grade(&dir, &SuccessCriterion::CargoTestPasses).unwrap();
    assert_eq!(verdict, Verdict::Pass);
}

#[test]
fn test_grade_borrow_error_is_e0382() {
    let dir = temp_project(
        "e0382",
        "borrowproj",
        &[(
            "src/main.rs",
            "fn main() {\n    let s = String::from(\"hi\");\n    let _t = s;\n    println!(\"{}\", s);\n}",
        )],
    );
    match grade(&dir, &SuccessCriterion::CargoTestPasses).unwrap() {
        Verdict::CompileError(diags) => {
            assert!(
                diags.iter().any(|d| d.code.as_deref() == Some("E0382")),
                "expected an E0382 diagnostic, got {diags:?}"
            );
        }
        other => panic!("expected CompileError, got {other:?}"),
    }
}

#[test]
fn test_grade_failing_test_is_tests_failed() {
    let dir = temp_project(
        "fail",
        "failproj",
        &[(
            "src/lib.rs",
            "#[cfg(test)] mod t { #[test] fn nope() { assert!(false); } }",
        )],
    );
    let verdict = grade(&dir, &SuccessCriterion::CargoTestPasses).unwrap();
    assert_eq!(verdict, Verdict::TestsFailed);
}

#[test]
fn test_grade_nonexistent_sandbox_errors() {
    // `grade()` returns Result precisely so spawn/exec failures propagate rather than
    // being silently graded as Pass.
    let missing = std::env::temp_dir().join("rusty_s3_does_not_exist_zzz");
    assert!(grade(&missing, &SuccessCriterion::CargoTestPasses).is_err());
}

#[test]
fn test_grade_run_output_match_and_mismatch() {
    let dir = temp_project(
        "run",
        "runproj",
        &[("src/main.rs", "fn main() { println!(\"rusty_marker\"); }")],
    );
    let ok = grade(
        &dir,
        &SuccessCriterion::CargoRunOutputMatches {
            expected: "rusty_marker".to_string(),
        },
    )
    .unwrap();
    assert_eq!(ok, Verdict::Pass);

    let bad = grade(
        &dir,
        &SuccessCriterion::CargoRunOutputMatches {
            expected: "something else".to_string(),
        },
    )
    .unwrap();
    assert!(matches!(bad, Verdict::RunMismatch { .. }));
}
