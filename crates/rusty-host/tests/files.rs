//! Integration tests for the sandboxed editor file I/O (T-401). Uses real temp dirs
//! (no mocks), mirroring the s3 grade integration tests.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use rusty_host::{list_sandbox_files, read_sandbox_file, write_sandbox_file};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// A fresh, unique sandbox directory under the OS temp dir.
fn unique_sandbox(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("rusty-files-{tag}-{nanos}-{n}"));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn touch(path: &Path) {
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p).unwrap();
    }
    std::fs::write(path, "// fixture\n").unwrap();
}

#[test]
fn test_list_files_skips_target() {
    let sandbox = unique_sandbox("list");
    touch(&sandbox.join("src").join("main.rs"));
    touch(&sandbox.join("tests").join("t.rs"));
    touch(&sandbox.join("target").join("debug").join("x.rs"));
    touch(&sandbox.join("Cargo.toml")); // .toml is now allowed

    let files = list_sandbox_files(&sandbox).unwrap();

    assert_eq!(files.len(), 3, "exactly the three source files: {files:?}");
    assert!(files.iter().any(|p| p.ends_with("main.rs")));
    assert!(files.iter().any(|p| p.ends_with("t.rs")));
    assert!(files.iter().any(|p| p.ends_with("Cargo.toml")));
    assert!(
        !files
            .iter()
            .any(|p| p.components().any(|c| c.as_os_str() == "target")),
        "target/ artifacts must never be listed: {files:?}"
    );

    std::fs::remove_dir_all(&sandbox).ok();
}

#[test]
fn test_read_write_round_trip() {
    let sandbox = unique_sandbox("roundtrip");
    let rel = Path::new("src/lib.rs");

    // Parent dir does not exist yet — write must create it inside the sandbox.
    write_sandbox_file(&sandbox, rel, "pub fn x() {}\n").unwrap();
    assert!(sandbox.join("src").join("lib.rs").exists());
    assert_eq!(read_sandbox_file(&sandbox, rel).unwrap(), "pub fn x() {}\n");

    std::fs::remove_dir_all(&sandbox).ok();
}

#[test]
fn test_write_rejects_parent_escape() {
    let sandbox = unique_sandbox("escape");
    let outside = sandbox.parent().unwrap().join("evil.rs");
    std::fs::remove_file(&outside).ok();

    let err = write_sandbox_file(&sandbox, Path::new("../evil.rs"), "pwn");
    assert!(err.is_err(), "a `../` escape must be refused");
    assert!(
        !outside.exists(),
        "no file may be written outside the sandbox"
    );

    std::fs::remove_dir_all(&sandbox).ok();
}

#[test]
fn test_read_rejects_absolute_path() {
    let sandbox = unique_sandbox("abs");
    let abs = if cfg!(windows) {
        Path::new(r"C:\Windows\System32\drivers\etc\hosts")
    } else {
        Path::new("/etc/hosts")
    };
    assert!(
        read_sandbox_file(&sandbox, abs).is_err(),
        "an absolute path must be refused regardless of whether it exists"
    );

    std::fs::remove_dir_all(&sandbox).ok();
}

#[test]
fn test_write_rejects_target() {
    // `target/` is *inside* the sandbox, so the containment guard alone allows it;
    // the explicit build-artifact denial (C-001) is what refuses this.
    let sandbox = unique_sandbox("target");
    let err = write_sandbox_file(&sandbox, Path::new("target/x.rs"), "nope");
    assert!(err.is_err(), "writes into target/ must be refused");
    assert!(
        !sandbox.join("target").join("x.rs").exists(),
        "no file may be written under target/"
    );

    std::fs::remove_dir_all(&sandbox).ok();
}
