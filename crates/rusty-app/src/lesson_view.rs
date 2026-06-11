//! Renders a [`Lesson`]'s body blocks into the left pane. Lesson copy comes from the
//! lesson data; only chrome (labels, the run-prompt prefix) lives in [`crate::voice`].

use rusty_curriculum::{
    visible_prefix, Block, CalloutTone, Lesson, RecallPrompt, Reference, Step, SuccessCriterion,
};

use crate::exercise_view::{self, ExerciseState};
use crate::{markdown, voice, LessonProgress};

/// Amber used for Rusty's tip (a hint after the learner's first failed Check).
const TIP_COLOR: egui::Color32 = egui::Color32::from_rgb(0xff, 0xb3, 0x00);

/// What the lesson pane wants `main` to do this frame: grade a pressed Check, run a
/// command the learner clicked from a `▶ run` prompt, or both. `Default` is the empty
/// no-op state (`check: None, run: None`).
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct LessonAction {
    /// `Some((step_index, criterion))` when a Check button was pressed.
    pub check: Option<(usize, SuccessCriterion)>,
    /// `Some(command)` when a `▶ run` prompt button was clicked.
    pub run: Option<String>,
    /// `true` if the recall prompt was successfully answered this frame.
    pub recall_passed: bool,
}

/// Whether to show this step's tip: a gating step with a `hint` whose Check has failed at
/// least once. Pure, so the tip-gate is unit-testable without egui.
fn tip_visible(step: &Step, attempts: u32) -> bool {
    step.is_gating() && step.hint.is_some() && attempts >= 1
}

/// Bind a step's index to the criterion its Check produced (the grade→step contract).
/// Pure + unit-tested so the index↔criterion pairing can't silently mis-associate.
fn pair_check(
    step_idx: usize,
    criterion: Option<SuccessCriterion>,
) -> Option<(usize, SuccessCriterion)> {
    criterion.map(|c| (step_idx, c))
}

/// The command a block represents when "run this" is requested. Pure: `NowRun{command}`
/// maps to `Some(command)`; every other block kind maps to `None`. This makes the
/// click→action mapping unit-testable without egui.
fn run_request_for_block(block: &Block) -> Option<String> {
    match block {
        Block::NowRun { command, .. } => Some(command.clone()),
        _ => None,
    }
}

