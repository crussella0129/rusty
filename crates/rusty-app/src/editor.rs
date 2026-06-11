//! The in-app code editor (prompt §6 Phase 3): a picker over the lesson sandbox's
//! `.rs` files plus a syntax-highlighted, editable buffer that saves back to disk.
//! The egui UI lives here; every filesystem touch goes through `rusty_host`'s
//! sandbox-guarded file I/O (the OS boundary, §11).

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::Receiver;

use rusty_host::{list_sandbox_rs_files, read_sandbox_file, write_sandbox_file, LspSession};
use lsp_types::{Diagnostic, Hover, CompletionItem};

use crate::voice;

/// Outcome of the most recent save, surfaced beside the Save button.
#[derive(Debug, Clone, PartialEq, Eq)]
enum SaveState {
    Idle,
    Saved,
    Error(String),
}

/// Code-editor state over one lesson sandbox.
pub struct Editor {
    sandbox: PathBuf,
    files: Vec<PathBuf>,
    selected: Option<PathBuf>,
    buffer: String,
    dirty: bool,
    save_state: SaveState,
    list_error: Option<String>,
    lsp_session: Option<Arc<LspSession>>,
    doc_version: i32,
    diagnostics: Vec<Diagnostic>,
    hover_rx: Option<Receiver<Result<Option<Hover>, String>>>,
    hover_info: Option<Hover>,
    completion_rx: Option<Receiver<Result<Vec<CompletionItem>, String>>>,
    completions: Option<Vec<CompletionItem>>,
    completion_sel: usize,
}

fn lsp_pos_to_byte_offset(text: &str, line: u32, character: u32) -> usize {
    let mut current_line = 0;
    let mut current_char = 0;
    for (i, c) in text.char_indices() {
        if current_line == line && current_char == character {
            return i;
        }
        if c == '\n' {
            current_line += 1;
            current_char = 0;
        } else {
            current_char += c.len_utf16() as u32;
        }
    }
    if current_line == line && current_char == character {
        return text.len();
    }
    text.len()
}

fn byte_offset_to_lsp_pos(text: &str, offset: usize) -> (u32, u32) {
    let mut line = 0;
    let mut character = 0;
    for (i, c) in text.char_indices() {
        if i >= offset {
            break;
        }
        if c == '\n' {
            line += 1;
            character = 0;
        } else {
            character += c.len_utf16() as u32;
        }
    }
    (line, character)
}

fn split_layout_sections_by_diagnostic(job: &mut egui::text::LayoutJob, text: &str, diagnostics: &[Diagnostic]) {
    for diag in diagnostics {
        let start = lsp_pos_to_byte_offset(text, diag.range.start.line, diag.range.start.character);
        let end = lsp_pos_to_byte_offset(text, diag.range.end.line, diag.range.end.character);
        if start >= end || end > text.len() {
            continue;
        }

        let is_error = diag.severity == Some(lsp_types::DiagnosticSeverity::ERROR);
        let color = if is_error {
            egui::Color32::LIGHT_RED
        } else {
            egui::Color32::from_rgb(255, 165, 0) // orange for warnings
        };

        let mut new_sections = Vec::new();
        for section in &job.sections {
            let s_start = section.byte_range.start;
            let s_end = section.byte_range.end;
            
            // Before intersection
            if s_start < start && s_start < s_end {
                let mut s = section.clone();
                s.byte_range.end = std::cmp::min(s_end, start);
                new_sections.push(s);
            }
            
            // Intersection
            let i_start = std::cmp::max(s_start, start);
            let i_end = std::cmp::min(s_end, end);
            if i_start < i_end {
                let mut s = section.clone();
                s.byte_range.start = i_start;
                s.byte_range.end = i_end;
                s.format.underline = egui::Stroke::new(1.0, color);
                new_sections.push(s);
            }
            
            // After intersection
            if s_end > end && s_start < s_end {
                let mut s = section.clone();
                s.byte_range.start = std::cmp::max(s_start, end);
                new_sections.push(s);
            }
        }
        job.sections = new_sections;
    }
}

