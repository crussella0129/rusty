//! Sandbox helpers: which shell to launch, and `cd`-line interception.
//!
//! Rusty locks the embedded terminal's working directory inside a lesson's sandbox
//! root. This module provides the *pure* decision logic (no filesystem access, no
//! subprocess) so it is fully unit-testable: [`resolve_cd`] inspects a submitted
//! input line and decides whether a `cd` would escape the sandbox.
//!
//! This is sandbox-as-good-UX, not sandbox-as-security (prompt §2): the goal is to
//! keep accidental wandering inside the safe zone, not to imprison a determined user.

use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};

/// What a submitted input line means for the sandbox `cwd`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CdOutcome {
    /// The line is a `cd` whose resolved target stays inside the sandbox root.
    Allowed(PathBuf),
    /// The line is a `cd` whose target would escape the sandbox root — refuse it.
    Refused,
    /// The line is not a `cd` command — forward it to the shell unchanged.
    NotCd,
}

/// The platform's default interactive shell program name.
///
/// Windows: prefer `pwsh` (PowerShell 7+) when on PATH, else `cmd`. Unix: `$SHELL`
/// if set, else `/bin/bash`. We return only the program string; the caller builds
/// the full command and sets the sandbox `cwd`.
pub fn default_shell() -> String {
    #[cfg(windows)]
    {
        // `pwsh` if discoverable on PATH, otherwise the always-present `cmd`.
        if which_on_path("pwsh.exe") {
            "pwsh.exe".to_string()
        } else {
            "cmd.exe".to_string()
        }
    }
    #[cfg(not(windows))]
    {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
    }
}

#[cfg(windows)]
fn which_on_path(program: &str) -> bool {
    let Some(paths) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&paths).any(|dir| dir.join(program).is_file())
}

/// Decide whether a submitted input `line` is a sandbox-escaping `cd`.
///
/// `cwd` is the terminal's current directory; `root` is the sandbox root. Both are
/// treated lexically — neither needs to exist on disk yet. A bare `cd` (no argument)
/// is treated as "go home", i.e. the sandbox root.
pub fn resolve_cd(line: &str, cwd: &Path, root: &Path) -> CdOutcome {
    let mut tokens = line.split_whitespace();
    match tokens.next() {
        Some("cd") => {}
        _ => return CdOutcome::NotCd,
    }

    let candidate = match tokens.next() {
        None => root.to_path_buf(), // bare `cd` → home → sandbox root
        Some(target) => {
            let tp = Path::new(target);
            if tp.is_absolute() {
                tp.to_path_buf()
            } else {
                cwd.join(tp)
            }
        }
    };

    match (normalize(&candidate), normalize(root)) {
        (Some(cand), Some(root_norm)) if starts_with(&cand, &root_norm) => {
            CdOutcome::Allowed(rebuild(&cand))
        }
        _ => CdOutcome::Refused,
    }
}

/// Resolve a learner-supplied *relative* path against the sandbox `root`, returning a
/// real absolute path **iff** it stays inside the sandbox. Lexical/pure (no disk
/// access). Used by the file-I/O layer ([`crate::files`]) to refuse reads/writes that
/// would escape via `..` or an absolute path. The returned path is rebuilt onto the
/// real `root` (not from segments) so it is a valid argument to `std::fs`.
pub fn contain(root: &Path, rel: &Path) -> Option<PathBuf> {
    if rel.is_absolute() {
        return None;
    }
    let candidate = root.join(rel);
    let cand = normalize(&candidate)?;
    let root_norm = normalize(root)?;
    if !starts_with(&cand, &root_norm) {
        return None;
    }
    // Append only the in-sandbox tail segments onto the real root path.
    let mut out = root.to_path_buf();
    for (_, seg) in &cand[root_norm.len()..] {
        out.push(seg);
    }
    Some(out)
}

/// A normalized path as a flat list of segments. Anchor segments (drive prefix, root
/// separator) are `(false, ..)`; named directory segments are `(true, ..)`. Returns
/// `None` if a `..` walks above the anchor — that always escapes the sandbox.
type Segments = Vec<(bool, OsString)>;

