use std::io;
use std::io::Write;
use crossterm::style::{style, Color as TermColor, Stylize};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Color{
    Black,
    White,
    Green,
    Yellow,
    Blue,
    Red,
    DarkRed,
    Gray,
}

#[derive(Copy, Clone)]
pub struct Cell {
    pub ch: char,
    pub foreground: Color,
    pub background: Color,
}

impl Default for Cell{
    fn default() -> Self {
        Self {
            ch: ' ',
            foreground: Color::White,
            background: Color::Black,
        }
    }
}

impl Cell{
    fn set(&mut self, ch: char, foreground: Color, background: Color) {
        self.ch = ch;
        self.foreground = foreground;
        self.background = background;
    }

    fn clear(&mut self) {
        self.ch = ' ';
        self.foreground = Color::White;
        self.background = Color::Black;
    }
}

impl Color {
    fn to_term_color(self) -> TermColor {
        match self {
            Color::Black => TermColor::Black,
            Color::White => TermColor::White,
            Color::Green => TermColor::Green,
            Color::Yellow => TermColor::Yellow,
            Color::Blue => TermColor::Blue,
            Color::Red => TermColor::Red,
            Color::DarkRed => TermColor::DarkRed,
            Color::Gray => TermColor::DarkGrey,
        }
    }
}

pub struct ScreenBuf { pub w: u16, pub h: u16, cells: Vec<Cell> }

impl ScreenBuf {
    pub fn new(x: u16, y: u16) -> Self {
        let size: usize = (x as u32 * y as u32) as usize;
        Self {
            w: x,
            h: y,
            cells: vec![Cell::default(); size],
        }
    }

    pub fn resize(&mut self, x: u16, y: u16) {
        let size: usize = (x as u32 * y as u32) as usize;
        self.cells.resize_with(size, Cell::default);

        self.w = x;
        self.h = y;
    }

    fn idx(&self, x: u16, y: u16) -> Option<usize> {
        if x >= self.w || y >= self.h { return None; }
        Some(y as usize * self.w as usize + x as usize)
    }

    pub fn set(&mut self, x: u16, y: u16, ch: char, foreground: Color) {
        self.set_with_bg(x, y, ch, foreground, Color::Black);
    }

    pub fn set_with_bg(&mut self, x: u16, y: u16, ch: char, foreground: Color, background: Color) {
        if let Some(i) = self.idx(x, y) {
            self.cells[i].set(ch, foreground, background);
        }
    }

    pub fn clear(&mut self) {
        for c in &mut self.cells {
            c.clear();
        }
    }

    pub fn present<W: Write>(&mut self, out: &mut W) -> io::Result<()> {
        let mut s = String::with_capacity(self.w as usize * self.h as usize + self.h as usize);

        for y in 0..self.h {
            for x in 0..self.w {
                let i = y as usize * self.w as usize + x as usize;
                let cell = self.cells[i];

                s.push_str(
                    style(cell.ch)
                        .with(cell.foreground.to_term_color())
                        .on(cell.background.to_term_color())
                        .to_string()
                        .as_str(),
                );
            }

            if y + 1 < self.h {
                s.push('\n');
            }
        }

        out.write_all(s.as_bytes())?;
        out.flush()
    }
}
