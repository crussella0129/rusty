//! The typed curriculum model (prompt §3). Pure data — no filesystem, no process.
//!
//! Lessons are authored as `lesson.toml` and deserialized into [`Lesson`]. Block and
//! recall variants use internally-tagged enums (`kind = "..."`), which `toml`
//! deserializes from `[[body]]` / `[recall_prompt]` tables.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Stable lesson identifier, e.g. `"foundations-01-hello"`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LessonId(pub String);

/// Stable concept identifier (tracked individually for spaced repetition).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ConceptId(pub String);

/// Which track a lesson belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Track {
    Foundations,
    Intermediate,
    Async,
    Macros,
}

/// One atomic concept a lesson teaches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Concept {
    pub id: ConceptId,
    pub claim: String,
    pub why_it_matters: String,
    #[serde(default)]
    pub common_misconception: Option<String>,
}

/// Tone of a callout block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CalloutTone {
    #[default]
    Note,
    Tip,
    Warning,
}

/// A unit of lesson body content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Block {
    /// Markdown prose.
    Prose { text: String },
    /// A read-only code sample.
    Code {
        #[serde(default)]
        lang: String,
        source: String,
    },
    /// A "now run this in the terminal" prompt.
    NowRun {
        command: String,
        #[serde(default)]
        note: Option<String>,
    },
    /// A boxed aside (note/tip/warning).
    Callout {
        #[serde(default)]
        tone: CalloutTone,
        text: String,
    },
}

/// The mandatory recall prompt shown before exercises.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RecallPrompt {
    MultipleChoice {
        question: String,
        choices: Vec<String>,
        answer_index: usize,
        explanation: String,
    },
    ShortAnswer {
        question: String,
        expected: String,
        explanation: String,
    },
}

/// How an exercise is graded (rendered/evaluated in Phase 3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SuccessCriterion {
    CargoTestPasses,
    CargoRunOutputMatches { expected: String },
}

/// An exercise (variants rendered/graded in Phase 3; defined now for the schema).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Exercise {
    Worked {
        prompt: String,
        code: String,
        annotation: String,
    },
    Faded {
        prompt: String,
        file_path: PathBuf,
        check_command: String,
        success_criterion: SuccessCriterion,
    },
    Open {
        prompt: String,
        check_command: String,
        success_criterion: SuccessCriterion,
    },
    PredictThenRun {
        code: String,
        question: String,
        expected_output: String,
        explanation: String,
    },
}

/// One step of a lesson: a chunk of content plus an optional exercise. Steps render in
/// order; a *gating* step (one whose exercise is `Faded`/`Open`) hides everything after
/// it until its code grades `Pass` — this is how lesson content "materializes" as the
/// learner progresses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Step {
    /// The prose/code/now-run/callout shown for this step (may be empty).
    #[serde(default)]
    pub blocks: Vec<Block>,
    /// An optional exercise belonging to this step.
    #[serde(default)]
    pub exercise: Option<Exercise>,
    /// A tip shown after the learner's first failed Check on this step's gating exercise.
    #[serde(default)]
    pub hint: Option<String>,
}

impl Step {
    /// A step *gates* later content iff it carries a gradeable (Faded/Open) exercise that
    /// must pass. Worked/PredictThenRun and exercise-less steps never block.
    pub fn is_gating(&self) -> bool {
        matches!(
            self.exercise,
            Some(Exercise::Faded { .. }) | Some(Exercise::Open { .. })
        )
    }
}

/// How many leading steps are currently visible: the prefix up to *and including* the
/// first gating step that is not yet completed. When no step gates (or all gates are
/// complete), every step is visible. `completed[i]` is whether step `i`'s gate has been
/// satisfied (a `Pass`); indices past `completed`'s length are treated as incomplete.
pub fn visible_prefix(steps: &[Step], completed: &[bool]) -> usize {
    for (i, step) in steps.iter().enumerate() {
        let done = completed.get(i).copied().unwrap_or(false);
        if step.is_gating() && !done {
            return i + 1; // this gating step is visible; nothing past it is
        }
    }
    steps.len()
}

/// A further-reading pointer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reference {
    pub title: String,
    pub url: String,
    #[serde(default)]
    pub note: Option<String>,
}

