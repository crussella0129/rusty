//! Renders a [`Lesson`]'s body blocks into the left pane. Lesson copy comes from the
//! lesson data; only chrome (labels, the run-prompt prefix) lives in [`crate::voice`].

use rusty_curriculum::{Block, CalloutTone, Lesson};

use crate::{markdown, voice};

/// Render the whole lesson (title + body blocks) in a scroll area.
pub fn render(ui: &mut egui::Ui, lesson: &Lesson) {
    ui.heading(&lesson.title);
    ui.separator();
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for block in &lesson.body {
                render_block(ui, block);
                ui.add_space(8.0);
            }
        });
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
                ui.label(egui::RichText::new(label).strong().color(color));
                markdown::render_markdown(ui, text);
            });
        }
    }
}
