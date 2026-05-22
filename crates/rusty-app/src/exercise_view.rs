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

/// Render every exercise. Returns the criterion to grade if a Check was pressed this
/// frame. `checking` disables Check buttons while a grade is in flight.
pub fn render(
    ui: &mut egui::Ui,
    exercises: &[Exercise],
    state: &mut ExerciseState,
    checking: bool,
) -> Option<SuccessCriterion> {
    if exercises.is_empty() {
        return None;
    }
    ui.label(
        egui::RichText::new(voice::EXERCISES_HEADING)
            .size(crate::theme::H2)
            .strong(),
    );
    ui.separator();

    let mut requested: Option<SuccessCriterion> = None;
    for (i, ex) in exercises.iter().enumerate() {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            if let Some(crit) = render_one(ui, i, ex, state, checking) {
                requested = Some(crit);
            }
        });
        ui.add_space(6.0);
    }
    requested
}

fn render_one(
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
            if state.revealed(i) {
                ui.label(crate::theme::section_label("Output:"));
                code_block(ui, expected_output);
                markdown::render_markdown(ui, explanation);
            } else if ui.button(voice::EXERCISE_REVEAL).clicked() {
                state.toggle_reveal(i);
            }
        }
    }

    // Gradeable variants (Faded/Open — see `criterion_for_exercise`) get a Check button.
    let criterion = criterion_for_exercise(ex)?;
    check_button(ui, checking).then(|| criterion.clone())
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

    #[test]
    fn test_exercise_view_renders_each_variant() {
        let exercises = vec![worked(), faded(), open(), predict()];
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
            // Render the hidden state, then reveal the PredictThenRun (index 3) and render
            // again so the revealed branch (expected_output + explanation) also executes.
            let _ = render(ui, &exercises, &mut state, false);
            state.toggle_reveal(3);
            let _ = render(ui, &exercises, &mut state, false);
        });
    }
}
