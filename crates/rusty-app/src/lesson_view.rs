//! Renders a [`Lesson`]'s body blocks into the left pane. Lesson copy comes from the
//! lesson data; only chrome (labels, the run-prompt prefix) lives in [`crate::voice`].

use rusty_curriculum::{
    visible_prefix, Block, CalloutTone, Lesson, RecallPrompt, Reference, Step, SuccessCriterion,
};

use crate::exercise_view::{self, ExerciseState};
use crate::{markdown, voice, LessonProgress};

/// Amber used for Rusty's tip (a hint after the learner's first failed Check).
const TIP_COLOR: egui::Color32 = egui::Color32::from_rgb(0xff, 0xb3, 0x00);

/// Whether to show this step's tip: a gating step with a `hint` whose Check has failed at
/// least once. Pure, so the tip-gate is unit-testable without egui.
fn tip_visible(step: &Step, attempts: u32) -> bool {
    step.is_gating() && step.hint.is_some() && attempts >= 1
}

/// Render the lesson's title, then its **visible** steps (progressive disclosure: a
/// gating step hides everything after it until completed). Returns `Some((step_index,
/// criterion))` when a step's Check was pressed. When the whole lesson is complete, the
/// recall prompt + further-reading region is also rendered. The caller owns the scroll
/// area + the annotation pane.
pub fn render(
    ui: &mut egui::Ui,
    lesson: &Lesson,
    progress: &LessonProgress,
    ex_state: &mut ExerciseState,
    checking: bool,
) -> Option<(usize, SuccessCriterion)> {
    // The lesson name is THE title — render it larger than any in-body markdown heading
    // (see `theme`), so lesson prose should not repeat the title as its own `# heading`.
    ui.label(
        egui::RichText::new(&lesson.title)
            .size(crate::theme::TITLE)
            .strong(),
    );
    ui.separator();

    let visible = visible_prefix(&lesson.steps, progress.completed());
    let mut check: Option<(usize, SuccessCriterion)> = None;
    for (i, step) in lesson.steps.iter().take(visible).enumerate() {
        // Each step fades in the first time it becomes visible: a stable per-step id with
        // a `true` target ramps 0→1 once (already-visible steps sit at 1). This is the
        // "materialize" effect when a gate is passed and the next step appears.
        let factor =
            ui.ctx()
                .animate_bool_with_time(egui::Id::new(("rusty_step_reveal", i)), true, 0.35);
        let inner = ui
            .scope(|ui| {
                ui.multiply_opacity(factor);
                let mut step_check: Option<(usize, SuccessCriterion)> = None;
                for block in &step.blocks {
                    render_block(ui, block);
                    ui.add_space(8.0);
                }
                if let Some(exercise) = &step.exercise {
                    egui::Frame::group(ui.style()).show(ui, |ui| {
                        if let Some(c) =
                            exercise_view::render_exercise(ui, i, exercise, ex_state, checking)
                        {
                            step_check = Some((i, c));
                        }
                        // After the first failed Check, Rusty offers the step's tip.
                        if tip_visible(step, progress.attempts(i)) {
                            if let Some(hint) = &step.hint {
                                ui.add_space(4.0);
                                ui.label(
                                    crate::theme::section_label(voice::TIP_LABEL).color(TIP_COLOR),
                                );
                                markdown::render_markdown(ui, hint);
                            }
                        }
                    });
                    ui.add_space(6.0);
                }
                step_check
            })
            .inner;
        if inner.is_some() {
            check = inner;
        }
    }

    // The lesson's wrap-up (a complete flourish + recall + further reading) materializes
    // once every step is done.
    if progress.all_complete() {
        ui.separator();
        ui.label(
            egui::RichText::new(voice::LESSON_COMPLETE)
                .size(crate::theme::H2)
                .strong()
                .color(egui::Color32::from_rgb(0x4c, 0xaf, 0x50)),
        );
        ui.add_space(6.0);
        render_recall(ui, &lesson.recall_prompt);
        render_further_reading(ui, &lesson.further_reading);
    }
    check
}

/// Render the recall prompt as a read-only review (interactive grading is a later phase).
fn render_recall(ui: &mut egui::Ui, recall: &RecallPrompt) {
    ui.label(crate::theme::section_label(voice::RECALL_HEADING));
    match recall {
        RecallPrompt::MultipleChoice {
            question, choices, ..
        } => {
            markdown::render_markdown(ui, question);
            for choice in choices {
                ui.label(format!("• {choice}"));
            }
        }
        RecallPrompt::ShortAnswer { question, .. } => markdown::render_markdown(ui, question),
    }
}

