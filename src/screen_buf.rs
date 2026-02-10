use std::io;
use std::io::Write;

#[derive(Copy, Clone)]
pub struct Cell {
    pub ch: char
}

impl Default for Cell{
    fn default() -> Self {
        Self{ch: ' '}
    }
}
impl Cell{
    fn set_char(&mut self, ch: char) {
        self.ch = ch;
    }
    
    fn clear(&mut self) {
        self.ch = ' ';
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

    pub fn set(&mut self, x: u16, y: u16, ch: char) {
        if let Some(i) = self.idx(x, y) {
            self.cells[i].ch = ch;
        }
    }

    pub fn clear(&mut self) {
        for c in &mut self.cells {
            c.ch = ' ';
        }
    }

    pub fn present<W: Write>(&self, out: &mut W) -> io::Result<()> {
        // Строим один большой String: h строк по w символов
        let mut s = String::with_capacity(self.w as usize * self.h as usize + self.h as usize);

        for y in 0..self.h {
            for x in 0..self.w {
                let i = y as usize * self.w as usize + x as usize;
                s.push(self.cells[i].ch);
            }
            if y + 1 < self.h {
                s.push('\n'); // важно: не добавлять \n в конце последней строки
            }
        }

        out.write_all(s.as_bytes())?;
        out.flush()
    }

}
