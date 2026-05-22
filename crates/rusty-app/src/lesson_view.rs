//! Renders a [`Lesson`]'s body blocks into the left pane. Lesson copy comes from the
//! lesson data; only chrome (labels, the run-prompt prefix) lives in [`crate::voice`].

use rusty_curriculum::{Block, CalloutTone, Lesson, SuccessCriterion};

use crate::exercise_view::{self, ExerciseState};
use crate::{markdown, voice};

/// Render the lesson's title + its steps (each step's blocks, then its optional
/// exercise). Returns the [`SuccessCriterion`] of a pressed Check, if any. The caller
/// owns the scroll area + the annotation pane. (Gating/progress is wired in T-502; here
/// every step renders.)
pub fn render(
    ui: &mut egui::Ui,
    lesson: &Lesson,
    ex_state: &mut ExerciseState,
    checking: bool,
) -> Option<SuccessCriterion> {
    // The lesson name is THE title — render it larger than any in-body markdown heading
    // (see `theme`), so lesson prose should not repeat the title as its own `# heading`.
    ui.label(
        egui::RichText::new(&lesson.title)
            .size(crate::theme::TITLE)
            .strong(),
    );
    ui.separator();

    let mut check: Option<SuccessCriterion> = None;
    for (i, step) in lesson.steps.iter().enumerate() {
        for block in &step.blocks {
            render_block(ui, block);
            ui.add_space(8.0);
        }
        if let Some(exercise) = &step.exercise {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                if let Some(c) = exercise_view::render_exercise(ui, i, exercise, ex_state, checking)
                {
                    check = Some(c);
                }
            });
            ui.add_space(6.0);
        }
    }
    check
}

fn render_block(ui: &mut egui::Ui, block: &Block) {
    match block {
        Block::Prose { text } => markdown::render_markdown(ui, text),
        Block::Code { source, .. } => {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.label(egui::RichText::new(source.trim_end()).monospace());
            });
        }
        Block::NowRun { command, note } => {
            let line = format!("{}{}", voice::RUN_PROMPT_PREFIX, command);
            ui.label(
                egui::RichText::new(line)
                    .monospace()
                    .strong()
                    .color(ui.visuals().hyperlink_color),
            );
            if let Some(note) = note {
                ui.label(egui::RichText::new(note).weak().small());
            }
        }
        Block::Callout { tone, text } => {
            let (label, color) = match tone {
                CalloutTone::Note => (voice::CALLOUT_NOTE, ui.visuals().hyperlink_color),
                CalloutTone::Tip => (
                    voice::CALLOUT_TIP,
                    egui::Color32::from_rgb(0x4c, 0xaf, 0x50),
                ),
                CalloutTone::Warning => (
                    voice::CALLOUT_WARNING,
                    egui::Color32::from_rgb(0xff, 0xb3, 0x00),
                ),
            };
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.label(crate::theme::section_label(label).color(color));
                markdown::render_markdown(ui, text);
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusty_curriculum::parse_lesson;

    // A lesson exercising every Block variant + all three callout tones + an inline
    // exercise in a step, plus markdown (heading/bold/italic/inline-code/fenced/bullets).
    const ALL_BLOCKS: &str = r##"
        id = "t"
        title = "T"
        track = "Foundations"
        estimated_minutes = 1
        starter_project = "s"
        solution_project = "sol"

        [[steps]]

        [[steps.blocks]]
        kind = "prose"
        text = "# H\n\n**b** and `c` and *i*\n\n- one\n- two"

        [[steps.blocks]]
        kind = "code"
        lang = "rust"
        source = "fn main() {}"

        [[steps.blocks]]
        kind = "now_run"
        command = "cargo run"
        note = "go"

        [[steps.blocks]]
        kind = "callout"
        tone = "note"
        text = "a note"

        [[steps.blocks]]
        kind = "callout"
        tone = "tip"
        text = "a tip"

        [[steps.blocks]]
        kind = "callout"
        tone = "warning"
        text = "a warning"

        [[steps]]
        [steps.exercise]
        kind = "open"
        prompt = "write it"
        check_command = "cargo run"
        success_criterion = { kind = "cargo_run_output_matches", expected = "x" }

        [recall_prompt]
        kind = "short_answer"
        question = "q"
        expected = "a"
        explanation = "e"
    "##;

    /// Render every block variant + an inline exercise through a real headless egui
    /// layout pass; the test fails if any render branch panics. (egui has no pixel
    /// assertion, but `run` exercises the full layout/galley path without a GPU.)
    #[test]
    fn test_render_all_blocks_does_not_panic() {
        let lesson = parse_lesson(ALL_BLOCKS).expect("fixture lesson parses");
        let mut ex_state = ExerciseState::default();
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(800.0, 600.0),
            )),
            ..Default::default()
        };
        let _ = ctx.run_ui(input, |ui| {
            let _ = render(ui, &lesson, &mut ex_state, false);
        });
    }
}
