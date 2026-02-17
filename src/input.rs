use crossterm::event::KeyCode;
use crate::screen_buf::ScreenBuf;
use crate::text_buffer::TextBuf;

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
    pub fn handle_input(&mut self, k: KeyCode, screen: &ScreenBuf, text_buf: &mut TextBuf) {
        match k {
            KeyCode::Left => {
                if (self.mode == EInputMode::FreeMove) {
                    if self.cursor_x > 0 {
                        self.cursor_x -= 1;
                    }
                } else {
                    text_buf.change_cursor_horizontal(-1);
                }
            }
            KeyCode::Right => {
                if (self.mode == EInputMode::FreeMove) {
                    let nx = self.cursor_x + 1;
                    if nx < screen.w {
                        self.cursor_x = nx;
                    }
                } else {
                    text_buf.change_cursor_horizontal(1);
                }
            }
            KeyCode::Up => {
                if (self.mode == EInputMode::FreeMove) {
                    if self.cursor_y > 0 {
                        self.cursor_y -= 1;
                    }
                }else{
                    text_buf.change_cursor_vertical(-1);
                }
            }
            KeyCode::Down => {
                if (self.mode == EInputMode::FreeMove) {
                    let ny = self.cursor_y + 1;
                    if ny < screen.h {
                        self.cursor_y = ny;
                    }
                }else{
                    text_buf.change_cursor_vertical(1);
                }
            }

            KeyCode::Enter => {
                if self.mode == EInputMode::FreeMove {
                    self.clicked = Some((self.cursor_x, self.cursor_y));
                } else {
                    text_buf.add_line();
                }
            }

            KeyCode::Backspace => {
                if self.mode == EInputMode::TextEditor {
                    text_buf.remove_char_backspace();
                }
            }

            KeyCode::Delete => {
                if self.mode == EInputMode::TextEditor {
                    text_buf.remove_char_delete();
                }
            }


            KeyCode::Tab => {
                if self.mode == EInputMode::TextEditor {
                    text_buf.add_tab();
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
