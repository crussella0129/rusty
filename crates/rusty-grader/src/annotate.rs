//! A pure, egui-free rendering model for a grading [`Verdict`] (prompt §5). The app's
//! annotation pane draws this; keeping the model here — not in `rusty-app` — means the
//! UI depends only on plain owned types, never on `cargo_metadata`. The verbatim
//! rustc-`rendered` text is preserved as the teaching surface (§1.6), and known error
//! codes become [`ConceptLink`]s via [`concept_for_code`].

use crate::diagnostic::Diag;
use crate::error_map::concept_for_code;
use crate::verdict::Verdict;

/// The headline category of a graded result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Headline {
    Pass,
    CompileError,
    TestsFailed,
    RunMismatch,
}

/// A link from a compiler error code to the lesson that teaches the relevant concept.
/// Whether the link is *navigable* (the lesson exists) is decided by the UI, not here —
/// this model is forward-looking, like [`concept_for_code`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConceptLink {
    pub code: String,
    pub lesson_id: String,
}

/// A fully-rendered, egui-free view of a grading [`Verdict`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Annotation {
    pub headline: Headline,
    /// Verbatim text blocks to show monospaced: each error's rustc `rendered` text,
    /// or (for a run mismatch) the expected/actual outputs.
    pub body_blocks: Vec<String>,
    /// Concept links derived from the error codes present (deduplicated, in order).
    pub links: Vec<ConceptLink>,
}

/// Turn a grading [`Verdict`] into a renderable [`Annotation`].
pub fn annotate(verdict: &Verdict) -> Annotation {
    match verdict {
        Verdict::Pass => Annotation {
            headline: Headline::Pass,
            body_blocks: Vec::new(),
            links: Vec::new(),
        },
        Verdict::CompileError(diags) => Annotation {
            headline: Headline::CompileError,
            body_blocks: diags.iter().map(diag_text).collect(),
            links: links_for(diags),
        },
        Verdict::TestsFailed => Annotation {
            headline: Headline::TestsFailed,
            body_blocks: Vec::new(),
            links: Vec::new(),
        },
        Verdict::RunMismatch { expected, got } => Annotation {
            headline: Headline::RunMismatch,
            body_blocks: vec![format!("expected:\n{expected}"), format!("got:\n{got}")],
            links: Vec::new(),
        },
    }
}

/// A diagnostic's verbatim rustc text, falling back to its short message.
fn diag_text(d: &Diag) -> String {
    d.rendered.clone().unwrap_or_else(|| d.message.clone())
}

/// Deduplicated concept links for every diag carrying a *known* error code.
fn links_for(diags: &[Diag]) -> Vec<ConceptLink> {
    let mut links: Vec<ConceptLink> = Vec::new();
    for d in diags {
        if let Some(code) = &d.code {
            if let Some(lesson_id) = concept_for_code(code) {
                let link = ConceptLink {
                    code: code.clone(),
                    lesson_id: lesson_id.to_string(),
                };
                if !links.contains(&link) {
                    links.push(link);
                }
            }
        }
    }
    links
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostic::Level;

    fn diag(code: Option<&str>, rendered: &str) -> Diag {
        Diag {
            code: code.map(str::to_string),
            level: Level::Error,
            message: "msg".to_string(),
            rendered: Some(rendered.to_string()),
            primary_span: None,
        }
    }

    #[test]
    fn test_annotate_pass() {
        let a = annotate(&Verdict::Pass);
        assert_eq!(a.headline, Headline::Pass);
        assert!(a.links.is_empty());
        assert!(a.body_blocks.is_empty());
    }

    #[test]
    fn test_annotate_compile_error_links_concept() {
        let v = Verdict::CompileError(vec![diag(
            Some("E0382"),
            "error[E0382]: borrow of moved value",
        )]);
        let a = annotate(&v);
        assert_eq!(a.headline, Headline::CompileError);
        assert!(a.body_blocks[0].contains("error[E0382]"));
        assert_eq!(
            a.links,
            vec![ConceptLink {
                code: "E0382".to_string(),
                lesson_id: "foundations-03-ownership-moves".to_string(),
            }]
        );
    }

    #[test]
    fn test_annotate_unresolved_name_links_lesson1() {
        // C-002: an E0425/E0433 (unresolved name) maps to the *authored* lesson 1, so
        // the live-link path is reachable from a Faded starter with an unfilled TODO.
        let v = Verdict::CompileError(vec![diag(
            Some("E0425"),
            "error[E0425]: cannot find function `greeting`",
        )]);
        let a = annotate(&v);
        assert_eq!(a.links[0].lesson_id, "foundations-01-hello");
    }

    #[test]
    fn test_annotate_compile_error_unknown_code_no_link() {
        let v = Verdict::CompileError(vec![diag(Some("E9999"), "error[E9999]: imaginary")]);
        let a = annotate(&v);
        assert!(a.links.is_empty(), "unknown code yields no link");
        assert!(
            !a.body_blocks.is_empty(),
            "but the body text is still shown"
        );
    }

    #[test]
    fn test_annotate_dedupes_links() {
        let v = Verdict::CompileError(vec![diag(Some("E0382"), "a"), diag(Some("E0382"), "b")]);
        let a = annotate(&v);
        assert_eq!(a.links.len(), 1, "the same code links once");
        assert_eq!(a.body_blocks.len(), 2, "but every diag's text is kept");
    }

    #[test]
    fn test_annotate_tests_failed() {
        assert_eq!(
            annotate(&Verdict::TestsFailed).headline,
            Headline::TestsFailed
        );
    }

    #[test]
    fn test_annotate_run_mismatch() {
        let v = Verdict::RunMismatch {
            expected: "hello".to_string(),
            got: "nope".to_string(),
        };
        let a = annotate(&v);
        assert_eq!(a.headline, Headline::RunMismatch);
        let joined = a.body_blocks.join("\n");
        assert!(joined.contains("hello") && joined.contains("nope"));
    }
}
