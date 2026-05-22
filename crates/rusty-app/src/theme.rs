//! Rusty's shared typographic scale. One place defines the heading sizes used across
//! the lesson prose, callouts, exercises, and the annotation pane, so the whole app
//! shares a consistent hierarchy (title ≫ H1 ≫ H2 ≫ H3 ≫ body) — the size variance is
//! what gives the panes visual "breakup" and flow. If the scale changes, it changes here.

/// The lesson name at the very top of the lesson pane — the dominant heading.
pub const TITLE: f32 = 28.0;
/// In-body markdown heading sizes (H1 ≥ H2 ≥ H3).
pub const H1: f32 = 24.0;
pub const H2: f32 = 19.0;
/// H3 is also the size for section *labels* — a callout's "Tip", an exercise's
/// "Worked example", the annotation headline — so they read as sub-headings, not body.
pub const H3: f32 = 16.0;
/// Body prose / list text.
pub const BODY: f32 = 14.0;

/// The size for a markdown heading of `level` (1 → H1, 2 → H2, 3+ → H3).
pub fn heading_size(level: u8) -> f32 {
    match level {
        1 => H1,
        2 => H2,
        _ => H3,
    }
}

/// A strong, H3-sized section label (callout tones, exercise/annotation headers).
pub fn section_label(text: impl Into<String>) -> egui::RichText {
    egui::RichText::new(text).size(H3).strong()
}