fn normalize(p: &Path) -> Option<Segments> {
    let mut out: Segments = Vec::new();
    for comp in p.components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => match out.last() {
                Some((true, _)) => {
                    out.pop();
                }
                _ => return None, // `..` above the anchor → escaped
            },
            Component::Normal(s) => out.push((true, s.to_os_string())),
            Component::Prefix(pre) => out.push((false, pre.as_os_str().to_os_string())),
            Component::RootDir => out.push((false, OsString::from(std::path::MAIN_SEPARATOR_STR))),
        }
    }
    Some(out)
}

/// True iff `cand` is `base` or a descendant of it (segment-prefix comparison).
fn starts_with(cand: &Segments, base: &Segments) -> bool {
    cand.len() >= base.len() && cand[..base.len()] == base[..]
}

/// Rebuild a `PathBuf` from normalized segments (used only for the `Allowed` payload).
fn rebuild(segs: &Segments) -> PathBuf {
    let mut pb = PathBuf::new();
    for (_, s) in segs {
        pb.push(s);
    }
    pb
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> PathBuf {
        // An absolute, OS-appropriate synthetic sandbox root (need not exist on disk).
        if cfg!(windows) {
            PathBuf::from(r"C:\sandbox\lessons\spike")
        } else {
            PathBuf::from("/sandbox/lessons/spike")
        }
    }

    #[test]
    fn test_resolve_cd_into_subdir() {
        let r = root();
        match resolve_cd("cd sub", &r, &r) {
            CdOutcome::Allowed(p) => {
                assert!(starts_with(
                    &normalize(&p).unwrap(),
                    &normalize(&r).unwrap()
                ));
                assert!(p.ends_with("sub"));
            }
            other => panic!("expected Allowed, got {other:?}"),
        }
    }

    #[test]
    fn test_resolve_cd_dotdot_refused() {
        let r = root();
        assert_eq!(resolve_cd("cd ..", &r, &r), CdOutcome::Refused);
    }

    #[test]
    fn test_resolve_cd_root_refused() {
        let r = root();
        let target = if cfg!(windows) { r"cd C:\" } else { "cd /" };
        assert_eq!(resolve_cd(target, &r, &r), CdOutcome::Refused);
    }

    #[test]
    fn test_resolve_cd_absolute_outside_refused() {
        let r = root();
        let target = if cfg!(windows) {
            r"cd C:\Windows\System32"
        } else {
            "cd /etc"
        };
        assert_eq!(resolve_cd(target, &r, &r), CdOutcome::Refused);
    }

    #[test]
    fn test_resolve_cd_bare_is_root() {
        let r = root();
        match resolve_cd("cd", &r, &r) {
            CdOutcome::Allowed(p) => {
                assert_eq!(normalize(&p).unwrap(), normalize(&r).unwrap());
            }
            other => panic!("expected Allowed(root), got {other:?}"),
        }
    }

    #[test]
    fn test_resolve_cd_nested_then_back_stays_inside() {
        // `cd a/../b` resolves to root/b — still inside.
        let r = root();
        match resolve_cd("cd a/../b", &r, &r) {
            CdOutcome::Allowed(p) => assert!(p.ends_with("b")),
            other => panic!("expected Allowed, got {other:?}"),
        }
    }

    #[test]
    fn test_resolve_cd_deep_escape_refused() {
        // `cd ../../..` clearly climbs out.
        let r = root();
        assert_eq!(resolve_cd("cd ../../../..", &r, &r), CdOutcome::Refused);
    }

    #[test]
    fn test_resolve_cd_noncd_passthrough() {
        let r = root();
        assert_eq!(resolve_cd("cargo run", &r, &r), CdOutcome::NotCd);
        assert_eq!(resolve_cd("ls -la", &r, &r), CdOutcome::NotCd);
        assert_eq!(resolve_cd("", &r, &r), CdOutcome::NotCd);
    }

    #[test]
    fn test_default_shell_nonempty() {
        assert!(!default_shell().is_empty());
    }
}
