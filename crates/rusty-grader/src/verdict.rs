//! The grading verdict and the pure functions that produce it from captured cargo
//! output. No process spawning here — `rusty-host` runs cargo and passes the output in.

use crate::diagnostic::{parse_diagnostics, Diag, Level};

/// The outcome of grading one exercise submission.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// The check passed.
    Pass,
    /// The code did not compile — carries the error diagnostics (rustc `rendered`).
    CompileError(Vec<Diag>),
    /// Compiled, but `cargo test` reported failures (exit non-zero, no error diags).
    TestsFailed,
    /// `cargo run` output did not match the expected text.
    RunMismatch { expected: String, got: String },
}

/// Grade a `cargo test --message-format=json` run from its JSON + exit success.
pub fn grade_cargo_test(json: &str, exit_ok: bool) -> Verdict {
    verdict_from_diags(parse_diagnostics(json), exit_ok)
}

/// Pure verdict logic over already-parsed diagnostics (unit-testable without JSON).
pub fn verdict_from_diags(diags: Vec<Diag>, exit_ok: bool) -> Verdict {
    let errors: Vec<Diag> = diags
        .into_iter()
        .filter(|d| d.level == Level::Error)
        .collect();
    if !errors.is_empty() {
        Verdict::CompileError(errors)
    } else if exit_ok {
        Verdict::Pass
    } else {
        Verdict::TestsFailed
    }
}

/// Grade `cargo run` stdout against expected output (normalized — see [`normalize`]).
pub fn grade_run_output(stdout: &str, expected: &str) -> Verdict {
    if normalize(stdout) == normalize(expected) {
        Verdict::Pass
    } else {
        Verdict::RunMismatch {
            expected: expected.to_string(),
            got: stdout.to_string(),
        }
    }
}

/// Normalize program output for comparison: CRLF→LF, trim trailing whitespace on each
/// line, and drop trailing blank lines. Load-bearing on Windows (CRLF) and against
/// `lesson.toml`-authored `expected` strings.
fn normalize(s: &str) -> String {
    s.replace("\r\n", "\n")
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::Span;

    fn err_diag() -> Diag {
        Diag {
            code: Some("E0382".to_string()),
            level: Level::Error,
            message: "borrow of moved value".to_string(),
            rendered: Some("error[E0382]: ...".to_string()),
            primary_span: Some(Span {
                file_name: "src/main.rs".to_string(),
                line_start: 4,
                column_start: 20,
            }),
        }
    }

    #[test]
    fn test_grade_cargo_test_pass() {
        assert_eq!(verdict_from_diags(vec![], true), Verdict::Pass);
    }

    #[test]
    fn test_grade_cargo_test_pass_from_json() {
        // The named entry point over a real (empty/clean) JSON stream → Pass.
        assert_eq!(grade_cargo_test("", true), Verdict::Pass);
    }

    #[test]
    fn test_grade_cargo_test_compile_error() {
        let v = verdict_from_diags(vec![err_diag()], false);
        match v {
            Verdict::CompileError(diags) => {
                assert_eq!(diags[0].code.as_deref(), Some("E0382"))
            }
            other => panic!("expected CompileError, got {other:?}"),
        }
    }

    #[test]
    fn test_grade_cargo_test_tests_failed() {
        // Compiled (no error diags) but exit non-zero → a test failed.
        assert_eq!(verdict_from_diags(vec![], false), Verdict::TestsFailed);
    }

    #[test]
    fn test_grade_run_output_match() {
        assert_eq!(grade_run_output("hello\n", "hello"), Verdict::Pass);
    }

    #[test]
    fn test_grade_run_output_crlf() {
        // Windows cargo emits CRLF + a trailing newline; must still match a LF expected.
        assert_eq!(grade_run_output("hi\r\nbye\r\n", "hi\nbye"), Verdict::Pass);
    }

    #[test]
    fn test_grade_run_output_mismatch() {
        match grade_run_output("nope\n", "hello") {
            Verdict::RunMismatch { expected, .. } => assert_eq!(expected, "hello"),
            other => panic!("expected RunMismatch, got {other:?}"),
        }
    }
}
