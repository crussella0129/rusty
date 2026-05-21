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
    if lesson.body.is_empty() {
        return Err(CurriculumError::Invalid("lesson body is empty".to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: &str = r#"
        id = "foundations-01-hello"
        title = "Hello, compiler."
        track = "Foundations"
        estimated_minutes = 12
        starter_project = "starter"
        solution_project = "solution"

        [[body]]
        kind = "prose"
        text = "Welcome."

        [[body]]
        kind = "now_run"
        command = "cargo run"

        [recall_prompt]
        kind = "short_answer"
        question = "What runs your code?"
        expected = "cargo run"
        explanation = "cargo run builds then executes."
    "#;

    #[test]
    fn test_parse_valid_lesson() {
        let lesson = parse_lesson(VALID).expect("valid lesson parses");
        assert_eq!(lesson.id.0, "foundations-01-hello");
        assert_eq!(lesson.title, "Hello, compiler.");
        assert_eq!(lesson.body.len(), 2);
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
}
