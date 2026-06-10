//! `rusty-app` — the eframe binary for Rusty.
//!
//! Phase 1: a two-pane window — an (empty) lesson pane on the left and a live,
//! sandboxed embedded terminal on the right. The terminal runs a real shell via
//! `rusty-host`, renders its ANSI output through `rusty-terminal`, forwards
//! keystrokes, answers the ConPTY cursor-position handshake, and refuses `cd`s that
//! would escape the lesson sandbox.

mod annotation;
mod editor;
mod exercise_view;
mod lesson_view;
mod markdown;
mod state;
mod theme;
mod voice;

use std::path::PathBuf;
use std::sync::mpsc::{Receiver, TryRecvError};

use editor::Editor;
use eframe::egui;
use exercise_view::ExerciseState;
use rusty_curriculum::{Lesson, SuccessCriterion};
use rusty_grader::{annotate, Annotation, Verdict};
use rusty_host::{
    default_shell, is_sandbox_healthy, load_lesson, prepare_sandbox, resolve_cd, CdOutcome,
    PtySession, LspSession,
};
use rusty_terminal::{terminal_ui, Terminal};

const INIT_ROWS: usize = 24;
const INIT_COLS: usize = 80;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        voice::WINDOW_TITLE,
        native_options,
        Box::new(|cc| Ok(Box::new(RustyApp::new(cc)))),
    )
}

/// What submitting the current input line should do, decided purely from the line +
/// sandbox paths. Separated from the byte-writing in `handle_typed` so it is unit-
/// testable without a live PTY.
#[derive(Debug, PartialEq, Eq)]
enum SubmitAction {
    /// Forward the Enter to the shell unchanged.
    Forward,
    /// Refuse the line (sandbox-escaping `cd`); cancel + show the refusal.
    Refuse,
    /// An in-sandbox `cd`: forward it and track the new working directory.
    ChangeDir(PathBuf),
}

fn submit_action(line: &str, cwd: &std::path::Path, root: &std::path::Path) -> SubmitAction {
    match resolve_cd(line, cwd, root) {
        CdOutcome::Refused => SubmitAction::Refuse,
        CdOutcome::Allowed(path) => SubmitAction::ChangeDir(path),
        CdOutcome::NotCd => SubmitAction::Forward,
    }
}

/// Map a finished grade result into the (annotation, grade-error) the pane shows.
/// Pure, so the channel-delivered outcome is testable without the egui loop.
fn grade_outcome(result: &Result<Verdict, String>) -> (Option<Annotation>, Option<String>) {
    match result {
        Ok(verdict) => (Some(annotate(verdict)), None),
        Err(msg) => (None, Some(msg.clone())),
    }
}

/// The bytes typed into the embedded PTY when the learner clicks a ▶ run prompt: the
/// command followed by `\r` (the shell's Enter). Pure so the byte format is unit-tested
/// without spinning up a real PTY — a regression to `\n` (which `cmd.exe` ignores) or
/// missing-Enter would be caught directly.
fn pty_bytes_for_run(cmd: &str) -> Vec<u8> {
    format!("{cmd}\r").into_bytes()
}

/// Sprint-6 boundary guard: panic if `step` is not a gradeable (`Faded`/`Open`) step.
/// Called at the single chokepoint between a UI Check signal and `start_grade` so a
/// misroute (a Reveal click somehow producing a check_request, etc.) panics loudly with
/// the step index and the actual exercise kind — directly catching the Sprint-6 mystery
/// bug at the moment it crosses the grade boundary.
fn enforce_gradeable_step(lesson: &Lesson, step: usize) {
    let actual = lesson.steps.get(step).and_then(|s| s.exercise.as_ref());
    let kind = match actual {
        None => "no-exercise",
        Some(rusty_curriculum::Exercise::Worked { .. }) => "Worked",
        Some(rusty_curriculum::Exercise::Faded { .. }) => "Faded",
        Some(rusty_curriculum::Exercise::Open { .. }) => "Open",
        Some(rusty_curriculum::Exercise::PredictThenRun { .. }) => "PredictThenRun",
    };
    assert!(
        matches!(
            actual,
            Some(rusty_curriculum::Exercise::Faded { .. })
                | Some(rusty_curriculum::Exercise::Open { .. })
        ),
        "start_grade boundary: step {step} is not gradeable (got {kind})"
    );
}

