//! Renders a lesson's [`Exercise`]s (prompt §3/§4: Worked → Faded → Open, plus
//! PredictThenRun) and returns the [`SuccessCriterion`] to grade when a Check button is
//! pressed. The actual grading runs off-thread in `main.rs`; this module is pure UI +
//! the small, testable helpers (`criterion_for_exercise`, the reveal state).

use rusty_curriculum::{Exercise, SuccessCriterion};

use crate::{markdown, voice};

/// The success criterion to grade for an exercise, if it is gradeable. Faded and Open
/// exercises are gradeable; Worked and PredictThenRun are not (this is what suppresses
/// their Check control).
pub fn criterion_for_exercise(ex: &Exercise) -> Option<&SuccessCriterion> {
    match ex {
        Exercise::Faded {
            success_criterion, ..
        }
        | Exercise::Open {
            success_criterion, ..
        } => Some(success_criterion),
        Exercise::Worked { .. } | Exercise::PredictThenRun { .. } => None,
    }
}

/// Per-exercise transient UI state (the predict-then-run reveal toggles).
#[derive(Default)]
pub struct ExerciseState {
    revealed: Vec<bool>,
}

impl ExerciseState {
    /// Whether exercise `i`'s answer has been revealed (default false).
    pub fn revealed(&self, i: usize) -> bool {
        self.revealed.get(i).copied().unwrap_or(false)
    }

    /// Flip exercise `i`'s reveal flag, growing the backing store as needed.
    pub fn toggle_reveal(&mut self, i: usize) {
        if i >= self.revealed.len() {
            self.revealed.resize(i + 1, false);
        }
        self.revealed[i] = !self.revealed[i];
    }
}

/// Render one step's exercise (identified by its step index `i`, used for the
/// predict-then-run reveal state). Returns the criterion to grade if its Check was
/// pressed this frame. `checking` disables the Check button while a grade is in flight.
pub fn render_exercise(
    ui: &mut egui::Ui,
    i: usize,
    ex: &Exercise,
    state: &mut ExerciseState,
    checking: bool,
) -> Option<SuccessCriterion> {
    // Draw the variant-specific content.
    match ex {
        Exercise::Worked {
            prompt,
            code,
            annotation,
        } => {
            ui.label(crate::theme::section_label(voice::EXERCISE_WORKED));
            markdown::render_markdown(ui, prompt);
            code_block(ui, code);
            ui.label(egui::RichText::new(annotation).weak());
        }
        Exercise::Faded {
            prompt, file_path, ..
        } => {
            markdown::render_markdown(ui, prompt);
            ui.label(
                egui::RichText::new(format!(
                    "{}{}{}",
                    voice::EXERCISE_FADED_EDIT_PREFIX,
                    file_path.display(),
                    voice::EXERCISE_FADED_EDIT_SUFFIX
                ))
                .italics(),
            );
        }
        Exercise::Open { prompt, .. } => {
            markdown::render_markdown(ui, prompt);
        }
        Exercise::PredictThenRun {
            code,
            question,
            expected_output,
            explanation,
        } => {
            markdown::render_markdown(ui, question);
            code_block(ui, code);
            // Call animate_bool_with_time every frame (target = current reveal state) so
            // the stored animation value tracks the false→true transition on Reveal-click
            // and ramps 0→1 over ~0.35s. The render scope multiplies widget opacity by the
            // factor, fading the Output/explanation in instead of snapping. (T-604.)
            let factor = ui.ctx().animate_bool_with_time(
                egui::Id::new(("rusty_reveal_fade", i)),
                state.revealed(i),
                0.35,
            );
            if state.revealed(i) {
                ui.scope(|ui| {
                    ui.multiply_opacity(factor);
                    ui.label(crate::theme::section_label("Output:"));
                    code_block(ui, expected_output);
                    markdown::render_markdown(ui, explanation);
                });
            } else if ui.button(voice::EXERCISE_REVEAL).clicked() {
                state.toggle_reveal(i);
            }
        }
    }

    // Gradeable variants (Faded/Open — see `criterion_for_exercise`) get a Check button.
    let criterion = criterion_for_exercise(ex)?;
    let clicked = check_button(ui, checking);
    clicked.then(|| criterion.clone())
}

