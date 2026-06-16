//! Pure lesson parsing. No filesystem here — `rusty-host` reads the file and calls
//! [`parse_lesson`] with the contents, keeping this crate OS-portable.

use thiserror::Error;

use crate::model::Lesson;

/// Errors from parsing or validating a lesson.
#[derive(Debug, Error)]
pub enum CurriculumError {
    /// The TOML did not parse into the lesson model.
    #[error("failed to parse lesson TOML: {0}")]
    Parse(#[from] toml::de::Error),
    /// The TOML parsed but the lesson is semantically invalid.
    #[error("invalid lesson: {0}")]
    Invalid(String),
}

/// Parse a `lesson.toml` string into a validated [`Lesson`].
pub fn parse_lesson(src: &str) -> Result<Lesson, CurriculumError> {
    let lesson: Lesson = toml::from_str(src)?;
    validate(&lesson)?;
    Ok(lesson)
}

fn validate(lesson: &Lesson) -> Result<(), CurriculumError> {
    if lesson.id.0.trim().is_empty() {
        return Err(CurriculumError::Invalid("lesson id is empty".to_string()));
    }
    if lesson.title.trim().is_empty() {
        return Err(CurriculumError::Invalid(
            "lesson title is empty".to_string(),
        ));
    }
    if lesson.steps.is_empty() {
        return Err(CurriculumError::Invalid("lesson has no steps".to_string()));
    }
    Ok(())
}
/// Parse a complete curriculum manifest from a TOML string.
pub fn parse_manifest(src: &str) -> Result<crate::model::Manifest, CurriculumError> {
    toml::from_str(src).map_err(CurriculumError::Parse)
}

#[cfg(test)]
mod tests {
    use super::*;

    // A two-step lesson: step 1 has TWO blocks (prose + code); step 2 carries a Faded
    // exercise with an INLINE success_criterion + a hint. Exercises the doubly-nested
    // `[[steps.blocks]]` array-of-tables and the inline-table criterion (critique C-005).
    const VALID: &str = r#"
        id = "foundations-01-hello"
        title = "Hello, compiler."
        track = "Foundations"
        estimated_minutes = 12
        starter_project = "starter"
        solution_project = "solution"

        [[steps]]

        [[steps.blocks]]
        kind = "prose"
        text = "Welcome."

        [[steps.blocks]]
        kind = "code"
        lang = "rust"
        source = "fn main() {}"

        [[steps]]
        hint = "Define `greeting` first."

        [[steps.blocks]]
        kind = "prose"
        text = "Now make the test pass."

        [steps.exercise]
        kind = "faded"
        prompt = "Fill it in."
        file_path = "src/main.rs"
        check_command = "cargo test"
        success_criterion = { kind = "cargo_test_passes" }

        [recall_prompt]
        kind = "short_answer"
        question = "What runs your code?"
        expected = "cargo run"
        explanation = "cargo run builds then executes."
    "#;

    #[test]
    fn test_parse_lesson_steps() {
        let lesson = parse_lesson(VALID).expect("valid stepped lesson parses");
        assert_eq!(lesson.id.0, "foundations-01-hello");
        assert_eq!(lesson.steps.len(), 2);
        // Step 1: two blocks, order + contents preserved.
        assert_eq!(lesson.steps[0].blocks.len(), 2);
        assert!(matches!(
            lesson.steps[0].blocks[0],
            crate::Block::Prose { .. }
        ));
        assert!(matches!(
            lesson.steps[0].blocks[1],
            crate::Block::Code { .. }
        ));
        assert!(lesson.steps[0].exercise.is_none());
        // Step 2: a Faded exercise (inline criterion) + a hint.
        assert!(matches!(
            lesson.steps[1].exercise,
            Some(crate::Exercise::Faded { .. })
        ));
        assert_eq!(
            lesson.steps[1].hint.as_deref(),
            Some("Define `greeting` first.")
        );
        assert!(lesson.steps[1].is_gating());
    }

    #[test]
    fn test_parse_invalid_toml() {
        let err = parse_lesson("this is = = not valid").unwrap_err();
        assert!(matches!(err, CurriculumError::Parse(_)));
    }

    #[test]
    fn test_parse_empty_id_rejected() {
        let src = VALID.replace(r#"id = "foundations-01-hello""#, r#"id = """#);
        let err = parse_lesson(&src).unwrap_err();
        assert!(matches!(err, CurriculumError::Invalid(_)));
    }

    #[test]
    fn test_parse_zero_steps_errors() {
        // Strip the `[[steps]]` tables → a lesson with no steps must be rejected.
        let src = r#"
            id = "x"
            title = "X"
            track = "Foundations"
            estimated_minutes = 1
            starter_project = "s"
            solution_project = "sol"

            [recall_prompt]
            kind = "short_answer"
            question = "q"
            expected = "a"
            explanation = "e"
        "#;
        let err = parse_lesson(src).unwrap_err();
        assert!(matches!(err, CurriculumError::Invalid(_)));
    }
}