/// Render further-reading links.
fn render_further_reading(ui: &mut egui::Ui, refs: &[Reference]) {
    if refs.is_empty() {
        return;
    }
    ui.add_space(6.0);
    ui.label(crate::theme::section_label(voice::FURTHER_READING_HEADING));
    for r in refs {
        ui.hyperlink_to(&r.title, &r.url);
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

    /// Render every block variant + an inline exercise through a real headless egui
    /// layout pass; the test fails if any render branch panics. Renders both with fresh
    /// progress and with a complete progress (which also renders the recall + further-
    /// reading wrap-up). (egui has no pixel assertion, but `run` exercises the full
    /// layout/galley path without a GPU.)
    #[test]
    fn test_render_all_blocks_does_not_panic() {
        let lesson = parse_lesson(ALL_BLOCKS).expect("fixture lesson parses");
        let mut ex_state = ExerciseState::default();
        let fresh = LessonProgress::new(lesson.steps.len());
        let mut complete = LessonProgress::new(lesson.steps.len());
        for i in 0..lesson.steps.len() {
            complete.apply(i, &rusty_grader::Verdict::Pass);
        }
        headless(|ui| {
            let _ = render(ui, &lesson, &fresh, &mut ex_state, false);
        });
        headless(|ui| {
            let _ = render(ui, &lesson, &complete, &mut ex_state, false);
        });
    }

    /// A lesson `[prose, faded, open]` with fresh progress: the Faded gates, so the Open
    /// step must not render. Verified by `visible_prefix` (curriculum) + a no-panic pass.
    #[test]
    fn test_render_hides_steps_past_gate() {
        const GATED: &str = r##"
            id = "g"
            title = "G"
            track = "Foundations"
            estimated_minutes = 1
            starter_project = "s"
            solution_project = "sol"

            [[steps]]
            [[steps.blocks]]
            kind = "prose"
            text = "intro"

            [[steps]]
            [steps.exercise]
            kind = "faded"
            prompt = "faded"
            file_path = "src/main.rs"
            check_command = "cargo test"
            success_criterion = { kind = "cargo_test_passes" }

            [[steps]]
            [steps.exercise]
            kind = "open"
            prompt = "open"
            check_command = "cargo run"
            success_criterion = { kind = "cargo_run_output_matches", expected = "x" }

            [recall_prompt]
            kind = "short_answer"
            question = "q"
            expected = "a"
            explanation = "e"
        "##;
        let lesson = parse_lesson(GATED).expect("gated fixture parses");
        let mut ex_state = ExerciseState::default();
        let fresh = LessonProgress::new(lesson.steps.len());
        assert_eq!(
            rusty_curriculum::visible_prefix(&lesson.steps, fresh.completed()),
            2,
            "the Open step is gated behind the incomplete Faded"
        );
        headless(|ui| {
            let _ = render(ui, &lesson, &fresh, &mut ex_state, false);
        });
    }

    /// Render two frames on one `Context` so the reveal animation actually advances
    /// (`animate_bool_with_time` + `multiply_opacity`); the test fails if either frame
    /// panics. (T-503.)
    #[test]
    fn test_lesson_pane_animates_without_panic() {
        let lesson = parse_lesson(ALL_BLOCKS).expect("fixture lesson parses");
        let mut ex_state = ExerciseState::default();
        let progress = LessonProgress::new(lesson.steps.len());
        let ctx = egui::Context::default();
        let input = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::pos2(0.0, 0.0),
                egui::vec2(800.0, 600.0),
            )),
            ..Default::default()
        };
        for _ in 0..2 {
            let _ = ctx.run_ui(input.clone(), |ui| {
                let _ = render(ui, &lesson, &progress, &mut ex_state, false);
            });
        }
    }

    fn gating_step_with_hint() -> Step {
        Step {
            blocks: vec![],
            exercise: Some(rusty_curriculum::Exercise::Faded {
                prompt: "p".into(),
                file_path: "src/main.rs".into(),
                check_command: "cargo test".into(),
                success_criterion: rusty_curriculum::SuccessCriterion::CargoTestPasses,
            }),
            hint: Some("define it".into()),
        }
    }

    #[test]
    fn test_tip_visible_predicate() {
        let mut s = gating_step_with_hint();
        assert!(!tip_visible(&s, 0), "no tip before any failed Check");
        assert!(tip_visible(&s, 1), "tip after the first failure");
        s.hint = None;
        assert!(!tip_visible(&s, 3), "no tip without a hint");
        let worked = Step {
            blocks: vec![],
            exercise: Some(rusty_curriculum::Exercise::Worked {
                prompt: "p".into(),
                code: "c".into(),
                annotation: "a".into(),
            }),
            hint: Some("h".into()),
        };
        assert!(!tip_visible(&worked, 5), "a non-gating step never tips");
    }

    /// A one-gating-step lesson with a hint: attempts=0 hides the tip, attempts=1 shows it
    /// — both render without panic. (T-504.)
    #[test]
    fn test_tip_hidden_then_shown_render() {
        const HINTED: &str = r##"
            id = "h"
            title = "H"
            track = "Foundations"
            estimated_minutes = 1
            starter_project = "s"
            solution_project = "sol"

            [[steps]]
            hint = "Rusty says: define `greeting`."
            [steps.exercise]
            kind = "faded"
            prompt = "fill it in"
            file_path = "src/main.rs"
            check_command = "cargo test"
            success_criterion = { kind = "cargo_test_passes" }

            [recall_prompt]
            kind = "short_answer"
            question = "q"
            expected = "a"
            explanation = "e"
        "##;
        let lesson = parse_lesson(HINTED).expect("hinted fixture parses");
        let mut ex_state = ExerciseState::default();
        let mut progress = LessonProgress::new(lesson.steps.len());
        headless(|ui| {
            let _ = render(ui, &lesson, &progress, &mut ex_state, false); // attempts 0: hidden
        });
        progress.apply(0, &rusty_grader::Verdict::TestsFailed); // attempts[0] = 1
        headless(|ui| {
            let _ = render(ui, &lesson, &progress, &mut ex_state, false); // tip now shown
        });
    }
}
