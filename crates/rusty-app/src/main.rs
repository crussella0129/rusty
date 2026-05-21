//! `rusty-app` — the eframe binary for Rusty.
//!
//! Phase 0 skeleton: opens a single native window that renders the title. The two
//! real panes (lesson on the left, workspace/terminal on the right, prompt §2) are
//! built out across Phases 1–5. This binary is meant to become feature-complete
//! after Phase 7 and never change to ship new content — new lessons are data, not
//! code (prompt §12).

mod voice;

use eframe::egui;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        voice::WINDOW_TITLE,
        native_options,
        Box::new(|_cc| Ok(Box::<RustyApp>::default())),
    )
}

/// The root application state. Empty in Phase 0; grows as the lesson, workspace,
/// scheduler, and mascot panes are added.
#[derive(Default)]
struct RustyApp;

impl eframe::App for RustyApp {
    // eframe 0.34: `ui` is the required method — the framework wraps the central
    // panel and hands us its `Ui` directly, so we no longer build a `CentralPanel`.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.heading(voice::WINDOW_TITLE);
    }
}