impl Editor {
    /// Build an editor over `sandbox`: list its `.rs` files and open a sensible first
    /// one (`main.rs`/`lib.rs` if present).
    pub fn new(sandbox: &Path, lsp_session: Option<Arc<LspSession>>) -> Self {
        let mut ed = Self {
            sandbox: sandbox.to_path_buf(),
            files: Vec::new(),
            selected: None,
            buffer: String::new(),
            dirty: false,
            save_state: SaveState::Idle,
            list_error: None,
            lsp_session,
            doc_version: 1,
            diagnostics: Vec::new(),
            hover_rx: None,
            hover_info: None,
            completion_rx: None,
            completions: None,
            completion_sel: 0,
        };
        ed.refresh_files();
        if let Some(first) = ed.preferred_file() {
            ed.open(&first);
        }
        ed
    }

    fn refresh_files(&mut self) {
        match list_sandbox_rs_files(&self.sandbox) {
            Ok(files) => {
                self.files = files;
                self.list_error = None;
            }
            Err(e) => {
                self.files.clear();
                self.list_error = Some(format!("{e:#}"));
            }
        }
    }

    fn preferred_file(&self) -> Option<PathBuf> {
        self.files
            .iter()
            .find(|p| p.ends_with("main.rs"))
            .or_else(|| self.files.iter().find(|p| p.ends_with("lib.rs")))
            .or_else(|| self.files.first())
            .cloned()
    }

    /// Pure state transition: adopt `contents` for `rel` as a clean (unmodified) buffer.
    /// Separated from [`Self::open`] so it is unit-testable without the filesystem.
    fn load_contents(&mut self, rel: PathBuf, contents: String) {
        self.buffer = contents.clone();
        self.selected = Some(rel.clone());
        self.dirty = false;
        self.save_state = SaveState::Idle;
        self.doc_version = 1;
        self.diagnostics.clear();
        self.hover_rx = None;
        self.hover_info = None;
        self.completion_rx = None;
        self.completions = None;
        
        if let Some(lsp) = &self.lsp_session {
            let _ = lsp.did_open(&self.sandbox.join(&rel), &contents);
        }
    }

    /// Load `rel` from the sandbox into the buffer (discarding unsaved edits).
    pub fn open(&mut self, rel: &Path) {
        match read_sandbox_file(&self.sandbox, rel) {
            Ok(contents) => self.load_contents(rel.to_path_buf(), contents),
            Err(e) => self.save_state = SaveState::Error(format!("{e:#}")),
        }
    }

    /// Persist the buffer to the selected file via the host's guarded writer.
    pub fn save(&mut self) {
        let Some(rel) = self.selected.clone() else {
            return;
        };
        match write_sandbox_file(&self.sandbox, &rel, &self.buffer) {
            Ok(()) => {
                self.dirty = false;
                self.save_state = SaveState::Saved;
                if let Some(lsp) = &self.lsp_session {
                    let _ = lsp.did_save(&self.sandbox.join(&rel));
                }
            }
            Err(e) => self.save_state = SaveState::Error(format!("{e:#}")),
        }
    }

