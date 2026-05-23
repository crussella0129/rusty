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
mod theme;
mod voice;

use std::io::Write as _;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, TryRecvError};

use editor::Editor;
use eframe::egui;
use exercise_view::ExerciseState;
use rusty_curriculum::{Lesson, SuccessCriterion};
use rusty_grader::{annotate, Annotation, Verdict};
use rusty_host::{default_shell, load_lesson, prepare_sandbox, resolve_cd, CdOutcome, PtySession};
use rusty_terminal::{terminal_ui, Terminal};

const INIT_ROWS: usize = 24;
const INIT_COLS: usize = 80;

/// Path of the cross-run trace log (`$TEMP/rusty-trace.log`). Used by the Sprint-6
/// diagnostic instrumentation — Windows GUI-subsystem stderr buffering can swallow
/// `eprintln!` output before a panic-exit, so traces ALSO go to this file.
pub(crate) fn trace_log_path() -> PathBuf {
    std::env::temp_dir().join("rusty-trace.log")
}

/// Append a diagnostic line to both stderr and the trace log. The log write is
/// flushed/synced so a panic immediately after still leaves the line on disk.
pub(crate) fn trace(msg: &str) {
    let line = format!("[rusty-trace] {msg}\n");
    let _ = std::io::stderr().write_all(line.as_bytes());
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(trace_log_path())
    {
        let _ = f.write_all(line.as_bytes());
        let _ = f.sync_all();
    }
}

fn main() -> eframe::Result {
    // T-602 diagnostics: panics in `App::ui` (e.g. enforce_gradeable_step) must leave a
    // record even when stderr buffers can't flush before the GUI-subsystem hard-exits.
    // Truncate any prior log and install a chained panic hook that writes the panic
    // info to the trace log before the default hook unwinds.
    let _ = std::fs::remove_file(trace_log_path());
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        trace(&format!("PANIC: {info}"));
        default_hook(info);
    }));
    trace(&format!("startup log={}", trace_log_path().display()));

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

/// Fallback sandbox dir if a lesson fails to load.
fn fallback_sandbox() -> PathBuf {
    let root = std::env::current_dir()
        .unwrap_or_default()
        .join("workspace")
        .join("lessons")
        .join("spike");
    let _ = std::fs::create_dir_all(&root);
    root
}

/// The lesson shipped this sprint (multi-lesson selection is a later phase).
const LESSON_REL: &str = "content/lessons/foundations-01-hello";

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
}

impl RustyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let cwd0 = std::env::current_dir().unwrap_or_default();
        let content_dir = cwd0.join(LESSON_REL);
        let workspace_root = cwd0.join("workspace");

        // Load lesson 1, prepare its sandbox, and pick the PTY's working directory.
        // No `?` here — `new` is infallible; failures surface in the lesson pane.
        let (lesson, load_error, sandbox) = match load_lesson(&content_dir) {
            Ok(lesson) => match prepare_sandbox(&content_dir, &workspace_root, &lesson.id.0) {
                Ok(sandbox) => (Some(lesson), None, sandbox),
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

        let editor = Editor::new(&sandbox);
        let known_lessons = lesson
            .as_ref()
            .map(|l| vec![l.id.0.clone()])
            .unwrap_or_default();
        let progress = LessonProgress::new(lesson.as_ref().map(|l| l.steps.len()).unwrap_or(0));

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
            known_lessons,
            ex_state: ExerciseState::default(),
            grade_job: None,
            annotation: None,
            grade_error: None,
            progress,
            pending_step: None,
        }
    }

    /// Poll the background grade thread; when it finishes, build the annotation (or
    /// surface a host error) and clear the in-flight job.
    fn poll_grade(&mut self) {
        let Some(rx) = &self.grade_job else { return };
        match rx.try_recv() {
            Ok(received) => {
                trace(&format!(
                    "poll_grade.ok pending_step={:?} verdict={received:?}",
                    self.pending_step
                ));
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
        trace(&format!("start_grade step={step} criterion={criterion:?}"));
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
                        if let Some(lesson) = &self.lesson {
                            action = lesson_view::render(
                                ui,
                                lesson,
                                &self.progress,
                                &mut self.ex_state,
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
        // 5b. Type a clicked ▶ run command into the embedded PTY (followed by Enter).
        if let Some(cmd) = action.run {
            let bytes = format!("{cmd}\r").into_bytes();
            let _ = self.session.write(&bytes);
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
    #[should_panic(expected = "not gradeable")]
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
    #[should_panic(expected = "not gradeable")]
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
