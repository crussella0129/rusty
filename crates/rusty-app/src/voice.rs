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
