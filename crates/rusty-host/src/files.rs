//! Sandboxed file access for the in-app code editor: list a lesson sandbox's `.rs`
//! source files and read/write them. Every path is routed through the sandbox-escape
//! guard ([`crate::sandbox::contain`]) plus an explicit `target/` denial, so a crafted
//! relative path can neither climb out of the sandbox nor scribble in the build-
//! artifact directory. All editor `std::fs` lives here (the OS boundary, §11).

use std::path::{Component, Path, PathBuf};

use anyhow::{anyhow, Context, Result};

use crate::sandbox::contain;

/// List the relative paths of every `.rs` file under `sandbox`, skipping `target/`.
/// Paths are returned sorted, relative to `sandbox`.
pub fn list_sandbox_rs_files(sandbox: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    collect_rs(sandbox, sandbox, &mut out)
        .with_context(|| format!("listing .rs files under {}", sandbox.display()))?;
    out.sort();
    Ok(out)
}

fn collect_rs(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        // Never surface build artifacts to the editor.
        if entry.file_name().to_str() == Some("target") {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            collect_rs(root, &path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            if let Ok(rel) = path.strip_prefix(root) {
                out.push(rel.to_path_buf());
            }
        }
    }
    Ok(())
}

/// Read an in-sandbox file by its `sandbox`-relative path.
pub fn read_sandbox_file(sandbox: &Path, rel: &Path) -> Result<String> {
    let path = safe_path(sandbox, rel)?;
    std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))
}

/// Write an in-sandbox file by its `sandbox`-relative path, creating parent
/// directories (inside the sandbox) as needed.
pub fn write_sandbox_file(sandbox: &Path, rel: &Path, contents: &str) -> Result<()> {
    let path = safe_path(sandbox, rel)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    std::fs::write(&path, contents).with_context(|| format!("writing {}", path.display()))
}

/// Resolve `rel` against `sandbox`, refusing sandbox escapes and the build-artifact
/// `target/` directory. The `target/` denial is a *separate* explicit check: `target/`
/// lives inside the sandbox, so [`contain`] alone would (correctly) allow it.
fn safe_path(sandbox: &Path, rel: &Path) -> Result<PathBuf> {
    if enters_target(rel) {
        return Err(anyhow!(
            "refusing to touch the build-artifact directory `target/`: {}",
            rel.display()
        ));
    }
    contain(sandbox, rel).ok_or_else(|| {
        anyhow!(
            "path {} escapes the lesson sandbox {}",
            rel.display(),
            sandbox.display()
        )
    })
}

/// True if the first real segment of `rel` is the build-artifact dir `target`.
/// Anchor/parent components are left for [`contain`] to reject.
fn enters_target(rel: &Path) -> bool {
    for comp in rel.components() {
        match comp {
            Component::CurDir => continue,
            Component::Normal(s) => return s.to_str() == Some("target"),
            _ => return false,
        }
    }
    false
}
