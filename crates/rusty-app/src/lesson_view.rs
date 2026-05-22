//! Renders a [`Lesson`]'s body blocks into the left pane. Lesson copy comes from the
//! lesson data; only chrome (labels, the run-prompt prefix) lives in [`crate::voice`].

use rusty_curriculum::{Block, CalloutTone, Lesson};

use crate::{markdown, voice};

/// Render the lesson's title + body blocks. The caller owns the scroll area (so the
/// exercises and annotation pane can scroll together with the prose).
pub fn render(ui: &mut egui::Ui, lesson: &Lesson) {
    // The lesson name is THE title — render it larger than any in-body markdown heading
    // (see `theme`), so lesson prose should not repeat the title as its own `# heading`.
    ui.label(
        egui::RichText::new(&lesson.title)
            .size(crate::theme::TITLE)
            .strong(),
    );
    ui.separator();
    for block in &lesson.body {
        render_block(ui, block);
        ui.add_space(8.0);
    }
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

    // A lesson exercising every Block variant and all three callout tones, plus
    // markdown (heading/bold/italic/inline-code/fenced-code/bullets).
    const ALL_BLOCKS: &str = r##"
        id = "t"
        title = "T"
        track = "Foundations"
        estimated_minutes = 1
        starter_project = "s"
        solution_project = "sol"

        [[body]]
        kind = "prose"
        text = "# H\n\n**b** and `c` and *i*\n\n- one\n- two"

        [[body]]
        kind = "code"
        lang = "rust"
        source = "fn main() {}"

        [[body]]
        kind = "now_run"
        command = "cargo run"
        note = "go"

        [[body]]
        kind = "callout"
        tone = "note"
        text = "a note"

        [[body]]
        kind = "callout"
        tone = "tip"
        text = "a tip"

        [[body]]
        kind = "callout"
        tone = "warning"
        text = "a warning"

        [recall_prompt]
        kind = "short_answer"
        question = "q"
        expected = "a"
        explanation = "e"
    "##;

    /// Render every block variant through a real headless egui layout pass; the test
    /// fails if any render branch panics. (egui has no pixel assertion, but `run`
    /// exercises the full layout/galley path without a GPU or window.)
    #[test]
    fn test_render_all_blocks_does_not_panic() {
        let lesson = parse_lesson(ALL_BLOCKS).expect("fixture lesson parses");
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(800.0, 600.0),
            )),
            ..Default::default()
        };
        let _ = ctx.run_ui(input, |ui| {
            render(ui, &lesson);
        });
    }
}