/// A read-only monospace code block.
fn code_block(ui: &mut egui::Ui, code: &str) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.label(egui::RichText::new(code.trim_end()).monospace());
    });
}

/// A Check button (disabled while a grade is in flight). Returns whether it was clicked.
fn check_button(ui: &mut egui::Ui, checking: bool) -> bool {
    let clicked = ui
        .add_enabled(!checking, egui::Button::new(voice::EXERCISE_CHECK))
        .clicked();
    if checking {
        ui.label(egui::RichText::new(voice::EXERCISE_CHECKING).weak());
    }
    clicked
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn faded() -> Exercise {
        Exercise::Faded {
            prompt: "fill it in".to_string(),
            file_path: PathBuf::from("src/lib.rs"),
            check_command: "cargo test".to_string(),
            success_criterion: SuccessCriterion::CargoTestPasses,
        }
    }

    fn open() -> Exercise {
        Exercise::Open {
            prompt: "write it".to_string(),
            check_command: "cargo run".to_string(),
            success_criterion: SuccessCriterion::CargoRunOutputMatches {
                expected: "hi".to_string(),
            },
        }
    }

    fn worked() -> Exercise {
        Exercise::Worked {
            prompt: "watch".to_string(),
            code: "fn main() {}".to_string(),
            annotation: "see?".to_string(),
        }
    }

    fn predict() -> Exercise {
        Exercise::PredictThenRun {
            code: "println!(\"3\");".to_string(),
            question: "what prints?".to_string(),
            expected_output: "3".to_string(),
            explanation: "it prints 3".to_string(),
        }
    }

    #[test]
    fn test_criterion_for_faded_and_open() {
        assert!(criterion_for_exercise(&faded()).is_some());
        assert!(criterion_for_exercise(&open()).is_some());
        assert!(criterion_for_exercise(&worked()).is_none());
        assert!(criterion_for_exercise(&predict()).is_none());
    }

    #[test]
    fn test_predict_then_run_hides_answer_until_reveal() {
        let mut s = ExerciseState::default();
        assert!(!s.revealed(0), "answer starts hidden");
        s.toggle_reveal(0);
        assert!(s.revealed(0), "after reveal it shows");
    }

    /// Render two frames on one `Context` so the reveal-fade `animate_bool_with_time`
    /// actually tracks the false→true transition; the test fails on panic, and the
    /// stored animation value (queried with the same id/target the render uses) sits in
    /// `[0, 1]`. (T-604.)
    #[test]
    fn test_predict_reveal_animates_without_panic() {
        let mut state = ExerciseState::default();
        let ex = predict();
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(800.0, 600.0),
            )),
            ..Default::default()
        };
        // Frame 1: revealed=false; animate sees `false` for the first time.
        let _ = ctx.run_ui(input.clone(), |ui| {
            let _ = render_exercise(ui, 0, &ex, &mut state, false);
        });
        // Flip to revealed; Frame 2: animate sees the false→true transition and ramps.
        state.toggle_reveal(0);
        let _ = ctx.run_ui(input, |ui| {
            let _ = render_exercise(ui, 0, &ex, &mut state, false);
        });
        let factor =
            ctx.animate_bool_with_time(egui::Id::new(("rusty_reveal_fade", 0usize)), true, 0.35);
        assert!(
            (0.0..=1.0).contains(&factor),
            "reveal-fade opacity stays in [0,1]"
        );
    }

    #[test]
    fn test_exercise_view_renders_each_variant() {
        let exercises = [worked(), faded(), open(), predict()];
        let mut state = ExerciseState::default();
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(800.0, 600.0),
            )),
            ..Default::default()
        };
        let _ = ctx.run_ui(input, |ui| {
            // Render each variant (predict at index 3); then reveal it and render again so
            // the revealed branch (expected_output + explanation) also executes.
            for (i, ex) in exercises.iter().enumerate() {
                let _ = render_exercise(ui, i, ex, &mut state, false);
            }
            state.toggle_reveal(3);
            let _ = render_exercise(ui, 3, &exercises[3], &mut state, false);
        });
    }
}