    /// Render the editor: file picker, Save control, and the highlighted buffer.
    pub fn ui(&mut self, ui: &mut egui::Ui, focus_request: Option<crate::FocusTarget>) {
        if let Some(lsp) = &self.lsp_session {
            while let Some(diags) = lsp.poll_diagnostics() {
                self.diagnostics = diags.diagnostics;
            }
            
            if let Some(rx) = &self.hover_rx {
                if let Ok(Ok(Some(hover))) = rx.try_recv() {
                    self.hover_info = Some(hover);
                }
            }
            
            if let Some(rx) = &self.completion_rx {
                if let Ok(Ok(items)) = rx.try_recv() {
                    self.completions = Some(items);
                    self.completion_sel = 0;
                }
            }
        }

        ui.horizontal_wrapped(|ui| {
            ui.label(voice::EDITOR_FILES_LABEL);
            // Clone the list so the click handler can borrow `self` mutably.
            for rel in self.files.clone() {
                let is_sel = self.selected.as_deref() == Some(rel.as_path());
                if ui.selectable_label(is_sel, rel.to_string_lossy()).clicked() {
                    self.open(&rel);
                }
            }
        });

        if self.files.is_empty() {
            ui.colored_label(
                egui::Color32::from_rgb(0xff, 0xb3, 0x00),
                self.list_error
                    .clone()
                    .unwrap_or_else(|| voice::EDITOR_NO_FILES.to_string()),
            );
            return;
        }

        ui.horizontal(|ui| {
            if ui.button(voice::EDITOR_SAVE).clicked() {
                self.save();
            }
            match &self.save_state {
                SaveState::Idle => {}
                SaveState::Saved => {
                    ui.colored_label(
                        egui::Color32::from_rgb(0x4c, 0xaf, 0x50),
                        voice::EDITOR_SAVED,
                    );
                }
                SaveState::Error(e) => {
                    ui.colored_label(egui::Color32::LIGHT_RED, e.clone());
                }
            }
        });

        let diagnostics = self.diagnostics.clone();
        let buffer_text = self.buffer.clone();

        let mut layouter = |ui: &egui::Ui, buf: &dyn egui::TextBuffer, wrap_width: f32| {
            let theme = egui_extras::syntax_highlighting::CodeTheme::from_style(ui.style());
            let mut job = egui_extras::syntax_highlighting::highlight(
                ui.ctx(),
                ui.style(),
                &theme,
                buf.as_str(),
                "rs",
            );
            
            split_layout_sections_by_diagnostic(&mut job, buf.as_str(), &diagnostics);
            
            job.wrap.max_width = wrap_width;
            ui.painter().layout_job(job)
        };

        // Capture keyboard events if completion popup is open
        let mut close_completions = false;
        let mut apply_completion = None;
        if let Some(items) = &self.completions {
            if !items.is_empty() {
                ui.input_mut(|i| {
                    if i.consume_key(egui::Modifiers::NONE, egui::Key::Escape) {
                        close_completions = true;
                    }
                    if i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown) {
                        self.completion_sel = (self.completion_sel + 1).min(items.len() - 1);
                    }
                    if i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp) {
                        self.completion_sel = self.completion_sel.saturating_sub(1);
                    }
                    if i.consume_key(egui::Modifiers::NONE, egui::Key::Enter) || i.consume_key(egui::Modifiers::NONE, egui::Key::Tab) {
                        apply_completion = Some(items[self.completion_sel].clone());
                    }
                });
            } else {
                close_completions = true;
            }
        }

        let text_edit = egui::TextEdit::multiline(&mut self.buffer)
            .code_editor()
            .desired_width(f32::INFINITY)
            .id_source("editor_text_edit")
            .layouter(&mut layouter);

        let output = egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| text_edit.show(ui))
            .inner;
        
        let resp = output.response;

        if focus_request == Some(crate::FocusTarget::Editor) {
            resp.request_focus();
        }

        if let Some(comp) = apply_completion {
            let insert_text = comp.insert_text.as_deref().unwrap_or(&comp.label);
            if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), resp.id) {
                if let Some(ccur) = state.cursor.char_range() {
                    let mut start = std::cmp::min(ccur.primary.index, self.buffer.len());
                    // Find trigger word start
                    while start > 0 {
                        let c = self.buffer[..start].chars().last().unwrap();
                        if !c.is_alphanumeric() && c != '_' { break; }
                        start -= c.len_utf8();
                    }
                    let end = ccur.primary.index;
                    self.buffer.replace_range(start..end, insert_text);
                    let new_pos = start + insert_text.len();
                    state.cursor.set_char_range(Some(egui::text::CCursorRange::one(egui::text::CCursor::new(new_pos))));
                    egui::TextEdit::store_state(ui.ctx(), resp.id, state);
                    self.dirty = true;
                }
            }
            close_completions = true;
        }

        if close_completions {
            self.completions = None;
            self.completion_rx = None;
        }

        if resp.changed() {
            self.dirty = true;
            self.save_state = SaveState::Idle;
            self.doc_version += 1;
            
            // Check for completion trigger
            if let (Some(lsp), Some(rel)) = (&self.lsp_session, &self.selected) {
                let full_path = self.sandbox.join(rel);
                let _ = lsp.did_change(&full_path, self.doc_version, &self.buffer);

                if let Some(state) = egui::TextEdit::load_state(ui.ctx(), resp.id) {
                    if let Some(ccur) = state.cursor.char_range() {
                        let pos = ccur.primary.index;
                        if pos > 0 {
                            let last_char = self.buffer[..pos].chars().last().unwrap();
                            if last_char == '.' || last_char == ':' {
                                let (line, character) = byte_offset_to_lsp_pos(&self.buffer, pos);
                                if let Ok(rx) = lsp.completion(&full_path, line, character) {
                                    self.completion_rx = Some(rx);
                                }
                            } else {
                                self.completions = None;
                                self.completion_rx = None;
                            }
                        }
                    }
                }
            }
        }

        // Paint gutter markers for diagnostics
        let galley = &output.galley;
        for diag in &self.diagnostics {
            let offset = lsp_pos_to_byte_offset(&buffer_text, diag.range.start.line, diag.range.start.character);
            let cursor = egui::text::CCursor::new(offset);
            let pos = galley.pos_from_cursor(cursor);
            
            let is_error = diag.severity == Some(lsp_types::DiagnosticSeverity::ERROR);
            let color = if is_error {
                egui::Color32::LIGHT_RED
            } else {
                egui::Color32::from_rgb(255, 165, 0)
            };
            
            let screen_pos = resp.rect.min + pos.min.to_vec2();
            ui.painter().circle_filled(
                screen_pos + egui::vec2(-8.0, 6.0),
                4.0,
                color
            );
        }
            
            // Hover tooltip
            if resp.hovered() {
                if let Some(pointer) = ui.ctx().pointer_hover_pos() {
                    let local_pos = pointer - resp.rect.min;
                    let cursor = galley.cursor_from_pos(local_pos);
                    let offset = cursor.index;
                    let (line, character) = byte_offset_to_lsp_pos(&buffer_text, offset);
                    
                    if let (Some(lsp), Some(rel)) = (&self.lsp_session, &self.selected) {
                        if self.hover_rx.is_none() {
                            if let Ok(rx) = lsp.hover(&self.sandbox.join(rel), line, character) {
                                self.hover_rx = Some(rx);
                            }
                        }
                    }

                    if let Some(hover) = &self.hover_info {
                        let text = match &hover.contents {
                            lsp_types::HoverContents::Scalar(marked) => match marked {
                                lsp_types::MarkedString::String(s) => s.clone(),
                                lsp_types::MarkedString::LanguageString(ls) => ls.value.clone(),
                            },
                            lsp_types::HoverContents::Array(arr) => arr.iter().map(|m| match m {
                                lsp_types::MarkedString::String(s) => s.clone(),
                                lsp_types::MarkedString::LanguageString(ls) => ls.value.clone(),
                            }).collect::<Vec<_>>().join("\n"),
                            lsp_types::HoverContents::Markup(markup) => markup.value.clone(),
                        };
                        #[allow(deprecated)]
                        egui::show_tooltip_text(ui.ctx(), ui.layer_id(), egui::Id::new("hover"), text);
                    }
                }
            } else {
                self.hover_info = None;
                self.hover_rx = None;
            }

            // Render completions popup
            if let Some(items) = &self.completions {
                if !items.is_empty() {
                    if let Some(state) = egui::TextEdit::load_state(ui.ctx(), resp.id) {
                        if let Some(ccur) = state.cursor.char_range() {
                            let pos = galley.pos_from_cursor(ccur.primary);
                            let screen_pos = resp.rect.min + pos.min.to_vec2() + egui::vec2(0.0, 15.0);
                            
                            egui::Area::new(egui::Id::new("completions_popup"))
                                .fixed_pos(screen_pos)
                                .order(egui::Order::Tooltip)
                                .show(ui.ctx(), |ui| {
                                    egui::Frame::menu(ui.style()).show(ui, |ui| {
                                        ui.set_max_width(300.0);
                                        for (i, item) in items.iter().enumerate() {
                                            let is_selected = i == self.completion_sel;
                                            let response = ui.selectable_label(is_selected, &item.label);
                                            if response.clicked() {
                                                // apply completion via click is harder, ignore for now as keyboard works
                                            }
                                        }
                                    });
                                });
                        }
                    }
                }
            }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A bare editor with no sandbox interaction, for pure state tests.
    fn bare() -> Editor {
        Editor {
            sandbox: PathBuf::new(),
            files: Vec::new(),
            selected: None,
            buffer: String::new(),
            dirty: false,
            save_state: SaveState::Idle,
            list_error: None,
            lsp_session: None,
            doc_version: 1,
            diagnostics: Vec::new(),
            hover_rx: None,
            hover_info: None,
            completion_rx: None,
            completions: None,
            completion_sel: 0,
        }
    }

    #[test]
    fn test_editor_state_load() {
        let mut ed = bare();
        ed.dirty = true;
        ed.load_contents(PathBuf::from("src/main.rs"), "fn main() {}".to_string());
        assert_eq!(ed.buffer, "fn main() {}");
        assert!(!ed.dirty, "a freshly loaded buffer is clean");
        assert_eq!(ed.selected.as_deref(), Some(Path::new("src/main.rs")));
    }

    /// Build a temp sandbox with a `.rs` file so the render tests have something real.
    fn temp_sandbox(tag: &str, file: &str, contents: &str) -> PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("rusty-editor-{tag}-{nanos}"));
        let path = dir.join(file);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, contents).unwrap();
        dir
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
    fn test_editor_renders_loaded_buffer() {
        let dir = temp_sandbox(
            "render",
            "src/main.rs",
            "fn main() {\n    println!(\"hi\");\n}\n",
        );
        let mut ed = Editor::new(&dir, None);
        assert!(ed.selected.is_some(), "the first file is auto-opened");
        // A full headless layout pass exercises the highlight layouter without a GPU.
        headless(|ui| ed.ui(ui, None));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_editor_renders_empty_sandbox() {
        // An empty sandbox: the editor shows the "no files" notice without panicking.
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("rusty-editor-empty-{nanos}"));
        std::fs::create_dir_all(&dir).unwrap();
        let mut ed = Editor::new(&dir, None);
        assert!(ed.selected.is_none());
        headless(|ui| ed.ui(ui, None));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_editor_save_round_trip() {
        // C-001: the editor's own `save()` wiring — buffer → write_sandbox_file → disk,
        // with the dirty flag cleared and `SaveState::Saved` set.
        let dir = temp_sandbox("save", "src/main.rs", "fn main() {}\n");
        let mut ed = Editor::new(&dir, None);
        ed.buffer = "fn main() { println!(\"x\"); }\n".to_string();
        ed.dirty = true;
        ed.save();
        assert_eq!(ed.save_state, SaveState::Saved);
        assert!(!ed.dirty, "a successful save clears the dirty flag");
        let on_disk = std::fs::read_to_string(dir.join("src").join("main.rs")).unwrap();
        assert_eq!(on_disk, "fn main() { println!(\"x\"); }\n");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_editor_save_noop_without_selection() {
        // C-001 negative path: Save with nothing open must not panic and stays Idle.
        let mut ed = bare();
        ed.save();
        assert_eq!(ed.save_state, SaveState::Idle);
    }

    #[test]
    fn test_editor_open_switches_file() {
        // C-003: selecting a *different* file loads its bytes (not just the auto-opened one).
        let dir = temp_sandbox("switch", "src/main.rs", "fn main() {}\n");
        std::fs::write(dir.join("src").join("other.rs"), "pub fn o() {}\n").unwrap();
        let mut ed = Editor::new(&dir, None);
        ed.open(Path::new("src/other.rs"));
        assert_eq!(ed.selected.as_deref(), Some(Path::new("src/other.rs")));
        assert_eq!(ed.buffer, "pub fn o() {}\n");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_lsp_pos_to_byte_offset() {
        let text = "fn main() {\n    let x = 1;\n}\n";
        assert_eq!(lsp_pos_to_byte_offset(text, 0, 0), 0);
        assert_eq!(lsp_pos_to_byte_offset(text, 1, 4), 16); // 'l' in let
    }

    #[test]
    fn test_byte_offset_to_lsp_pos() {
        let text = "fn main() {\n    let x = 1;\n}\n";
        assert_eq!(byte_offset_to_lsp_pos(text, 0), (0, 0));
        assert_eq!(byte_offset_to_lsp_pos(text, 16), (1, 4));
    }

    #[test]
    fn test_split_layout_sections_by_diagnostic() {
        let mut job = egui::text::LayoutJob::default();
        job.append("hello world", 0.0, egui::text::TextFormat::default());
        let range = lsp_types::Range {
            start: lsp_types::Position { line: 0, character: 6 },
            end: lsp_types::Position { line: 0, character: 11 },
        };
        let diag = lsp_types::Diagnostic {
            range,
            severity: Some(lsp_types::DiagnosticSeverity::ERROR),
            code: None,
            code_description: None,
            source: None,
            message: "error".to_string(),
            related_information: None,
            tags: None,
            data: None,
        };
        split_layout_sections_by_diagnostic(&mut job, "hello world", &[diag]);
        assert_eq!(job.sections.len(), 2);
        assert_eq!(job.sections[0].byte_range, 0..6);
        assert_eq!(job.sections[1].byte_range, 6..11);
        assert_ne!(job.sections[1].format.underline, egui::Stroke::NONE);
    }
}
