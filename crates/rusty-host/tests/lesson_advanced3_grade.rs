//! End-to-end grading of advanced lesson 3 (Advanced Traits).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use rusty_curriculum::SuccessCriterion;
use rusty_host::{grade, Verdict};

static COUNTER: AtomicU32 = AtomicU32::new(0);

fn lesson_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content/lessons/advanced-03-traits")
}

fn sandbox_from(which: &str, tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dst = std::env::temp_dir().join(format!("rusty_a3_{tag}_{nanos}_{n}"));
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
fn test_advanced3_starter_fails() {
    let sandbox = sandbox_from("starter", "starter_fail");
    let verdict = grade(&sandbox, &SuccessCriterion::CargoTestPasses).unwrap();
    assert!(
        matches!(verdict, Verdict::TestsFailed | Verdict::CompileError { .. }),
        "expected failure for starter, got {:?}",
        verdict
    );
    std::fs::remove_dir_all(&sandbox).ok();
}

#[test]
fn test_advanced3_step2_faded_solution_passes() {
    let sandbox = sandbox_from("starter", "sol_step2");

    // Solve step 2
    let main_rs = sandbox.join("src/main.rs");
    let code = std::fs::read_to_string(&main_rs).unwrap();
    let solved = code.replace(
        "impl Graph for MyGraph {\n    fn edges(&self, _node: &Self::Node) -> Vec<Self::Node> {\n        vec![]\n    }\n}",
        "impl Graph for MyGraph {\n    type Node = i32;\n    fn edges(&self, _node: &Self::Node) -> Vec<Self::Node> {\n        vec![]\n    }\n}",
    );

    // Comment out tests for steps 3 and 4
    let pruned = comment_out_from(&solved, "    #[test]\n    fn test_step_3()");
    std::fs::write(&main_rs, pruned).unwrap();

    let verdict = grade(&sandbox, &SuccessCriterion::CargoTestPasses).unwrap();
    assert_eq!(verdict, Verdict::Pass);
    std::fs::remove_dir_all(&sandbox).ok();
}

#[test]
fn test_advanced3_step3_faded_solution_passes() {
    let sandbox = sandbox_from("starter", "sol_step3");

    // Solve step 2
    let main_rs = sandbox.join("src/main.rs");
    let code = std::fs::read_to_string(&main_rs).unwrap();
    let solved2 = code.replace(
        "impl Graph for MyGraph {\n    fn edges(&self, _node: &Self::Node) -> Vec<Self::Node> {\n        vec![]\n    }\n}",
        "impl Graph for MyGraph {\n    type Node = i32;\n    fn edges(&self, _node: &Self::Node) -> Vec<Self::Node> {\n        vec![]\n    }\n}",
    );

    // Solve step 3 (implement Drop)
    let mut lines: Vec<&str> = solved2.lines().collect();
    let drop_impl = "impl Drop for CustomPointer {\n    fn drop(&mut self) {\n        println!(\"Dropping CustomPointer!\");\n    }\n}";

    for (i, line) in lines.iter().enumerate() {
        if line.contains("pub fn step_3() {") {
            lines.insert(i, drop_impl);
            break;
        }
    }
    let solved3 = lines.join("\n");

    // Remove the panic from test_step_3
    let solved3 = solved3.replace(
        "        // TODO: Implement Drop for CustomPointer, then delete this panic!\n        panic!(\"Implement Drop!\");",
        "        step_3();",
    );

    // Comment out test for step 4
    let pruned = comment_out_from(&solved3, "    #[test]\n    fn test_step_4()");
    std::fs::write(&main_rs, pruned).unwrap();

    let verdict = grade(&sandbox, &SuccessCriterion::CargoTestPasses).unwrap();
    assert_eq!(verdict, Verdict::Pass);
    std::fs::remove_dir_all(&sandbox).ok();
}

#[test]
fn test_advanced3_step4_open_solution_passes() {
    let sandbox = sandbox_from("solution", "sol_step4");

    let verdict = grade(&sandbox, &SuccessCriterion::CargoTestPasses).unwrap();
    assert_eq!(verdict, Verdict::Pass);
    std::fs::remove_dir_all(&sandbox).ok();
}