/// In-memory learner progress through the current lesson's steps (one slot per step).
/// Persistence across launches is deferred (the old Phase 5); this resets each run.
#[derive(Default)]
struct LessonProgress {
    /// Per-step: has this step's gate been satisfied (a `Verdict::Pass`)?
    completed: Vec<bool>,
    /// Per-step: how many failed Checks so far (drives the tip after the first).
    attempts: Vec<u32>,
}

impl LessonProgress {
    fn new(steps: usize) -> Self {
        Self {
            completed: vec![false; steps],
            attempts: vec![0; steps],
        }
    }

    /// Fold a finished grade for `step` into progress: a `Pass` completes the step
    /// (revealing the next); anything else bumps its attempt count (drives the tip).
    fn apply(&mut self, step: usize, verdict: &Verdict) {
        if step >= self.completed.len() {
            return;
        }
        if matches!(verdict, Verdict::Pass) {
            self.completed[step] = true;
        } else {
            self.attempts[step] += 1;
        }
    }

    /// Whether every step is complete (the lesson is finished). Empty → not complete.
    fn all_complete(&self) -> bool {
        !self.completed.is_empty() && self.completed.iter().all(|&c| c)
    }

    /// Borrow the per-step completion flags (for `visible_prefix`).
    fn completed(&self) -> &[bool] {
        &self.completed
    }

    /// Failed-Check count for `step` (0 if out of range) — drives the tip after the first.
    fn attempts(&self, step: usize) -> u32 {
        self.attempts.get(step).copied().unwrap_or(0)
    }
}

/// Fallback sandbox dir when the lesson can't be loaded OR `prepare_sandbox` produced
/// an unhealthy sandbox. Lives in **OS temp** (s7 ADR — resolves C-005), not
/// `cwd/workspace/lessons/spike`: the old `spike` path under the repo's workspace was
/// the *same* path the Sprint-6 inner-Rusty cascade created, and cargo invoked from
/// there would escalate right back into the parent virtual workspace. The OS temp dir
/// has no parent `Cargo.toml`, so cargo cleanly errors instead of escalating — the PTY
/// can spawn here safely even in the error path.
fn fallback_sandbox() -> PathBuf {
    let root = std::env::temp_dir().join("rusty-fallback-sandbox");
    let _ = std::fs::create_dir_all(&root);
    root
}

/// The Foundations track curriculum.
const CURRICULUM: &[&str] = &[
    "content/lessons/foundations-01-hello",
    "content/lessons/foundations-02-variables",
    "content/lessons/foundations-03-ownership",
    "content/lessons/foundations-04-borrows",
    "content/lessons/foundations-05-structs",
    "content/lessons/foundations-06-enums",
];

#[derive(PartialEq, Eq)]
enum AppMode {
    DueReviews,
    Lesson,
}

#[derive(Default, Clone, Debug)]
pub struct AppRecallState {
    pub selected_index: Option<usize>,
    pub typed_answer: String,
    pub attempts: u32,
    pub passed: bool,
}

