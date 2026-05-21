//! The terminal screen grid: a row-major buffer of [`Cell`]s plus the cursor and the
//! current SGR "pen". All mutation goes through methods so the [`crate::performer`]
//! stays a thin translation layer.
//!
//! The pen (current fg/bg/bold) and cursor live here, on the *persistent* grid, not
//! on the per-frame performer — otherwise styling would reset every frame mid-stream.

use crate::cell::{Cell, DEFAULT_BG, DEFAULT_FG};

/// A fixed-size character grid with a cursor.
pub struct Grid {
    pub rows: usize,
    pub cols: usize,
    cells: Vec<Cell>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pen_fg: egui::Color32,
    pen_bg: egui::Color32,
    pen_bold: bool,
    /// Bytes the terminal owes the PTY (e.g. a CPR reply to a `ESC[6n` query).
    replies: Vec<u8>,
}

impl Grid {
    pub fn new(rows: usize, cols: usize) -> Self {
        let rows = rows.max(1);
        let cols = cols.max(1);
        Self {
            rows,
            cols,
            cells: vec![Cell::default(); rows * cols],
            cursor_row: 0,
            cursor_col: 0,
            pen_fg: DEFAULT_FG,
            pen_bg: DEFAULT_BG,
            pen_bold: false,
            replies: Vec::new(),
        }
    }

    fn idx(&self, row: usize, col: usize) -> usize {
        row * self.cols + col
    }

    /// The cell at (row, col). Panics out of bounds (callers stay in range).
    pub fn cell(&self, row: usize, col: usize) -> &Cell {
        &self.cells[self.idx(row, col)]
    }

    /// Iterate the cells of a row left-to-right.
    pub fn row_cells(&self, row: usize) -> &[Cell] {
        let start = self.idx(row, 0);
        &self.cells[start..start + self.cols]
    }

    /// Resize the grid, preserving the top-left content and clamping the cursor.
    pub fn resize(&mut self, rows: usize, cols: usize) {
        let rows = rows.max(1);
        let cols = cols.max(1);
        if rows == self.rows && cols == self.cols {
            return;
        }
        let mut next = vec![Cell::default(); rows * cols];
        for r in 0..rows.min(self.rows) {
            for c in 0..cols.min(self.cols) {
                next[r * cols + c] = self.cells[self.idx(r, c)];
            }
        }
        self.cells = next;
        self.rows = rows;
        self.cols = cols;
        self.cursor_row = self.cursor_row.min(rows - 1);
        self.cursor_col = self.cursor_col.min(cols - 1);
    }

    /// Write a glyph at the cursor with the current pen, then advance (wrapping).
    pub fn put(&mut self, ch: char) {
        if self.cursor_col >= self.cols {
            self.cursor_col = 0;
            self.line_feed();
        }
        let (r, c) = (self.cursor_row, self.cursor_col);
        let i = self.idx(r, c);
        self.cells[i] = Cell {
            ch,
            fg: self.pen_fg,
            bg: self.pen_bg,
            bold: self.pen_bold,
        };
        self.cursor_col += 1;
    }

    /// Line feed (`\n`): move down a row, scrolling if at the bottom.
    pub fn line_feed(&mut self) {
        if self.cursor_row + 1 >= self.rows {
            self.scroll_up();
        } else {
            self.cursor_row += 1;
        }
    }

    /// Carriage return (`\r`): cursor to column 0.
    pub fn carriage_return(&mut self) {
        self.cursor_col = 0;
    }

    /// Backspace: move left one column (no erase).
    pub fn backspace(&mut self) {
        self.cursor_col = self.cursor_col.saturating_sub(1);
    }

    /// Tab: advance to the next 8-column stop, clamped to the last column.
    pub fn tab(&mut self) {
        let next = ((self.cursor_col / 8) + 1) * 8;
        self.cursor_col = next.min(self.cols - 1);
    }

    /// Move the cursor to an absolute (row, col), clamped to the grid.
    pub fn move_to(&mut self, row: usize, col: usize) {
        self.cursor_row = row.min(self.rows - 1);
        self.cursor_col = col.min(self.cols - 1);
    }

    /// Move the cursor up/down/left/right by `n`, clamped.
    pub fn move_up(&mut self, n: usize) {
        self.cursor_row = self.cursor_row.saturating_sub(n);
    }
    pub fn move_down(&mut self, n: usize) {
        self.cursor_row = (self.cursor_row + n).min(self.rows - 1);
    }
    pub fn move_left(&mut self, n: usize) {
        self.cursor_col = self.cursor_col.saturating_sub(n);
    }
    pub fn move_right(&mut self, n: usize) {
        self.cursor_col = (self.cursor_col + n).min(self.cols - 1);
    }

    /// Erase the display. `mode`: 0 = cursor→end, 1 = start→cursor, 2 = all.
    pub fn erase_display(&mut self, mode: u16) {
        let cursor = self.idx(self.cursor_row, self.cursor_col);
        let (start, end) = match mode {
            0 => (cursor, self.cells.len()),
            1 => (0, (cursor + 1).min(self.cells.len())),
            _ => (0, self.cells.len()),
        };
        for cell in &mut self.cells[start..end] {
            *cell = Cell::default();
        }
    }

    /// Erase within the cursor's line. `mode`: 0 = cursor→eol, 1 = bol→cursor, 2 = line.
    pub fn erase_line(&mut self, mode: u16) {
        let row_start = self.idx(self.cursor_row, 0);
        let (start, end) = match mode {
            0 => (row_start + self.cursor_col, row_start + self.cols),
            1 => (row_start, row_start + self.cursor_col + 1),
            _ => (row_start, row_start + self.cols),
        };
        for cell in &mut self.cells[start..end] {
            *cell = Cell::default();
        }
    }

    /// Apply one SGR (Select Graphic Rendition) parameter to the pen.
    pub fn apply_sgr(&mut self, code: u16) {
        use crate::cell::ansi_color;
        match code {
            0 => {
                self.pen_fg = DEFAULT_FG;
                self.pen_bg = DEFAULT_BG;
                self.pen_bold = false;
            }
            1 => self.pen_bold = true,
            22 => self.pen_bold = false,
            30..=37 => self.pen_fg = ansi_color(code - 30),
            90..=97 => self.pen_fg = ansi_color(code - 90 + 8),
            39 => self.pen_fg = DEFAULT_FG,
            40..=47 => self.pen_bg = ansi_color(code - 40),
            100..=107 => self.pen_bg = ansi_color(code - 100 + 8),
            49 => self.pen_bg = DEFAULT_BG,
            _ => {}
        }
    }

    /// Queue a Cursor Position Report (CPR) reply to a `ESC[6n` device-status query.
    /// 1-based coordinates, as the protocol expects.
    pub fn queue_cpr(&mut self) {
        let reply = format!("\x1b[{};{}R", self.cursor_row + 1, self.cursor_col + 1);
        self.replies.extend_from_slice(reply.as_bytes());
    }

    /// Take any bytes the terminal owes the PTY (CPR replies, etc.).
    pub fn take_replies(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.replies)
    }

    fn scroll_up(&mut self) {
        self.cells.drain(0..self.cols);
        self.cells
            .extend(std::iter::repeat_n(Cell::default(), self.cols));
    }
}