/// Render the lesson's title, then its **visible** steps (progressive disclosure: a
/// gating step hides everything after it until completed). Returns a [`LessonAction`]
/// describing what the learner did this frame (pressed a Check, clicked a ▶ run, or
/// neither). When the whole lesson is complete, the recall prompt + further-reading
/// region is also rendered. The caller owns the scroll area + the annotation pane.
pub fn render(
    ui: &mut egui::Ui,
    lesson: &Lesson,
    progress: &LessonProgress,
    ex_state: &mut ExerciseState,
    recall_state: &mut crate::AppRecallState,
    checking: bool,
) -> LessonAction {
    // The lesson name is THE title — render it larger than any in-body markdown heading
    // (see `theme`), so lesson prose should not repeat the title as its own `# heading`.
    ui.label(
        egui::RichText::new(&lesson.title)
            .size(crate::theme::TITLE)
            .strong(),
    );
    ui.separator();

    let visible = visible_prefix(&lesson.steps, progress.completed());
    let mut action = LessonAction::default();
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
                let mut step_action = LessonAction::default();
                for block in &step.blocks {
                    if let Some(cmd) = render_block(ui, block) {
                        step_action.run = Some(cmd);
                    }
                    ui.add_space(8.0);
                }
                if let Some(exercise) = &step.exercise {
                    egui::Frame::group(ui.style()).show(ui, |ui| {
                        let c = exercise_view::render_exercise(ui, i, exercise, ex_state, checking);
                        if let Some(paired) = pair_check(i, c) {
                            step_action.check = Some(paired);
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
                step_action
            })
            .inner;
        if inner.check.is_some() {
            action.check = inner.check;
        }
        if inner.run.is_some() {
            action.run = inner.run;
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
        if render_recall(ui, &lesson.recall_prompt, recall_state) {
            action.recall_passed = true;
        }
        render_further_reading(ui, &lesson.further_reading);
    }
    action
}

/// Render the recall prompt as an interactive review.
pub fn render_recall(ui: &mut egui::Ui, recall: &RecallPrompt, state: &mut crate::AppRecallState) -> bool {
    ui.label(crate::theme::section_label(voice::RECALL_HEADING));
    let mut just_passed = false;
    match recall {
        RecallPrompt::MultipleChoice {
            question, choices, answer_index, explanation
        } => {
            markdown::render_markdown(ui, question);
            for (i, choice) in choices.iter().enumerate() {
                ui.radio_value(&mut state.selected_index, Some(i), choice);
            }
            if !state.passed && ui.button("Submit").clicked() {
                state.attempts += 1;
                if state.selected_index == Some(*answer_index) {
                    state.passed = true;
                    just_passed = true;
                }
            }
            if state.attempts > 0 {
                if state.passed {
                    ui.label(egui::RichText::new("Correct!").color(egui::Color32::from_rgb(0x4c, 0xaf, 0x50)));
                    markdown::render_markdown(ui, explanation);
                } else {
                    ui.label(egui::RichText::new("Not quite right, try again.").color(egui::Color32::from_rgb(0xf4, 0x43, 0x36)));
                }
            }
        }
        RecallPrompt::ShortAnswer { question, expected, explanation } => {
            markdown::render_markdown(ui, question);
            ui.text_edit_singleline(&mut state.typed_answer);
            if !state.passed && ui.button("Submit").clicked() {
                state.attempts += 1;
                if state.typed_answer.trim().eq_ignore_ascii_case(expected.trim()) {
                    state.passed = true;
                    just_passed = true;
                }
            }
            if state.attempts > 0 {
                if state.passed {
                    ui.label(egui::RichText::new("Correct!").color(egui::Color32::from_rgb(0x4c, 0xaf, 0x50)));
                    markdown::render_markdown(ui, explanation);
                } else {
                    ui.label(egui::RichText::new("Not quite right, try again.").color(egui::Color32::from_rgb(0xf4, 0x43, 0x36)));
                }
            }
        }
    }
    just_passed
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

/// Render one block. Returns `Some(command)` iff a `▶ run` button was clicked this
/// frame; otherwise `None` (the other variants are non-interactive).
fn render_block(ui: &mut egui::Ui, block: &Block) -> Option<String> {
    match block {
        Block::Prose { text } => {
            markdown::render_markdown(ui, text);
            None
        }
        Block::Code { source, .. } => {
            egui::Frame::group(ui.style()).show(ui, |ui| {
                ui.label(egui::RichText::new(source.trim_end()).monospace());
            });
            None
        }
        Block::NowRun { command, note } => {
            let line = format!("{}{}", voice::RUN_PROMPT_PREFIX, command);
            let text = egui::RichText::new(line)
                .monospace()
                .strong()
                .color(ui.visuals().hyperlink_color);
            // `frame(false)` strips the default button fill/stroke/padding so the visual
            // stays the same hyperlink-blue monospace run-prompt as before, while
            // gaining a real click + hover affordance (clicking sends the command to
            // the embedded PTY via the LessonAction returned from `render`).
            let clicked = ui.add(egui::Button::new(text).frame(false)).clicked();
            if let Some(note) = note {
                ui.label(egui::RichText::new(note).weak().small());
            }
            if clicked {
                run_request_for_block(block)
            } else {
                None
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
            None
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

        [[further_reading]]
        title = "The Book"
        url = "https://doc.rust-lang.org/book/"
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
            let mut recall_state = crate::AppRecallState::default();
            let _ = render(ui, &lesson, &fresh, &mut ex_state, &mut recall_state, false);
        });
        headless(|ui| {
            let mut recall_state = crate::AppRecallState::default();
            let _ = render(ui, &lesson, &complete, &mut ex_state, &mut recall_state, false);
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
            let mut recall_state = crate::AppRecallState::default();
            let _ = render(ui, &lesson, &fresh, &mut ex_state, &mut recall_state, false);
        });
    }

    /// Render two frames on one `Context` (the reveal path: `animate_bool_with_time` +
    /// `multiply_opacity`); fails on panic, and asserts the per-step reveal factor the
    /// render used is a valid opacity in `[0, 1]`. (The full 0→1 ramp over wall-clock and
    /// the per-step fade are heartbeat-verified — egui has no view-tree assertion.) (T-503.)
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
                let mut recall_state = crate::AppRecallState::default();
                let _ = render(ui, &lesson, &progress, &mut ex_state, &mut recall_state, false);
            });
        }
        // The same id/target the render uses → the opacity factor it applied is in [0,1].
        let factor =
            ctx.animate_bool_with_time(egui::Id::new(("rusty_step_reveal", 0usize)), true, 0.35);
        assert!(
            (0.0..=1.0).contains(&factor),
            "reveal opacity stays in [0,1]"
        );
    }

    #[test]
    fn test_pair_check_binds_step_index() {
        let c = SuccessCriterion::CargoTestPasses;
        assert_eq!(pair_check(2, Some(c.clone())), Some((2, c)));
        assert_eq!(pair_check(2, None), None);
    }

    #[test]
    fn test_run_request_for_block_now_run() {
        let b = Block::NowRun {
            command: "cargo run".to_string(),
            note: None,
        };
        assert_eq!(run_request_for_block(&b), Some("cargo run".to_string()));
    }

    #[test]
    fn test_run_request_for_other_blocks() {
        assert_eq!(
            run_request_for_block(&Block::Prose {
                text: "x".to_string()
            }),
            None
        );
        assert_eq!(
            run_request_for_block(&Block::Code {
                lang: "rust".to_string(),
                source: "fn main() {}".to_string()
            }),
            None
        );
        assert_eq!(
            run_request_for_block(&Block::Callout {
                tone: rusty_curriculum::CalloutTone::Tip,
                text: "x".to_string()
            }),
            None
        );
    }

    /// C-001: a dedicated headless render of a lesson whose only block is a `NowRun`
    /// proves the button is laid out (the T-601 `Button::new(text).frame(false)` path
    /// doesn't panic). Tighter coverage (i.e. proving it's a *button* and not a
    /// regressed-to-`ui.label`) needs an input-injection harness (`kittest`) and is
    /// heartbeat-verified; the `run_request_for_block` pure test above verifies the
    /// click→command mapping.
    #[test]
    fn test_lesson_view_renders_now_run_button_no_panic() {
        const NOW_RUN_ONLY: &str = r##"
            id = "n"
            title = "N"
            track = "Foundations"
            estimated_minutes = 1
            starter_project = "s"
            solution_project = "sol"

            [[steps]]
            [[steps.blocks]]
            kind = "now_run"
            command = "cargo run"
            note = "go"

            [recall_prompt]
            kind = "short_answer"
            question = "q"
            expected = "a"
            explanation = "e"
        "##;
        let lesson = parse_lesson(NOW_RUN_ONLY).expect("fixture parses");
        let mut ex_state = ExerciseState::default();
        let progress = LessonProgress::new(lesson.steps.len());
        headless(|ui| {
            let mut recall_state = crate::AppRecallState::default();
            let _ = render(ui, &lesson, &progress, &mut ex_state, &mut recall_state, false);
        });
    }

    #[test]
    fn test_lesson_action_default_is_no_op() {
        let a = LessonAction::default();
        assert!(a.check.is_none());
        assert!(a.run.is_none());
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
            let mut recall_state = crate::AppRecallState::default();
            let _ = render(ui, &lesson, &progress, &mut ex_state, &mut recall_state, false); // attempts 0: hidden
        });
        progress.apply(0, &rusty_grader::Verdict::TestsFailed); // attempts[0] = 1
        headless(|ui| {
            let mut recall_state = crate::AppRecallState::default();
            let _ = render(ui, &lesson, &progress, &mut ex_state, &mut recall_state, false); // tip now shown
        });
    }
}