struct RustyApp {
    term: Terminal,
    session: PtySession,
    root: PathBuf,
    cwd: PathBuf,
    dims: (usize, usize),
    /// Best-effort mirror of the line currently being typed, for `cd` interception.
    line: String,
    /// The loaded lesson, or `None` if loading failed (then `load_error` is set).
    lesson: Option<Lesson>,
    load_error: Option<String>,
    /// The code editor over the lesson sandbox's `.rs` files.
    editor: Editor,
    lsp_session: Option<std::sync::Arc<LspSession>>,
    /// Lesson ids that exist (so concept-links to them render as live, not "coming soon").
    known_lessons: Vec<String>,
    /// Per-exercise UI state (predict-then-run reveal toggles).
    ex_state: ExerciseState,
    /// An in-flight grade running on a background thread (process #2), if any.
    grade_job: Option<Receiver<Result<Verdict, String>>>,
    /// The annotation pane's current content (the last verdict), if any.
    annotation: Option<Annotation>,
    /// A host-side grade error (couldn't run cargo at all), shown plainly.
    grade_error: Option<String>,
    /// Progressive-disclosure progress through the lesson's steps (in-memory).
    progress: LessonProgress,
    /// Which step the in-flight grade is for (so its result updates that step).
    pending_step: Option<usize>,
    /// Persistent app state containing completed lessons and SM-2 schedules.
    persistent_state: state::PersistentState,
    /// Mode the app is in: due reviews vs normal lesson viewing.
    app_mode: AppMode,
    /// State for the currently displayed recall prompt.
    recall_state: AppRecallState,
}

