use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyModifiers};
use crate::screen_buffer::ScreenBuf;
use crate::text_buffer::TextBuf;

#[derive(Default, PartialEq)]
pub enum EInputMode{
    #[default]
    FreeMove,
    TextEditor,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum EKeyCommand {
    Undo,
    Redo,
    SelectAll,
    Copy,
    Cut,
    Paste,
}

#[derive(Default)]
pub struct Input{
    pub cursor_x: u16,
    pub cursor_y: u16,

    pub clicked:Option<(u16, u16)>,

    pub mode: EInputMode,

    pub pending_text: String,
    pub is_shift: bool,
    pub text_cursor_move: Option<((usize, usize), (usize, usize))>,
    pub key_command: Option<EKeyCommand>,
    paste_suppression_remaining: usize,
    paste_suppression_started: Option<Instant>,
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
    pub fn arm_paste_suppression(&mut self, text: &str) {
        self.paste_suppression_remaining = text.chars().count().saturating_add(64);
        self.paste_suppression_started = Some(Instant::now());
    }

    pub fn consume_paste_suppression_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        const PASTE_SUPPRESSION_WINDOW: Duration = Duration::from_millis(1500);

        let Some(started_at) = self.paste_suppression_started else {
            return false;
        };

        if started_at.elapsed() > PASTE_SUPPRESSION_WINDOW || self.paste_suppression_remaining == 0 {
            self.paste_suppression_remaining = 0;
            self.paste_suppression_started = None;
            return false;
        }

        if modifiers.contains(KeyModifiers::CONTROL)
            || modifiers.contains(KeyModifiers::ALT)
            || modifiers.contains(KeyModifiers::SUPER)
        {
            self.paste_suppression_remaining = 0;
            self.paste_suppression_started = None;
            return false;
        }

        let is_paste_text_key = matches!(code, KeyCode::Char(_) | KeyCode::Enter | KeyCode::Tab);
        if !is_paste_text_key {
            self.paste_suppression_remaining = 0;
            self.paste_suppression_started = None;
            return false;
        }

        self.paste_suppression_remaining = self.paste_suppression_remaining.saturating_sub(1);
        if self.paste_suppression_remaining == 0 {
            self.paste_suppression_started = None;
        };
        true
    }

    fn shortcut_char(code: KeyCode) -> Option<char> {
        let KeyCode::Char(c) = code else {
            return None;
        };

        c.to_lowercase().next()
    }

    fn resolve_key_command(code: KeyCode, modifiers: KeyModifiers) -> Option<EKeyCommand> {
        if code == KeyCode::Insert && modifiers.contains(KeyModifiers::SHIFT) {
            return Some(EKeyCommand::Paste);
        }

        if matches!(code, KeyCode::Char('v') | KeyCode::Char('V'))
            && modifiers.contains(KeyModifiers::ALT)
            && !modifiers.contains(KeyModifiers::CONTROL)
        {
            return Some(EKeyCommand::Paste);
        }

        let is_ctrl_shortcut = modifiers.contains(KeyModifiers::CONTROL) && !modifiers.contains(KeyModifiers::ALT);
        if !is_ctrl_shortcut {
            return None;
        }

        match Self::shortcut_char(code) {
            Some('z') | Some('я') if modifiers.contains(KeyModifiers::SHIFT) => Some(EKeyCommand::Redo),
            Some('z') | Some('я') => Some(EKeyCommand::Undo),
            Some('y') | Some('н') => Some(EKeyCommand::Redo),
            Some('a') | Some('ф') => Some(EKeyCommand::SelectAll),
            Some('c') | Some('с') => Some(EKeyCommand::Copy),
            Some('x') | Some('ч') => Some(EKeyCommand::Cut),
            Some('v') | Some('м') => Some(EKeyCommand::Paste),
            _ => None,
        }
    }

    pub fn handle_input(&mut self, k: KeyCode, modifiers: KeyModifiers, screen: &ScreenBuf, text_buf: &mut TextBuf) {
        self.is_shift = modifiers.contains(KeyModifiers::SHIFT);
        self.text_cursor_move = None;
        self.key_command = None;

        if let Some(command) = Self::resolve_key_command(k, modifiers) {
            self.key_command = Some(command);
            return;
        }

        let is_ctrl_shortcut = modifiers.contains(KeyModifiers::CONTROL) && !modifiers.contains(KeyModifiers::ALT);

        match k {
            KeyCode::Left => {
                if (self.mode == EInputMode::FreeMove) {
                    if self.cursor_x > 0 {
                        self.cursor_x -= 1;
                    }
                } else {
                    let from = (text_buf.current_index, text_buf.line_index);
                    text_buf.change_cursor_horizontal(-1);
                    let to = (text_buf.current_index, text_buf.line_index);
                    if from != to {
                        self.text_cursor_move = Some((from, to));
                    }
                }
            }
            KeyCode::Right => {
                if (self.mode == EInputMode::FreeMove) {
                    let nx = self.cursor_x + 1;
                    if nx < screen.w {
                        self.cursor_x = nx;
                    }
                } else {
                    let from = (text_buf.current_index, text_buf.line_index);
                    text_buf.change_cursor_horizontal(1);
                    let to = (text_buf.current_index, text_buf.line_index);
                    if from != to {
                        self.text_cursor_move = Some((from, to));
                    }
                }
            }
            KeyCode::Up => {
                if (self.mode == EInputMode::FreeMove) {
                    if self.cursor_y > 0 {
                        self.cursor_y -= 1;
                    }
                }else{
                    let from = (text_buf.current_index, text_buf.line_index);
                    text_buf.change_cursor_vertical(-1);
                    let to = (text_buf.current_index, text_buf.line_index);
                    if from != to {
                        self.text_cursor_move = Some((from, to));
                    }
                }
            }
            KeyCode::Down => {
                if (self.mode == EInputMode::FreeMove) {
                    let ny = self.cursor_y + 1;
                    if ny < screen.h {
                        self.cursor_y = ny;
                    }
                }else{
                    let from = (text_buf.current_index, text_buf.line_index);
                    text_buf.change_cursor_vertical(1);
                    let to = (text_buf.current_index, text_buf.line_index);
                    if from != to {
                        self.text_cursor_move = Some((from, to));
                    }
                }
            }

            KeyCode::Enter => {
                if self.mode == EInputMode::FreeMove {
                    self.clicked = Some((self.cursor_x, self.cursor_y));
                } else {
                    self.pending_text.push('\n');
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
                    self.pending_text.push_str(&" ".repeat(4));
                }
            }

            KeyCode::Char(c) => {
                if self.mode == EInputMode::TextEditor && !is_ctrl_shortcut {
                    self.pending_text.push(c);
                }
            }

            _ => {}
        }
    }
}
