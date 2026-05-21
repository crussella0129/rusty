//! A single terminal cell and the 16-color ANSI palette.

use egui::Color32;

/// One character cell in the terminal grid.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Cell {
    pub ch: char,
    pub fg: Color32,
    pub bg: Color32,
    pub bold: bool,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: DEFAULT_FG,
            bg: DEFAULT_BG,
            bold: false,
        }
    }
}

/// Default foreground (light gray) and background (near-black) for unstyled text.
pub const DEFAULT_FG: Color32 = Color32::from_rgb(0xcc, 0xcc, 0xcc);
pub const DEFAULT_BG: Color32 = Color32::from_rgb(0x0c, 0x0c, 0x0c);

/// The 16-color ANSI palette (xterm-ish). `code` is 0..=15; out-of-range → default fg.
pub fn ansi_color(code: u16) -> Color32 {
    match code {
        0 => Color32::from_rgb(0x00, 0x00, 0x00),  // black
        1 => Color32::from_rgb(0xcd, 0x00, 0x00),  // red
        2 => Color32::from_rgb(0x00, 0xcd, 0x00),  // green
        3 => Color32::from_rgb(0xcd, 0xcd, 0x00),  // yellow
        4 => Color32::from_rgb(0x00, 0x00, 0xee),  // blue
        5 => Color32::from_rgb(0xcd, 0x00, 0xcd),  // magenta
        6 => Color32::from_rgb(0x00, 0xcd, 0xcd),  // cyan
        7 => Color32::from_rgb(0xe5, 0xe5, 0xe5),  // white
        8 => Color32::from_rgb(0x7f, 0x7f, 0x7f),  // bright black (gray)
        9 => Color32::from_rgb(0xff, 0x00, 0x00),  // bright red
        10 => Color32::from_rgb(0x00, 0xff, 0x00), // bright green
        11 => Color32::from_rgb(0xff, 0xff, 0x00), // bright yellow
        12 => Color32::from_rgb(0x5c, 0x5c, 0xff), // bright blue
        13 => Color32::from_rgb(0xff, 0x00, 0xff), // bright magenta
        14 => Color32::from_rgb(0x00, 0xff, 0xff), // bright cyan
        15 => Color32::from_rgb(0xff, 0xff, 0xff), // bright white
        _ => DEFAULT_FG,
    }
}