impl RustyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let persistent_state = state::PersistentState::load(&state::PersistentState::default_path());

        let active_lesson_rel = CURRICULUM
            .iter()
            .find(|rel| {
                let id = rel.split('/').last().unwrap();
                !persistent_state.completed_lessons.contains(&rusty_curriculum::LessonId(id.to_string()))
            })
            .unwrap_or(CURRICULUM.last().unwrap());

        let cwd0 = std::env::current_dir().unwrap_or_default();
        let content_dir = cwd0.join(active_lesson_rel);
        let workspace_root = cwd0.join("workspace");

        // Load lesson 1, prepare its sandbox, validate that the result is structurally
        // healthy (s7 health check — defends against the Sprint-6 bug class even if
        // `prepare_sandbox` ever silently returns a corrupt dir), and pick the PTY's
        // working directory. No `?` — `new` is infallible; failures surface in the
        // lesson pane. The fallback is the OS-temp safe dir (see `fallback_sandbox`).
        let (lesson, load_error, sandbox) = match load_lesson(&content_dir) {
            Ok(lesson) => match prepare_sandbox(&content_dir, &workspace_root, &lesson.id.0) {
                Ok(sandbox) if is_sandbox_healthy(&sandbox) => (Some(lesson), None, sandbox),
                Ok(sandbox) => (
                    Some(lesson),
                    Some(format!(
                        "sandbox at {} is missing Cargo.toml / src/main.rs / [workspace] \
                         table — try deleting it so Rusty can recopy the starter",
                        sandbox.display()
                    )),
                    fallback_sandbox(),
                ),
                Err(err) => (Some(lesson), Some(format!("{err:#}")), fallback_sandbox()),
            },
            Err(err) => (None, Some(format!("{err:#}")), fallback_sandbox()),
        };

        let ctx = cc.egui_ctx.clone();
        let session = PtySession::spawn(
            &default_shell(),
            &sandbox,
            INIT_ROWS as u16,
            INIT_COLS as u16,
            move || ctx.request_repaint(),
        )
        .expect("spawn shell");

        let lsp_session = if load_error.is_none() {
            LspSession::new(&sandbox).ok().map(std::sync::Arc::new)
        } else {
            None
        };

        let editor = Editor::new(&sandbox, lsp_session.clone());
        let known_lessons = lesson
            .as_ref()
            .map(|l| vec![l.id.0.clone()])
            .unwrap_or_default();
        let progress = LessonProgress::new(lesson.as_ref().map(|l| l.steps.len()).unwrap_or(0));

        let current_lesson_index = persistent_state.current_lesson_index;
        let has_due_reviews = persistent_state.concept_reviews.values().any(|r| r.due_at_lesson <= current_lesson_index);
        let app_mode = if has_due_reviews { AppMode::DueReviews } else { AppMode::Lesson };

        Self {
            term: Terminal::new(INIT_ROWS, INIT_COLS),
            session,
            cwd: sandbox.clone(),
            root: sandbox,
            dims: (INIT_ROWS, INIT_COLS),
            line: String::new(),
            lesson,
            load_error,
            editor,
            lsp_session,
            known_lessons,
            ex_state: ExerciseState::default(),
            grade_job: None,
            annotation: None,
            grade_error: None,
            progress,
            pending_step: None,
            persistent_state,
            app_mode,
            recall_state: AppRecallState::default(),
        }
    }

    /// Poll the background grade thread; when it finishes, build the annotation (or
    /// surface a host error) and clear the in-flight job.
    fn poll_grade(&mut self) {
        let Some(rx) = &self.grade_job else { return };
        match rx.try_recv() {
            Ok(received) => {
                // Update the graded step's progress (a `Pass` reveals the next step), then
                // render the verdict in the annotation pane.
                if let (Ok(verdict), Some(step)) = (&received, self.pending_step) {
                    self.progress.apply(step, verdict);
                }
                (self.annotation, self.grade_error) = grade_outcome(&received);
                self.grade_job = None;
                self.pending_step = None;
            }
            Err(TryRecvError::Empty) => {} // still running
            Err(TryRecvError::Disconnected) => {
                self.grade_job = None;
                self.pending_step = None;
            }
        }
    }

    /// Spawn the cargo grade (process #2) on a background thread so a multi-second
    /// `cargo test` never freezes the UI; the result is delivered over a channel and
    /// polled each frame (mirrors the PTY's thread + repaint pattern).
    fn start_grade(&mut self, step: usize, criterion: SuccessCriterion, ctx: &egui::Context) {
        if self.grade_job.is_some() {
            return; // one grade at a time
        }
        let sandbox = self.root.clone();
        let ctx = ctx.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let result = rusty_host::grade(&sandbox, &criterion).map_err(|e| format!("{e:#}"));
            let _ = tx.send(result);
            ctx.request_repaint();
        });
        self.annotation = None;
        self.grade_error = None;
        self.grade_job = Some(rx);
        self.pending_step = Some(step);
    }

    /// Forward the bytes typed this frame to the shell, intercepting sandbox-escaping
    /// `cd` commands at the moment Enter is pressed.
    fn handle_typed(&mut self, typed: &[u8]) {
        for &b in typed {
            match b {
                b'\r' | b'\n' => {
                    match submit_action(&self.line, &self.cwd, &self.root) {
                        SubmitAction::Refuse => {
                            // Cancel the half-typed command and explain, instead of
                            // letting the shell change directory out of the sandbox.
                            let _ = self.session.write(&[0x03]); // Ctrl-C
                            let msg = format!("\r\n{}\r\n", voice::CD_REFUSED);
                            self.term.feed(msg.as_bytes());
                        }
                        SubmitAction::ChangeDir(path) => {
                            self.cwd = path;
                            let _ = self.session.write(&[b]);
                        }
                        SubmitAction::Forward => {
                            let _ = self.session.write(&[b]);
                        }
                    }
                    self.line.clear();
                }
                0x7f | 0x08 => {
                    self.line.pop();
                    let _ = self.session.write(&[b]);
                }
                0x03 => {
                    // Ctrl-C: forward and abandon the mirrored line.
                    self.line.clear();
                    let _ = self.session.write(&[b]);
                }
                _ => {
                    if b >= 0x20 {
                        self.line.push(b as char);
                    }
                    let _ = self.session.write(&[b]);
                }
            }
        }
    }
}

impl eframe::App for RustyApp {
    // eframe 0.34 hands us the framework's central `Ui`; nest panels with
    // `show_inside` (the `Context`-level `.show()` is deprecated). This keeps the
    // Phase-0 `App::ui` decision intact.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // 1. Drain shell output into the terminal grid.
        while let Some(chunk) = self.session.try_recv() {
            self.term.feed(&chunk);
        }
        // 2. Answer device-status queries (e.g. the ConPTY startup CPR handshake).
        let replies = self.term.take_replies();
        if !replies.is_empty() {
            let _ = self.session.write(&replies);
        }

        // 3. Poll an in-flight grade before laying out the result pane.
        self.poll_grade();
        let checking = self.grade_job.is_some();

