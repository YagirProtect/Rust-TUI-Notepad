#[derive(Copy, Clone, Debug)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

impl Rect {
    pub fn new(x: u16, y: u16, w: u16, h: u16) -> Self {
        Rect { x, y, w, h }
    }
    pub fn set_position(&mut self, x: u16, y: u16) {
        self.x = x;
        self.y = y;
    }
    pub fn set_x(&mut self, x: u16) {
        self.x = x;
    }
    
    pub fn set_y(&mut self, y: u16) {
        self.y = y;
    }
    
    pub fn get_position(&self) -> (u16, u16) {
        (self.x, self.y)
    }
    
    
    pub fn set_size(&mut self, w: u16, h: u16) {
        self.w = w;
        self.h = h;
    }
    
    pub fn get_size(&self) -> (u16, u16) {
        (self.w, self.h)
    }
    
    pub fn set_w(&mut self, w: u16) {
        self.w = w;
    }
    
    pub fn set_h(&mut self, h: u16) {
        self.h = h;
    }

    pub fn contains(&self, x: u16, y: u16) -> bool {
        if x >= self.x &&
            y >= self.y &&
            x < self.x + self.w &&
            y < self.y + self.h
        {
            true
        } else {
            false
        }
    }
}