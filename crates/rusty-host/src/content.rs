//! Lesson content on disk: reading `lesson.toml` and copying a lesson's `starter/`
//! cargo project into its sandbox. The OS/filesystem half of the curriculum
//! (`rusty-curriculum` stays pure; it only parses strings).

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use rusty_curriculum::{parse_lesson, Lesson};

/// Read and parse `<content_dir>/lesson.toml` into a [`Lesson`].
pub fn load_lesson(content_dir: &Path) -> Result<Lesson> {
    let toml_path = content_dir.join("lesson.toml");
    let src = std::fs::read_to_string(&toml_path)
        .with_context(|| format!("reading {}", toml_path.display()))?;
    let lesson = parse_lesson(&src).with_context(|| format!("parsing {}", toml_path.display()))?;
    Ok(lesson)
}

/// Ensure the learner sandbox for `id` exists under `workspace_root/lessons/<id>/`,
/// copying `<content_dir>/starter/` into it the first time. Idempotent: if the
/// sandbox already exists it is returned untouched (so learner edits survive).
pub fn prepare_sandbox(content_dir: &Path, workspace_root: &Path, id: &str) -> Result<PathBuf> {
    let sandbox = workspace_root.join("lessons").join(id);
    if sandbox.exists() {
        return Ok(sandbox);
    }
    let starter = content_dir.join("starter");
    copy_dir_recursive(&starter, &sandbox)
        .with_context(|| format!("copying {} into {}", starter.display(), sandbox.display()))?;
    Ok(sandbox)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    // Read the source first, so a missing `starter/` errors *before* we create an
    // empty destination (which would otherwise satisfy the idempotency check and
    // leave a corrupt, empty sandbox).
    let entries = std::fs::read_dir(src).with_context(|| format!("reading {}", src.display()))?;
    std::fs::create_dir_all(dst)?;
    for entry in entries {
        let entry = entry?;
        let name = entry.file_name();
        // Never copy build artifacts into the learner sandbox.
        if name.to_str() == Some("target") {
            continue;
        }
        let path = entry.path();
        let target = dst.join(&name);
        if path.is_dir() {
            copy_dir_recursive(&path, &target)?;
        } else {
            std::fs::copy(&path, &target)?;
        }
    }
    Ok(())
}