/// A complete lesson, deserialized from `lesson.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lesson {
    pub id: LessonId,
    pub title: String,
    pub track: Track,
    #[serde(default)]
    pub prereqs: Vec<LessonId>,
    pub estimated_minutes: u8,
    #[serde(default)]
    pub concepts: Vec<Concept>,
    /// The lesson body as an ordered list of steps (prose/exercises), gated for
    /// progressive disclosure (see [`Step`] / [`visible_prefix`]). Defaults to empty so
    /// the loader can reject "no steps" with a clear message rather than a serde error.
    #[serde(default)]
    pub steps: Vec<Step>,
    pub recall_prompt: RecallPrompt,
    pub starter_project: PathBuf,
    pub solution_project: PathBuf,
    #[serde(default)]
    pub further_reading: Vec<Reference>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The C-001 spike: confirm `toml` deserializes an internally-tagged enum from a
    /// table. If this ever fails, switch the tagged enums to adjacently-tagged.
    #[test]
    fn test_block_now_run_deser() {
        let src = r#"
            kind = "now_run"
            command = "cargo run"
            note = "watch it compile"
        "#;
        let block: Block = toml::from_str(src).unwrap();
        assert_eq!(
            block,
            Block::NowRun {
                command: "cargo run".to_string(),
                note: Some("watch it compile".to_string()),
            }
        );
    }

    #[test]
    fn test_block_and_recall_variants_deser() {
        #[derive(Deserialize)]
        struct Doc {
            body: Vec<Block>,
            recall_prompt: RecallPrompt,
        }
        let src = r#"
            [[body]]
            kind = "prose"
            text = "Hello, compiler."

            [[body]]
            kind = "code"
            lang = "rust"
            source = "fn main() {}"

            [recall_prompt]
            kind = "multiple_choice"
            question = "What does `cargo run` do?"
            choices = ["compiles only", "compiles then runs", "formats"]
            answer_index = 1
            explanation = "It builds the binary, then executes it."
        "#;
        let doc: Doc = toml::from_str(src).unwrap();
        assert_eq!(doc.body.len(), 2);
        assert!(matches!(doc.body[0], Block::Prose { .. }));
        assert!(matches!(doc.body[1], Block::Code { .. }));
        assert!(matches!(
            doc.recall_prompt,
            RecallPrompt::MultipleChoice { .. }
        ));
    }

    fn step(exercise: Option<Exercise>) -> Step {
        Step {
            blocks: vec![],
            exercise,
            hint: None,
        }
    }

    fn faded() -> Exercise {
        Exercise::Faded {
            prompt: "p".into(),
            file_path: "src/main.rs".into(),
            check_command: "cargo test".into(),
            success_criterion: SuccessCriterion::CargoTestPasses,
        }
    }

    fn open() -> Exercise {
        Exercise::Open {
            prompt: "p".into(),
            check_command: "cargo run".into(),
            success_criterion: SuccessCriterion::CargoRunOutputMatches {
                expected: "x".into(),
            },
        }
    }

    fn worked() -> Exercise {
        Exercise::Worked {
            prompt: "p".into(),
            code: "c".into(),
            annotation: "a".into(),
        }
    }

    #[test]
    fn test_step_is_gating() {
        assert!(step(Some(faded())).is_gating());
        assert!(step(Some(open())).is_gating());
        assert!(!step(Some(worked())).is_gating());
        assert!(!step(None).is_gating(), "a prose-only step never gates");
    }

    #[test]
    fn test_visible_prefix_no_gates() {
        let steps = vec![step(None), step(Some(worked())), step(None)];
        assert_eq!(visible_prefix(&steps, &[false, false, false]), 3);
    }

    #[test]
    fn test_visible_prefix_stops_at_incomplete_gate() {
        // [prose, faded, open] — all incomplete → through the Faded, hiding the Open.
        let steps = vec![step(None), step(Some(faded())), step(Some(open()))];
        assert_eq!(visible_prefix(&steps, &[false, false, false]), 2);
    }

    #[test]
    fn test_visible_prefix_advances_when_gate_completed() {
        let steps = vec![step(None), step(Some(faded())), step(Some(open()))];
        // Faded done → Open now revealed (prefix runs to the next incomplete gate).
        assert_eq!(visible_prefix(&steps, &[false, true, false]), 3);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub tracks: Vec<String>,
    pub lessons: Vec<String>,
}
