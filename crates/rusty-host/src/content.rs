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

/// Whether a sandbox dir is *structurally* healthy — i.e. `prepare_sandbox`'s
/// idempotency may safely return early (preserving learner edits). The Sprint-6 cascade
/// taught us that a bare `sandbox.exists()` check is not enough: a half-corrupt sandbox
/// (e.g. just an empty dir with stray subdirs) passes `exists()` and short-circuits the
/// copy, leaving cargo to escalate to the parent workspace.
///
/// The three-part marker (s7 ADR):
///  1. `Cargo.toml` file present (cargo finds a local manifest).
///  2. `src/main.rs` file present (the package's main bin is there).
///  3. The `Cargo.toml` parses as TOML and carries a top-level `workspace` key — the
///     `[workspace]` table is the s2 detach mechanism that keeps `cargo` from
///     escalating to the outer Rusty workspace.
pub fn is_sandbox_healthy(sandbox: &Path) -> bool {
    let cargo_toml = sandbox.join("Cargo.toml");
    if !cargo_toml.is_file() || !sandbox.join("src").join("main.rs").is_file() {
        return false;
    }
    let Ok(src) = std::fs::read_to_string(&cargo_toml) else {
        return false;
    };
    // NB: `str::parse::<toml::Value>()` in toml 1.x parses a *single* TOML value, so
    // `[workspace]` is read as an inline-array header and the trailing tables trip an
    // "expected nothing" error. `toml::Table` (FromStr) parses a whole document.
    let Ok(parsed) = src.parse::<toml::Table>() else {
        return false;
    };
    parsed.contains_key("workspace")
}

/// Ensure the learner sandbox for `id` exists under `workspace_root/lessons/<id>/`,
/// copying `<content_dir>/starter/` into it. **Marker-file idempotent (s7):** if the
/// sandbox dir already exists AND [`is_sandbox_healthy`] is true, returns the existing
/// dir untouched (preserves learner edits, the s2 promise). Otherwise — whether the dir
/// is missing, empty, or corrupt — wipes the dir contents and re-copies `starter/` so
/// the lesson always has its `Cargo.toml` (with the `[workspace]` detach table) and
/// `src/main.rs` in place. The heavy hand is justified by the user being unable to
/// work in a broken sandbox.
pub fn prepare_sandbox(content_dir: &Path, workspace_root: &Path, id: &str) -> Result<PathBuf> {
    let sandbox = workspace_root.join("lessons").join(id);
    if sandbox.exists() && is_sandbox_healthy(&sandbox) {
        return Ok(sandbox);
    }
    if sandbox.exists() {
        std::fs::remove_dir_all(&sandbox)
            .with_context(|| format!("wiping corrupt sandbox {}", sandbox.display()))?;
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

/// Read and parse `<content_dir>/manifest.toml` into a `Manifest`.
pub fn load_manifest(content_dir: &Path) -> Result<rusty_curriculum::Manifest> {
    let toml_path = content_dir.join("manifest.toml");
    let src = std::fs::read_to_string(&toml_path)
        .with_context(|| format!("reading {}", toml_path.display()))?;
    let manifest = rusty_curriculum::parse_manifest(&src)
        .with_context(|| format!("parsing {}", toml_path.display()))?;
    Ok(manifest)
}

