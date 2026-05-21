//! Integration tests for content loading + sandbox preparation, using temp-dir
//! fixtures (no dependency on the repo's real `content/` — the real-lesson load test
//! lives alongside the authored lesson).

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use rusty_host::{load_lesson, prepare_sandbox};

static COUNTER: AtomicU32 = AtomicU32::new(0);

fn unique_temp(tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("rusty_s2_{tag}_{nanos}_{n}"));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(path: &std::path::Path, contents: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, contents).unwrap();
}

#[test]
fn test_prepare_sandbox_copies_starter() {
    let content_dir = unique_temp("content");
    write(
        &content_dir.join("starter").join("Cargo.toml"),
        "[package]\nname=\"x\"\nversion=\"0.0.0\"\nedition=\"2021\"\n[workspace]\n",
    );
    write(
        &content_dir.join("starter").join("src").join("main.rs"),
        "fn main() { println!(\"hi\"); }",
    );
    let workspace_root = unique_temp("ws");

    let sandbox = prepare_sandbox(&content_dir, &workspace_root, "lid").unwrap();

    assert!(sandbox.join("Cargo.toml").is_file());
    assert!(sandbox.join("src").join("main.rs").is_file());
    assert!(sandbox.ends_with(PathBuf::from("lessons").join("lid")));
}

#[test]
fn test_prepare_sandbox_idempotent() {
    let content_dir = unique_temp("content");
    write(&content_dir.join("starter").join("marker.txt"), "original");
    let workspace_root = unique_temp("ws");

    let first = prepare_sandbox(&content_dir, &workspace_root, "lid").unwrap();
    // Simulate a learner edit; a second prepare must NOT clobber it.
    std::fs::write(first.join("marker.txt"), "edited").unwrap();
    let second = prepare_sandbox(&content_dir, &workspace_root, "lid").unwrap();

    assert_eq!(first, second);
    assert_eq!(
        std::fs::read_to_string(second.join("marker.txt")).unwrap(),
        "edited"
    );
}

#[test]
fn test_load_lesson_from_temp() {
    let content_dir = unique_temp("lesson");
    write(
        &content_dir.join("lesson.toml"),
        r#"
            id = "temp-lesson"
            title = "Temp"
            track = "Foundations"
            estimated_minutes = 5
            starter_project = "starter"
            solution_project = "solution"

            [[body]]
            kind = "prose"
            text = "Body."

            [recall_prompt]
            kind = "short_answer"
            question = "q?"
            expected = "a"
            explanation = "because"
        "#,
    );

    let lesson = load_lesson(&content_dir).unwrap();
    assert_eq!(lesson.id.0, "temp-lesson");
    assert_eq!(lesson.title, "Temp");
}

/// Load the real authored lesson 1 from the repo's `content/` tree.
#[test]
fn test_load_lesson_real_content() {
    // CARGO_MANIFEST_DIR = .../crates/rusty-host; repo root is two levels up.
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("content")
        .join("lessons")
        .join("foundations-01-hello");
    let lesson = load_lesson(&dir).expect("real lesson 1 loads");
    assert_eq!(lesson.id.0, "foundations-01-hello");
    assert_eq!(lesson.title, "Hello, compiler.");
    assert!(!lesson.body.is_empty());
}
