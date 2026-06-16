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

/// Heading above the end-of-lesson recall prompt.
pub const RECALL_HEADING: &str = "Quick check";

/// Heading above the further-reading links.
pub const FURTHER_READING_HEADING: &str = "Further reading";

/// Label above the tip Rusty offers after a first failed Check.
pub const TIP_LABEL: &str = "Rusty's tip";

/// Shown once every step of the lesson is complete.
pub const LESSON_COMPLETE: &str =
    "Lesson complete — Rusty knew you'd get there. On to the next one.";

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

// --- Mascot representation and state (prompt §12). ---

/// The three states the mascot "Rusty" (large orange dog) can be in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MascotState {
    /// Normal sitting/waiting pose.
    Idle,
    /// Tail up, ears forward, celebrating success.
    Happy,
    /// Head tilted, quizzical eyes, thinking or showing an error.
    Thinking,
}

/// Managing the mascot state and duration timeouts.
pub struct Mascot {
    state: MascotState,
    state_expires_at: Option<std::time::Instant>,
}

impl Mascot {
    /// Create a new Mascot instance starting in the Idle state.
    pub fn new() -> Self {
        Self {
            state: MascotState::Idle,
            state_expires_at: None,
        }
    }

    /// Retrieve the current mascot state, reverting to `Idle` if a temporary state has expired.
    pub fn state(&self) -> MascotState {
        if let Some(expires) = self.state_expires_at {
            if std::time::Instant::now() >= expires {
                return MascotState::Idle;
            }
        }
        self.state
    }

    /// Return the image source for the current state.
    pub fn image(&self) -> egui::ImageSource<'static> {
        match self.state() {
            MascotState::Idle => egui::include_image!("../../../assets/mascot_idle.svg"),
            MascotState::Happy => egui::include_image!("../../../assets/mascot_happy.svg"),
            MascotState::Thinking => egui::include_image!("../../../assets/mascot_thinking.svg"),
        }
    }

    /// Triggered when grading starts. Transition to `Thinking` indefinitely (clearing any expiration).
    pub fn handle_grade_start(&mut self) {
        self.state = MascotState::Thinking;
        self.state_expires_at = None;
    }

    /// Triggered when a grading result is obtained.
    pub fn handle_verdict(&mut self, verdict: &rusty_grader::Verdict) {
        match verdict {
            rusty_grader::Verdict::Pass => {
                self.state = MascotState::Happy;
                // Celebrate for 4 seconds, then return to idle
                self.state_expires_at =
                    Some(std::time::Instant::now() + std::time::Duration::from_secs(4));
            }
            _ => {
                // Tilt head on error for 4 seconds, then return to idle
                self.state = MascotState::Thinking;
                self.state_expires_at =
                    Some(std::time::Instant::now() + std::time::Duration::from_secs(4));
            }
        }
    }

    /// Triggered when a scheduled recall prompt is successfully completed.
    pub fn handle_recall_passed(&mut self) {
        self.state = MascotState::Happy;
        self.state_expires_at = Some(std::time::Instant::now() + std::time::Duration::from_secs(4));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_title() {
        assert_eq!(WINDOW_TITLE, "Rusty — Learn Rust by Doing");
    }
}
