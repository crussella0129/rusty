//! The on-screen annotation pane (prompt §5): renders a grading [`Annotation`] — a
//! headline, the verbatim rustc output, and concept links to the lesson that teaches
//! each error. Pure rendering over the grader's plain model; no grading happens here.

use rusty_grader::{Annotation, ConceptLink, Headline};

use crate::voice;

/// Is this link navigable — i.e. does its lesson exist in `known_lessons`? A link to an
/// unauthored lesson renders as a "coming soon" note, never a dead button.
pub fn link_is_available(link: &ConceptLink, known_lessons: &[String]) -> bool {
    known_lessons.iter().any(|id| id == &link.lesson_id)
}

/// The headline string + colour for an annotation.
fn headline(ui: &egui::Ui, h: Headline) -> (&'static str, egui::Color32) {
    match h {
        Headline::Pass => (
            voice::ANNOTATION_PASS,
            egui::Color32::from_rgb(0x4c, 0xaf, 0x50),
        ),
        Headline::CompileError => (voice::ANNOTATION_COMPILE_ERROR, egui::Color32::LIGHT_RED),
        Headline::TestsFailed => (
            voice::ANNOTATION_TESTS_FAILED,
            egui::Color32::from_rgb(0xff, 0xb3, 0x00),
        ),
        Headline::RunMismatch => (voice::ANNOTATION_RUN_MISMATCH, ui.visuals().hyperlink_color),
    }
}

/// Render an [`Annotation`]: headline, verbatim body blocks (monospace), concept links.
pub fn render(ui: &mut egui::Ui, annotation: &Annotation, known_lessons: &[String]) {
    let (label, color) = headline(ui, annotation.headline);
    ui.label(egui::RichText::new(label).strong().color(color));

    for block in &annotation.body_blocks {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.label(egui::RichText::new(block).monospace());
        });
    }

    for link in &annotation.links {
        if link_is_available(link, known_lessons) {
            let text = format!(
                "{}{}{}",
                link.code,
                voice::CONCEPT_LINK_TEACHES,
                link.lesson_id
            );
            // Single-lesson nav has no target yet (Phase 5/6); the link is informational.
            let _ = ui.link(text);
        } else {
            ui.label(
                egui::RichText::new(format!(
                    "{}{}{}{}",
                    link.code,
                    voice::CONCEPT_LINK_TEACHES,
                    link.lesson_id,
                    voice::CONCEPT_LINK_COMING_SOON
                ))
                .weak(),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusty_grader::{annotate, Diag, Level, Verdict};

    fn err_diag(code: &str) -> Diag {
        Diag {
            code: Some(code.to_string()),
            level: Level::Error,
            message: "m".to_string(),
            rendered: Some(format!("error[{code}]: …")),
            primary_span: None,
        }
    }

    fn headless(mut f: impl FnMut(&mut egui::Ui)) {
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(800.0, 600.0),
            )),
            ..Default::default()
        };
        let _ = ctx.run_ui(input, |ui| f(ui));
    }

    #[test]
    fn test_link_availability() {
        let link = ConceptLink {
            code: "E0382".to_string(),
            lesson_id: "foundations-03-ownership-moves".to_string(),
        };
        let known = vec!["foundations-01-hello".to_string()];
        assert!(
            !link_is_available(&link, &known),
            "unauthored lesson → not available"
        );

        let here = ConceptLink {
            code: "E0425".to_string(),
            lesson_id: "foundations-01-hello".to_string(),
        };
        assert!(
            link_is_available(&here, &known),
            "loaded lesson → available"
        );
    }

    #[test]
    fn test_annotation_pane_renders_all_shapes() {
        let known = vec!["foundations-01-hello".to_string()];
        let verdicts = [
            Verdict::Pass,
            // Two diags so BOTH link-render branches run: E0425 → lesson 1 (live `ui.link`),
            // E0382 → an unauthored lesson (the weak "coming soon" label).
            Verdict::CompileError(vec![err_diag("E0425"), err_diag("E0382")]),
            Verdict::TestsFailed,
            Verdict::RunMismatch {
                expected: "hi".to_string(),
                got: "bye".to_string(),
            },
        ];
        headless(|ui| {
            for v in &verdicts {
                render(ui, &annotate(v), &known);
            }
        });
    }
}
