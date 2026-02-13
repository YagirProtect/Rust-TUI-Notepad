use std::cmp::PartialEq;
use std::io;
use std::io::Write;
use crossterm::style::Stylize;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Color{
    White,
    Green,
    Yellow,
    Blue,
    Red,
    DarkRed,
}

#[derive(Copy, Clone)]
pub struct Cell {
    pub ch: char,
    pub foreground: Color,
}

impl Default for Cell{
    fn default() -> Self {
        Self{ch: ' ', foreground: Color::White}
    }
}
impl Cell{
    fn set_char(&mut self, ch: char) {
        self.ch = ch;
    }
    fn set_char_colored(&mut self, ch: char, foreground: Color) {
        self.ch = ch;
        self.foreground = foreground;
    }
    
    fn clear(&mut self) {
        self.ch = ' ';
        self.foreground = Color::White;
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
        if let Some(i) = self.idx(x, y) {
            self.cells[i].ch = ch;
            self.cells[i].foreground = foreground;
        }
    }

    pub fn clear(&mut self) {
        for c in &mut self.cells {
            c.ch = ' ';
        }
    }

    pub fn present<W: Write>(&mut self, out: &mut W) -> io::Result<()> {
        // Строим один большой String: h строк по w символов
        let mut s = String::with_capacity(self.w as usize * self.h as usize + self.h as usize);

        for y in 0..self.h {
            for x in 0..self.w {
                let i = y as usize * self.w as usize + x as usize;
                if (self.cells[i].foreground != Color::White){

                    match self.cells[i].foreground {
                        Color::White => {}
                        Color::Green => {
                            s.push_str(self.cells[i].ch.green().to_string().as_str());
                        }
                        Color::Yellow => {
                            s.push_str(self.cells[i].ch.yellow().to_string().as_str());
                        }
                        Color::Blue => {
                            s.push_str(self.cells[i].ch.blue().to_string().as_str());
                        }
                        Color::Red => {
                            s.push_str(self.cells[i].ch.red().to_string().as_str());
                        }
                        Color::DarkRed => {
                            s.push_str(self.cells[i].ch.dark_red().to_string().as_str());
                        }
                    }

                    self.cells[i].foreground = Color::White;
                }else{
                    s.push(self.cells[i].ch);
                }
            }
            if y + 1 < self.h {
                s.push('\n'); // важно: не добавлять \n в конце последней строки
            }
        }

        out.write_all(s.as_bytes())?;
        out.flush()
    }

}
