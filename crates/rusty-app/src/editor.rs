//! The in-app code editor (prompt §6 Phase 3): a picker over the lesson sandbox's
//! `.rs` files plus a syntax-highlighted, editable buffer that saves back to disk.
//! The egui UI lives here; every filesystem touch goes through `rusty_host`'s
//! sandbox-guarded file I/O (the OS boundary, §11).

use std::path::{Path, PathBuf};

use rusty_host::{list_sandbox_rs_files, read_sandbox_file, write_sandbox_file};

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
}

impl Editor {
    /// Build an editor over `sandbox`: list its `.rs` files and open a sensible first
    /// one (`main.rs`/`lib.rs` if present).
    pub fn new(sandbox: &Path) -> Self {
        let mut ed = Self {
            sandbox: sandbox.to_path_buf(),
            files: Vec::new(),
            selected: None,
            buffer: String::new(),
            dirty: false,
            save_state: SaveState::Idle,
            list_error: None,
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
        self.buffer = contents;
        self.selected = Some(rel);
        self.dirty = false;
        self.save_state = SaveState::Idle;
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
            }
            Err(e) => self.save_state = SaveState::Error(format!("{e:#}")),
        }
    }

    /// Render the editor: file picker, Save control, and the highlighted buffer.
    pub fn ui(&mut self, ui: &mut egui::Ui) {
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

        let mut layouter = |ui: &egui::Ui, buf: &dyn egui::TextBuffer, wrap_width: f32| {
            let theme = egui_extras::syntax_highlighting::CodeTheme::from_style(ui.style());
            let mut job = egui_extras::syntax_highlighting::highlight(
                ui.ctx(),
                ui.style(),
                &theme,
                buf.as_str(),
                "rs",
            );
            job.wrap.max_width = wrap_width;
            // egui 0.34: `Fonts::layout_job` needs `&mut`; `Painter::layout_job` is the
            // `&self` path used elsewhere in Rusty (terminal widget).
            ui.painter().layout_job(job)
        };

        let resp = egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.buffer)
                        .code_editor()
                        .desired_width(f32::INFINITY)
                        .layouter(&mut layouter),
                )
            })
            .inner;
        if resp.changed() {
            self.dirty = true;
            self.save_state = SaveState::Idle;
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
        let mut ed = Editor::new(&dir);
        assert!(ed.selected.is_some(), "the first file is auto-opened");
        // A full headless layout pass exercises the highlight layouter without a GPU.
        headless(|ui| ed.ui(ui));
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
        let mut ed = Editor::new(&dir);
        assert!(ed.selected.is_none());
        headless(|ui| ed.ui(ui));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_editor_save_round_trip() {
        // C-001: the editor's own `save()` wiring — buffer → write_sandbox_file → disk,
        // with the dirty flag cleared and `SaveState::Saved` set.
        let dir = temp_sandbox("save", "src/main.rs", "fn main() {}\n");
        let mut ed = Editor::new(&dir);
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
        let mut ed = Editor::new(&dir);
        ed.open(Path::new("src/other.rs"));
        assert_eq!(ed.selected.as_deref(), Some(Path::new("src/other.rs")));
        assert_eq!(ed.buffer, "pub fn o() {}\n");
        std::fs::remove_dir_all(&dir).ok();
    }
}
