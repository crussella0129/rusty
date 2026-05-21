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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_title() {
        assert_eq!(WINDOW_TITLE, "Rusty — Learn Rust by Doing");
    }
}
