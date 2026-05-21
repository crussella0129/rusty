//! `rusty-app` — the eframe binary for Rusty.
//!
//! Phase 1: a two-pane window — an (empty) lesson pane on the left and a live,
//! sandboxed embedded terminal on the right. The terminal runs a real shell via
//! `rusty-host`, renders its ANSI output through `rusty-terminal`, forwards
//! keystrokes, answers the ConPTY cursor-position handshake, and refuses `cd`s that
//! would escape the lesson sandbox.

mod voice;

use std::path::PathBuf;

use eframe::egui;
use rusty_host::{default_shell, resolve_cd, CdOutcome, PtySession};
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

/// Create (if needed) and return the spike's sandbox directory under the repo.
fn sandbox_root() -> PathBuf {
    let root = std::env::current_dir()
        .unwrap_or_default()
        .join("workspace")
        .join("lessons")
        .join("spike");
    let _ = std::fs::create_dir_all(&root);
    root
}

struct RustyApp {
    term: Terminal,
    session: PtySession,
    root: PathBuf,
    cwd: PathBuf,
    dims: (usize, usize),
    /// Best-effort mirror of the line currently being typed, for `cd` interception.
    line: String,
}

impl RustyApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let root = sandbox_root();
        let ctx = cc.egui_ctx.clone();
        let session = PtySession::spawn(
            &default_shell(),
            &root,
            INIT_ROWS as u16,
            INIT_COLS as u16,
            move || ctx.request_repaint(),
        )
        .expect("spawn shell");

        Self {
            term: Terminal::new(INIT_ROWS, INIT_COLS),
            session,
            cwd: root.clone(),
            root,
            dims: (INIT_ROWS, INIT_COLS),
            line: String::new(),
        }
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

        // 3. Lesson pane (empty placeholder for now).
        egui::Panel::left("lesson_pane")
            .resizable(true)
            .default_size(320.0)
            .show_inside(ui, |ui| {
                ui.heading(voice::LESSON_PANE_TITLE);
                ui.separator();
                ui.label(voice::LESSON_PANE_PLACEHOLDER);
            });

        // 4. Terminal pane.
        let mut typed: Vec<u8> = Vec::new();
        let mut fit = self.dims;
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.label(voice::TERMINAL_PANE_LABEL);
            ui.separator();
            fit = terminal_ui(ui, &self.term.grid, &mut typed);
        });

        // 5. Resize the grid + PTY to the space the terminal pane actually got.
        if fit != self.dims {
            self.dims = fit;
            self.term.resize(fit.0, fit.1);
            let _ = self.session.resize(fit.0 as u16, fit.1 as u16);
        }

        // 6. Forward keystrokes (with `cd` interception).
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
}