        // 4. Lesson pane — lesson 1 prose, its exercises, and the annotation pane, all
        //    in one scroll area. Captures a Check request to grade after the panel.
        let mut action = lesson_view::LessonAction::default();
        egui::Panel::left("lesson_pane")
            .resizable(true)
            .default_size(380.0)
            .show_inside(ui, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if self.app_mode == AppMode::DueReviews {
                            ui.heading("Due Reviews");
                            ui.separator();
                            ui.label("You have concepts due for review before continuing.");
                            ui.add_space(8.0);
                            if let Some(lesson) = &self.lesson {
                                if lesson_view::render_recall(ui, &lesson.recall_prompt, &mut self.recall_state) {
                                    action.recall_passed = true;
                                }
                            }
                        } else {
                            if let Some(lesson) = &self.lesson {
                                action = lesson_view::render(
                                    ui,
                                    lesson,
                                    &self.progress,
                                    &mut self.ex_state,
                                    &mut self.recall_state,
                                    checking,
                                );
                            } else {
                                ui.heading(voice::LESSON_PANE_TITLE);
                                ui.separator();
                                ui.colored_label(egui::Color32::LIGHT_RED, voice::LESSON_LOAD_ERROR);
                                if let Some(err) = &self.load_error {
                                    ui.label(egui::RichText::new(err).small().weak());
                                }
                            }

                            // The annotation pane: the last graded result, or a host error.
                            if checking {
                                ui.separator();
                                ui.label(egui::RichText::new(voice::EXERCISE_CHECKING).weak());
                            }
                            if let Some(ann) = &self.annotation {
                                ui.separator();
                                annotation::render(ui, ann, &self.known_lessons);
                            }
                            if let Some(err) = &self.grade_error {
                                ui.separator();
                                ui.colored_label(egui::Color32::LIGHT_RED, err);
                            }
                        }
                    });
            });

        // 5. Kick off grading for a pressed Check (off the UI thread), targeting its step.
        if let Some((step, criterion)) = action.check {
            // T-602 panic-guard: the single chokepoint where a UI Check turns into a
            // grade spawn. If a non-gradeable step somehow reaches here, fail loudly
            // with the step + variant — directly catching the Sprint-6 mystery bug.
            if let Some(lesson) = &self.lesson {
                enforce_gradeable_step(lesson, step);
            }
            let ctx = ui.ctx().clone();
            self.start_grade(step, criterion, &ctx);
        }

        // 6. Handle a successful recall review.
        if action.recall_passed {
            if let Some(lesson) = &self.lesson {
                let current_lesson_index = self.persistent_state.current_lesson_index;
                let quality = if self.recall_state.attempts <= 1 { 5 } else if self.recall_state.attempts == 2 { 3 } else { 1 };
                for concept in &lesson.concepts {
                    self.persistent_state.update_review(concept.id.clone(), quality, current_lesson_index);
                }
                
                if !self.persistent_state.completed_lessons.contains(&lesson.id) {
                    self.persistent_state.completed_lessons.insert(lesson.id.clone());
                    self.persistent_state.current_lesson_index += 1;
                }
                
                self.persistent_state.save(&state::PersistentState::default_path());
            }
            if self.app_mode == AppMode::DueReviews {
                self.app_mode = AppMode::Lesson;
                self.recall_state = AppRecallState::default();
            }
        }
        // 5b. Type a clicked ▶ run command into the embedded PTY (followed by Enter).
        if let Some(cmd) = action.run {
            let _ = self.session.write(&pty_bytes_for_run(&cmd));
        }

        // 6. Terminal pane (right) and code-editor pane (centre).
        let mut typed: Vec<u8> = Vec::new();
        let mut fit = self.dims;
        egui::Panel::right("terminal_pane")
            .resizable(true)
            .default_size(440.0)
            .show_inside(ui, |ui| {
                ui.label(voice::TERMINAL_PANE_LABEL);
                ui.separator();
                fit = terminal_ui(ui, &self.term.grid, &mut typed);
            });
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.label(voice::EDITOR_PANE_LABEL);
            ui.separator();
            self.editor.ui(ui);
        });

        // 7. Resize the grid + PTY to the space the terminal pane actually got.
        if fit != self.dims {
            self.dims = fit;
            self.term.resize(fit.0, fit.1);
            let _ = self.session.resize(fit.0 as u16, fit.1 as u16);
        }

        // 8. Forward keystrokes (with `cd` interception).
        self.handle_typed(&typed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> PathBuf {
        if cfg!(windows) {
            PathBuf::from(r"C:\sandbox\lessons\spike")
        } else {
            PathBuf::from("/sandbox/lessons/spike")
        }
    }

    #[test]
    fn test_submit_forward_for_noncd() {
        let r = root();
        assert_eq!(
            submit_action("cargo --version", &r, &r),
            SubmitAction::Forward
        );
    }

    #[test]
    fn test_submit_refuse_for_escaping_cd() {
        let r = root();
        assert_eq!(submit_action("cd ..", &r, &r), SubmitAction::Refuse);
    }

    #[test]
    fn test_submit_changedir_for_inside_cd() {
        let r = root();
        match submit_action("cd sub", &r, &r) {
            SubmitAction::ChangeDir(p) => assert!(p.ends_with("sub")),
            other => panic!("expected ChangeDir, got {other:?}"),
        }
    }

    #[test]
    fn test_submit_changedir_for_bare_cd_is_root() {
        let r = root();
        match submit_action("cd", &r, &r) {
            SubmitAction::ChangeDir(p) => assert!(p.ends_with("spike")),
            other => panic!("expected ChangeDir(root), got {other:?}"),
        }
    }

    // C-002: the grade-result mapping that `poll_grade` applies to a channel-delivered
    // outcome. The thread-spawn + egui repaint glue itself is validated in the visual
    // heartbeat (no headless egui event loop to assert against).
    #[test]
    fn test_grade_outcome_ok_builds_annotation() {
        let (ann, err) = grade_outcome(&Ok(Verdict::Pass));
        assert!(ann.is_some());
        assert!(err.is_none());
    }

    #[test]
    fn test_grade_outcome_err_surfaces_message() {
        let (ann, err) = grade_outcome(&Err("cargo not found".to_string()));
        assert!(ann.is_none());
        assert_eq!(err.as_deref(), Some("cargo not found"));
    }

    #[test]
    fn test_apply_grade_pass_completes() {
        let mut p = LessonProgress::new(3);
        p.apply(1, &Verdict::Pass);
        assert!(p.completed[1], "a Pass completes the step");
        assert_eq!(p.attempts[1], 0, "a Pass does not bump attempts");
    }

    #[test]
    fn test_apply_grade_fail_bumps_attempts() {
        let mut p = LessonProgress::new(3);
        p.apply(1, &Verdict::TestsFailed);
        assert!(!p.completed[1], "a non-Pass leaves the step incomplete");
        assert_eq!(p.attempts[1], 1, "a non-Pass bumps the attempt count");
    }

    #[test]
    fn test_all_complete_predicate() {
        let mut p = LessonProgress::new(2);
        assert!(!p.all_complete(), "fresh progress is not complete");
        p.apply(0, &Verdict::Pass);
        assert!(!p.all_complete(), "one of two done is not complete");
        p.apply(1, &Verdict::Pass);
        assert!(p.all_complete(), "all steps done → complete");
    }

    fn lesson_with_exercise(kind_toml: &str) -> rusty_curriculum::Lesson {
        let src = format!(
            r##"
                id = "t"
                title = "T"
                track = "Foundations"
                estimated_minutes = 1
                starter_project = "s"
                solution_project = "sol"
                [[steps]]
                [steps.exercise]
                {kind_toml}
                [recall_prompt]
                kind = "short_answer"
                question = "q"
                expected = "a"
                explanation = "e"
            "##
        );
        rusty_curriculum::parse_lesson(&src).expect("test fixture parses")
    }

    #[test]
    #[should_panic(expected = "step 0 is not gradeable (got Worked)")]
    fn test_enforce_gradeable_step_panics_for_non_gating_worked() {
        let lesson = lesson_with_exercise(
            r#"kind = "worked"
prompt = "p"
code = "c"
annotation = "a""#,
        );
        enforce_gradeable_step(&lesson, 0);
    }

    #[test]
    #[should_panic(expected = "step 0 is not gradeable (got PredictThenRun)")]
    fn test_enforce_gradeable_step_panics_for_predict_then_run() {
        let lesson = lesson_with_exercise(
            r#"kind = "predict_then_run"
code = "println!(\"3\");"
question = "?"
expected_output = "3"
explanation = "e""#,
        );
        enforce_gradeable_step(&lesson, 0);
    }

    /// C-006: a prose-only step (`exercise: None`) is "no-exercise" — must panic with
    /// the kind named.
    #[test]
    #[should_panic(expected = "step 0 is not gradeable (got no-exercise)")]
    fn test_enforce_gradeable_step_panics_for_no_exercise() {
        let src = r##"
            id = "t"
            title = "T"
            track = "Foundations"
            estimated_minutes = 1
            starter_project = "s"
            solution_project = "sol"
            [[steps]]
            [[steps.blocks]]
            kind = "prose"
            text = "intro"
            [recall_prompt]
            kind = "short_answer"
            question = "q"
            expected = "a"
            explanation = "e"
        "##;
        let lesson = rusty_curriculum::parse_lesson(src).expect("parses");
        enforce_gradeable_step(&lesson, 0);
    }

    /// C-006: an out-of-range step also routes through the "no-exercise" branch and
    /// panics — pins the chosen behaviour so "silently return on out-of-range" cannot
    /// regress in.
    #[test]
    #[should_panic(expected = "step 99 is not gradeable (got no-exercise)")]
    fn test_enforce_gradeable_step_panics_for_out_of_range() {
        let lesson = lesson_with_exercise(
            r#"kind = "faded"
prompt = "p"
file_path = "src/main.rs"
check_command = "cargo test"
success_criterion = { kind = "cargo_test_passes" }"#,
        );
        enforce_gradeable_step(&lesson, 99);
    }

    #[test]
    fn test_enforce_gradeable_step_ok_for_faded() {
        let lesson = lesson_with_exercise(
            r#"kind = "faded"
prompt = "p"
file_path = "src/main.rs"
check_command = "cargo test"
success_criterion = { kind = "cargo_test_passes" }"#,
        );
        enforce_gradeable_step(&lesson, 0); // does not panic
    }

    #[test]
    fn test_enforce_gradeable_step_ok_for_open() {
        let lesson = lesson_with_exercise(
            r#"kind = "open"
prompt = "p"
check_command = "cargo run"
success_criterion = { kind = "cargo_run_output_matches", expected = "hi" }"#,
        );
        enforce_gradeable_step(&lesson, 0); // does not panic
    }

    #[test]
    fn test_pty_bytes_for_run_appends_carriage_return() {
        // C-002: the exact byte format the embedded PTY needs for a ▶ run click.
        assert_eq!(pty_bytes_for_run("cargo run"), b"cargo run\r".to_vec());
        assert_eq!(
            pty_bytes_for_run(""),
            b"\r".to_vec(),
            "empty cmd still presses Enter"
        );
    }

    #[test]
    fn test_grade_channel_delivers_and_maps() {
        // The off-thread → channel handoff `start_grade`/`poll_grade` rely on: a Verdict
        // sent from another thread is received and mapped to the right annotation.
        let (tx, rx) = std::sync::mpsc::channel::<Result<Verdict, String>>();
        std::thread::spawn(move || {
            let _ = tx.send(Ok(Verdict::TestsFailed));
        });
        let received = rx.recv().expect("a result arrives over the channel");
        let (ann, _) = grade_outcome(&received);
        assert_eq!(ann.unwrap().headline, rusty_grader::Headline::TestsFailed);
    }
}
