//! The egui terminal widget: paints a [`Grid`] and forwards keystrokes to a writer.
//!
//! The pixel-painting path is verified by the manual smoke check (egui has no
//! headless render assertion); the testable surfaces — key→bytes mapping and grid
//! sizing — are factored into the pure [`key_to_bytes`] and [`grid_dims`] helpers.

use std::io::Write;

use egui::{Color32, Event, FontId, Key, Modifiers, Sense, Vec2};

use crate::cell::DEFAULT_BG;
use crate::grid::Grid;

/// Translate a non-text key press into the bytes a terminal sends. Returns `None`
/// for plain printable keys — those arrive via [`Event::Text`] and must not be
/// double-emitted here.
pub fn key_to_bytes(key: Key, modifiers: &Modifiers) -> Option<Vec<u8>> {
    if modifiers.ctrl {
        // Control codes: Ctrl-C = ETX (0x03) interrupts, Ctrl-D = EOT (0x04).
        match key {
            Key::C => return Some(vec![0x03]),
            Key::D => return Some(vec![0x04]),
            _ => {}
        }
    }
    let bytes: &[u8] = match key {
        Key::Enter => b"\r",
        Key::Backspace => b"\x7f",
        Key::Tab => b"\t",
        Key::Escape => b"\x1b",
        Key::ArrowUp => b"\x1b[A",
        Key::ArrowDown => b"\x1b[B",
        Key::ArrowRight => b"\x1b[C",
        Key::ArrowLeft => b"\x1b[D",
        _ => return None, // printable keys come through Event::Text
    };
    Some(bytes.to_vec())
}

/// How many (rows, cols) of monospaced cells fit in `avail`, given a cell size.
pub fn grid_dims(avail: Vec2, char_w: f32, row_h: f32) -> (usize, usize) {
    let cols = (avail.x / char_w).floor().max(1.0) as usize;
    let rows = (avail.y / row_h).floor().max(1.0) as usize;
    (rows, cols)
}

pub fn terminal_ui(
    ui: &mut egui::Ui,
    grid: &Grid,
    writer: &mut dyn Write,
    request_focus: bool,
) -> (usize, usize) {
    let font = FontId::monospace(14.0);
    // egui 0.34: `FontsView` metrics methods take `&mut`, so use `fonts_mut`.
    let (char_w, row_h) = ui.fonts_mut(|f| (f.glyph_width(&font, 'M'), f.row_height(&font)));

    let avail = ui.available_size();
    let dims = grid_dims(avail, char_w, row_h);

    let (rect, response) = ui.allocate_exact_size(avail, Sense::click());
    if response.clicked() || request_focus {
        response.request_focus();
    }

    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, DEFAULT_BG);

    if response.has_focus() {
        let border_color = ui.visuals().hyperlink_color;
        painter.rect_stroke(
            rect.shrink(1.0),
            0.0,
            egui::Stroke::new(1.5, border_color),
            egui::StrokeKind::Inside,
        );
    }

    for r in 0..grid.rows {
        let y = rect.min.y + r as f32 * row_h;
        let cells = grid.row_cells(r);

        // Per-cell background (only where it differs from the default).
        for (c, cell) in cells.iter().enumerate() {
            if cell.bg != DEFAULT_BG {
                let x = rect.min.x + c as f32 * char_w;
                painter.rect_filled(
                    egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(char_w, row_h)),
                    0.0,
                    cell.bg,
                );
            }
        }

        // Foreground glyphs, coalescing same-color runs into one layout job.
        let mut job = egui::text::LayoutJob::default();
        for cell in cells {
            job.append(
                &cell.ch.to_string(),
                0.0,
                egui::text::TextFormat {
                    font_id: font.clone(),
                    color: cell.fg,
                    ..Default::default()
                },
            );
        }
        let galley = painter.layout_job(job);
        painter.galley(egui::pos2(rect.min.x, y), galley, Color32::WHITE);
    }

    // Cursor block.
    if grid.cursor_row < grid.rows && grid.cursor_col < grid.cols {
        let x = rect.min.x + grid.cursor_col as f32 * char_w;
        let y = rect.min.y + grid.cursor_row as f32 * row_h;
        painter.rect_filled(
            egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(char_w, row_h)),
            0.0,
            Color32::from_white_alpha(90),
        );
    }

    // Input: only while focused, so the terminal doesn't steal global keystrokes.
    if response.has_focus() {
        let events = ui.input(|i| i.events.clone());
        for ev in events {
            match ev {
                Event::Text(text) => {
                    let _ = writer.write_all(text.as_bytes());
                }
                Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } => {
                    if let Some(bytes) = key_to_bytes(key, &modifiers) {
                        let _ = writer.write_all(&bytes);
                    }
                }
                _ => {}
            }
        }
        let _ = writer.flush();
    }

    dims
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_to_bytes_enter() {
        assert_eq!(
            key_to_bytes(Key::Enter, &Modifiers::NONE),
            Some(b"\r".to_vec())
        );
    }

    #[test]
    fn test_key_to_bytes_backspace() {
        assert_eq!(
            key_to_bytes(Key::Backspace, &Modifiers::NONE),
            Some(b"\x7f".to_vec())
        );
    }

    #[test]
    fn test_key_to_bytes_arrows() {
        assert_eq!(
            key_to_bytes(Key::ArrowUp, &Modifiers::NONE),
            Some(b"\x1b[A".to_vec())
        );
        assert_eq!(
            key_to_bytes(Key::ArrowDown, &Modifiers::NONE),
            Some(b"\x1b[B".to_vec())
        );
        assert_eq!(
            key_to_bytes(Key::ArrowRight, &Modifiers::NONE),
            Some(b"\x1b[C".to_vec())
        );
        assert_eq!(
            key_to_bytes(Key::ArrowLeft, &Modifiers::NONE),
            Some(b"\x1b[D".to_vec())
        );
    }

    #[test]
    fn test_ctrl_c_maps_to_etx() {
        let ctrl = Modifiers {
            ctrl: true,
            ..Modifiers::NONE
        };
        assert_eq!(key_to_bytes(Key::C, &ctrl), Some(vec![0x03]));
    }

    #[test]
    fn test_printable_not_double_emitted() {
        // A plain printable key yields no bytes here — it arrives via Event::Text.
        assert_eq!(key_to_bytes(Key::A, &Modifiers::NONE), None);
    }

    #[test]
    fn test_grid_dims_for_rect() {
        // 10 cols × 5 rows worth of space at 8×16 cells.
        let (rows, cols) = grid_dims(Vec2::new(80.0, 80.0), 8.0, 16.0);
        assert_eq!((rows, cols), (5, 10));
    }
}
