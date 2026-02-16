use crossterm::event::KeyCode;
use crate::screen_buf::ScreenBuf;


#[derive(Default, PartialEq)]
pub enum EInputMode{
    #[default]
    FreeMove,
    TextEditor,
}

#[derive(Default)]
pub struct Input{
    pub cursor_x: u16,
    pub cursor_y: u16,


    pub clicked:Option<(u16, u16)>,

    pub mode: EInputMode,

    pub last_character: Option<char>,
}

impl Input {
    pub fn clamp_cursor(&mut self, min_x: u16, max_x: u16, min_y: u16, max_y: u16) {
        if (self.cursor_x < min_x){
            self.cursor_x = min_x;
        }
        if (self.cursor_x > max_x){
            self.cursor_x = max_x;
        }
        if (self.cursor_y < min_y){
            self.cursor_y = min_y;
        }
        if (self.cursor_y > max_y){
            self.cursor_y = max_y;
        }
    }
}

impl Input {
    pub fn change_mode(&mut self, mode: EInputMode) {
        self.mode = mode;
    }
}

impl Input {
    pub fn handle_input(&mut self, k: KeyCode, screen: &ScreenBuf) {
        match k {
            KeyCode::Left => { /* ... */ }
            KeyCode::Right => { /* ... */ }
            KeyCode::Up => { /* ... */ }
            KeyCode::Down => { /* ... */ }

            KeyCode::Enter => {
                if self.mode == EInputMode::FreeMove {
                    self.clicked = Some((self.cursor_x, self.cursor_y));
                } else {
                    self.last_character = Some('\n');
                }
            }

            KeyCode::Backspace => {
                if self.mode == EInputMode::TextEditor {
                    self.last_character = Some('\x08');
                }
            }

            KeyCode::Tab => {
                if self.mode == EInputMode::TextEditor {
                    self.last_character = Some('\t');
                }
            }

            KeyCode::Char(c) => {
                if self.mode == EInputMode::TextEditor {
                    self.last_character = Some(c);
                }
            }

            _ => {}
        }
    }
}
