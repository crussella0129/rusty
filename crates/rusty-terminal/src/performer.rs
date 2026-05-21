//! The `vte::Perform` implementation that turns a parsed ANSI/VT byte stream into
//! mutations on a [`Grid`]. It is a thin translation layer: all state lives on the
//! grid, so a `Performer` is constructed fresh each time bytes are fed.

use vte::{Params, Perform};

use crate::grid::Grid;

/// Borrows the persistent grid for the duration of one `parser.advance(..)` call.
pub struct Performer<'a> {
    pub grid: &'a mut Grid,
}

impl<'a> Performer<'a> {
    pub fn new(grid: &'a mut Grid) -> Self {
        Self { grid }
    }

    fn apply_sgr(&mut self, params: &Params) {
        if params.iter().next().is_none() {
            self.grid.apply_sgr(0); // bare `ESC[m` == reset
            return;
        }
        for p in params.iter() {
            self.grid.apply_sgr(p.first().copied().unwrap_or(0));
        }
    }
}

/// First parameter as a count (missing or 0 → 1), for cursor-move CSIs.
fn count(params: &Params) -> usize {
    params
        .iter()
        .next()
        .and_then(|s| s.first().copied())
        .unwrap_or(0)
        .max(1) as usize
}

/// Nth parameter, defaulting to `default` when missing (used for absolute positions).
fn nth_or(params: &Params, n: usize, default: u16) -> u16 {
    params
        .iter()
        .nth(n)
        .and_then(|s| s.first().copied())
        .filter(|v| *v != 0)
        .unwrap_or(default)
}

/// First parameter value, or 0 (used for erase-mode CSIs and DSR).
fn first(params: &Params) -> u16 {
    params
        .iter()
        .next()
        .and_then(|s| s.first().copied())
        .unwrap_or(0)
}

impl Perform for Performer<'_> {
    fn print(&mut self, c: char) {
        self.grid.put(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => self.grid.line_feed(),
            b'\r' => self.grid.carriage_return(),
            b'\t' => self.grid.tab(),
            0x08 => self.grid.backspace(),
            _ => {} // bell (0x07) and other C0 controls: ignore for the spike
        }
    }

    fn csi_dispatch(
        &mut self,
        params: &Params,
        _intermediates: &[u8],
        _ignore: bool,
        action: char,
    ) {
        match action {
            'm' => self.apply_sgr(params),
            'H' | 'f' => {
                let row = nth_or(params, 0, 1) as usize;
                let col = nth_or(params, 1, 1) as usize;
                self.grid.move_to(row - 1, col - 1);
            }
            'A' => self.grid.move_up(count(params)),
            'B' => self.grid.move_down(count(params)),
            'C' => self.grid.move_right(count(params)),
            'D' => self.grid.move_left(count(params)),
            'J' => self.grid.erase_display(first(params)),
            'K' => self.grid.erase_line(first(params)),
            'n' => {
                // Device Status Report: 6 = "report cursor position" → reply with CPR.
                if first(params) == 6 {
                    self.grid.queue_cpr();
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::{ansi_color, DEFAULT_FG};

    fn run(bytes: &[u8], rows: usize, cols: usize) -> Grid {
        let mut grid = Grid::new(rows, cols);
        let mut parser = vte::Parser::new();
        {
            let mut perf = Performer::new(&mut grid);
            parser.advance(&mut perf, bytes);
        }
        grid
    }

    #[test]
    fn test_print_advances_cursor() {
        let g = run(b"hi", 2, 8);
        assert_eq!(g.cell(0, 0).ch, 'h');
        assert_eq!(g.cell(0, 1).ch, 'i');
        assert_eq!(g.cursor_col, 2);
    }

    #[test]
    fn test_sgr_color_then_reset() {
        let g = run(b"\x1b[31mR\x1b[0mG", 2, 8);
        assert_eq!(g.cell(0, 0).ch, 'R');
        assert_eq!(g.cell(0, 0).fg, ansi_color(1));
        assert_eq!(g.cell(0, 1).ch, 'G');
        assert_eq!(g.cell(0, 1).fg, DEFAULT_FG);
    }

    #[test]
    fn test_sgr_bold() {
        let g = run(b"\x1b[1mB", 2, 8);
        assert!(g.cell(0, 0).bold);
    }

    #[test]
    fn test_crlf_newline() {
        let g = run(b"a\r\nb", 3, 8);
        assert_eq!(g.cell(0, 0).ch, 'a');
        assert_eq!(g.cell(1, 0).ch, 'b');
    }

    #[test]
    fn test_erase_display() {
        let g = run(b"X\x1b[2J", 2, 8);
        for r in 0..g.rows {
            for c in 0..g.cols {
                assert_eq!(g.cell(r, c).ch, ' ', "cell ({r},{c}) not blank");
            }
        }
    }

    #[test]
    fn test_line_wrap_at_last_col() {
        // cols = 3: a,b,c fill row 0; d wraps to row 1 col 0.
        let g = run(b"abcd", 2, 3);
        assert_eq!(g.cell(0, 0).ch, 'a');
        assert_eq!(g.cell(0, 2).ch, 'c');
        assert_eq!(g.cell(1, 0).ch, 'd');
    }

    #[test]
    fn test_dsr_queues_cpr() {
        // `ESC[6n` at the home position → reply `ESC[1;1R`.
        let mut g = run(b"\x1b[6n", 4, 8);
        assert_eq!(g.take_replies(), b"\x1b[1;1R");
    }

    #[test]
    fn test_cursor_move_absolute() {
        let g = run(b"\x1b[2;3H", 5, 10);
        assert_eq!(g.cursor_row, 1);
        assert_eq!(g.cursor_col, 2);
    }
}
