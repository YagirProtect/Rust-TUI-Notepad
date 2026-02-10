use crossterm::event::KeyCode;
use crate::screen_buf::ScreenBuf;

#[derive(Default)]
pub struct Input{
    pub cursor_x: u16,
    pub cursor_y: u16,
}

impl Input {
    pub fn handle_input(&mut self, k: KeyCode, screen: &ScreenBuf) {
        match k {
            KeyCode::Left => {
                if self.cursor_x > 0 {
                    self.cursor_x -= 1;
                }
            }
            KeyCode::Right => {
                let nx = self.cursor_x + 1;
                if nx < screen.w { 
                    self.cursor_x = nx; 
                }
            }
            KeyCode::Up => {
                if self.cursor_y > 0
                {
                    self.cursor_y -= 1;
                }
            }
            KeyCode::Down => {
                let ny = self.cursor_y + 1;
                if ny < screen.h { 
                    self.cursor_y = ny; 
                }
            }
            _ => {}
        }
    }
}
