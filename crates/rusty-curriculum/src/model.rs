//! The typed curriculum model (prompt §3). Pure data — no filesystem, no process.
//!
//! Lessons are authored as `lesson.toml` and deserialized into [`Lesson`]. Block and
//! recall variants use internally-tagged enums (`kind = "..."`), which `toml`
//! deserializes from `[[body]]` / `[recall_prompt]` tables.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Stable lesson identifier, e.g. `"foundations-01-hello"`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LessonId(pub String);

/// Stable concept identifier (tracked individually for spaced repetition).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    pub body: Vec<Block>,
    pub recall_prompt: RecallPrompt,
    #[serde(default)]
    pub exercises: Vec<Exercise>,
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
}
