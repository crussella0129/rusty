//! [`Terminal`] bundles the persistent `vte::Parser` with its [`Grid`] so callers
//! (e.g. `rusty-app`) can feed PTY bytes without depending on `vte` directly. The
//! parser must persist across chunks because escape sequences can span reads.

use crate::grid::Grid;
use crate::performer::Performer;

/// A parser + screen grid pair. Feed it raw PTY bytes; read its `grid` to render.
pub struct Terminal {
    pub grid: Grid,
    parser: vte::Parser,
}

impl Terminal {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            grid: Grid::new(rows, cols),
            parser: vte::Parser::new(),
        }
    }

    /// Advance the parser over a chunk of PTY output, mutating the grid.
    pub fn feed(&mut self, bytes: &[u8]) {
        let mut performer = Performer::new(&mut self.grid);
        self.parser.advance(&mut performer, bytes);
    }

    /// Bytes the terminal owes the PTY (e.g. a CPR reply to an `ESC[6n` query).
    pub fn take_replies(&mut self) -> Vec<u8> {
        self.grid.take_replies()
    }

    /// Resize the screen grid (the caller also resizes the PTY).
    pub fn resize(&mut self, rows: usize, cols: usize) {
        self.grid.resize(rows, cols);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feed_then_replies() {
        let mut term = Terminal::new(4, 8);
        term.feed(b"ok\x1b[6n");
        assert_eq!(term.grid.cell(0, 0).ch, 'o');
        assert_eq!(term.grid.cell(0, 1).ch, 'k');
        // cursor at col 2 → CPR reports 1-based (1,3).
        assert_eq!(term.take_replies(), b"\x1b[1;3R");
    }
}
