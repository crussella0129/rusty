//! Rusty's voice.
//!
//! Every user-facing string and (in later phases) every mascot-state trigger lives
//! in this one file (prompt §12). When the tone changes, a translation is added, or
//! encouragement levels are A/B tested, it is a single-file edit. If you find
//! yourself writing UI copy inline in an egui widget, move it here instead.
//!
//! Voice rules: Rusty is referred to in the third person ("Looks like the borrow
//! checker isn't happy with line 4"), never the first person — first-person mascots
//! are uncanny. Rusty teaches Rust; Rusty is not Rust.

/// The application title, shown in the OS window title bar and the central panel.
pub const WINDOW_TITLE: &str = "Rusty — Learn Rust by Doing";

/// Heading of the lesson pane (shown as a fallback when no lesson is loaded).
pub const LESSON_PANE_TITLE: &str = "Lesson";

/// Label above the embedded terminal pane.
pub const TERMINAL_PANE_LABEL: &str = "Terminal (sandboxed)";

/// Label above the code-editor pane.
pub const EDITOR_PANE_LABEL: &str = "Editor";

/// Prefix before the editor's file picker.
pub const EDITOR_FILES_LABEL: &str = "Files:";

/// The editor's Save button.
pub const EDITOR_SAVE: &str = "Save";

/// Shown briefly after a successful save.
pub const EDITOR_SAVED: &str = "Saved ✓";

/// Shown when no `.rs` files are found in the sandbox.
pub const EDITOR_NO_FILES: &str = "Rusty found no Rust files to edit in this lesson yet.";

/// Heading above a lesson's exercises.
pub const EXERCISES_HEADING: &str = "Exercises";

/// Label for a Worked example.
pub const EXERCISE_WORKED: &str = "Worked example";

/// The grade-this-exercise button.
pub const EXERCISE_CHECK: &str = "Check";

/// Shown while a grade is running on the background thread.
pub const EXERCISE_CHECKING: &str = "Rusty is checking your work…";

/// Reveal button for a predict-then-run exercise.
pub const EXERCISE_REVEAL: &str = "Reveal the answer";

/// Prefix for the Faded exercise's "edit this file" hint, e.g. "Edit src/lib.rs…".
pub const EXERCISE_FADED_EDIT_PREFIX: &str = "Edit ";

/// Suffix for the Faded edit hint.
pub const EXERCISE_FADED_EDIT_SUFFIX: &str = " in the editor, then press Check.";

// --- Annotation pane (the on-screen rustc-style result, prompt §5). ---

/// Headline when a check passes.
pub const ANNOTATION_PASS: &str = "Rusty checked it — that passes. ✓";

/// Headline when the code did not compile.
pub const ANNOTATION_COMPILE_ERROR: &str =
    "It didn't compile yet — here is exactly what the compiler said:";

/// Headline when it compiled but a test failed.
pub const ANNOTATION_TESTS_FAILED: &str = "It compiled, but a test didn't pass yet.";

/// Headline when `cargo run` output didn't match.
pub const ANNOTATION_RUN_MISMATCH: &str = "It ran, but the output wasn't what was expected:";

/// Prefix on an available concept link, e.g. "E0382 → teaches: …".
pub const CONCEPT_LINK_TEACHES: &str = " → teaches: ";

/// Suffix marking a concept link whose lesson isn't authored yet.
pub const CONCEPT_LINK_COMING_SOON: &str = " (coming soon)";

/// Prefix for a lesson's "now run this" prompt, e.g. "▶ run: cargo run".
pub const RUN_PROMPT_PREFIX: &str = "▶ run: ";

/// Callout labels (note/tip/warning).
pub const CALLOUT_NOTE: &str = "Note";
pub const CALLOUT_TIP: &str = "Tip";
pub const CALLOUT_WARNING: &str = "Heads up";

/// Shown in the lesson pane when a lesson fails to load.
pub const LESSON_LOAD_ERROR: &str = "Rusty couldn't load this lesson.";

/// Shown when the learner tries to `cd` out of the lesson sandbox. Third-person,
/// patient voice (§12) — Rusty is on the learner's side.
pub const CD_REFUSED: &str =
    "Rusty's terminal stays inside the lesson directory. Try `cd .` to see where you are.";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_title() {
        assert_eq!(WINDOW_TITLE, "Rusty — Learn Rust by Doing");
    }
}
