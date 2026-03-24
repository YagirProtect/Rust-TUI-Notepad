use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyModifiers};
use crate::screen_buffer::ScreenBuf;
use crate::shortcuts::ShortcutMap;
use crate::text_buffer::TextBuf;

#[derive(Copy, Clone, Default, PartialEq)]
pub enum EInputMode{
    #[default]
    FreeMove,
    TextEditor,
    SearchQueryEditor,
    SearchReplaceEditor,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum EKeyCommand {
    Find,
    Replace,
    FindNext,
    NewFile,
    OpenFile,
    OpenInExplorer,
    SaveFile,
    SaveFileAs,
    Undo,
    Redo,
    SelectAll,
    Copy,
    Cut,
    Paste,
}

pub struct Input{
    pub cursor_x: u16,
    pub cursor_y: u16,

    pub clicked:Option<(u16, u16)>,
    pub middle_clicked: Option<(u16, u16)>,
    pub double_clicked: Option<(u16, u16)>,
    pub mouse_down: Option<(u16, u16)>,
    pub mouse_released: Option<(u16, u16)>,
    pub mouse_scroll: Option<(i32, i32)>,

    pub mode: EInputMode,

    pub pending_text: String,
    pub is_shift: bool,
    pub is_ctrl: bool,
    pub is_alt: bool,
    pub text_cursor_move: Option<((usize, usize), (usize, usize))>,
    pub key_command: Option<EKeyCommand>,
    pub text_mouse_anchor: Option<(usize, usize)>,
    paste_suppression_remaining: usize,
    paste_suppression_started: Option<Instant>,
    last_left_click_at: Option<Instant>,
    last_left_click_pos: Option<(u16, u16)>,
    shortcut_map: ShortcutMap,
}

impl Default for Input {
    fn default() -> Self {
        Self {
            cursor_x: 0,
            cursor_y: 0,
            clicked: None,
            middle_clicked: None,
            double_clicked: None,
            mouse_down: None,
            mouse_released: None,
            mouse_scroll: None,
            mode: EInputMode::FreeMove,
            pending_text: String::new(),
            is_shift: false,
            is_ctrl: false,
            is_alt: false,
            text_cursor_move: None,
            key_command: None,
            text_mouse_anchor: None,
            paste_suppression_remaining: 0,
            paste_suppression_started: None,
            last_left_click_at: None,
            last_left_click_pos: None,
            shortcut_map: ShortcutMap::default(),
        }
    }
}

impl Input {
    pub fn new(shortcut_map: ShortcutMap) -> Self {
        Self {
            shortcut_map,
            ..Self::default()
        }
    }

    fn is_text_mode(&self) -> bool {
        matches!(
            self.mode,
            EInputMode::TextEditor | EInputMode::SearchQueryEditor | EInputMode::SearchReplaceEditor
        )
    }

    pub fn is_search_mode(&self) -> bool {
        matches!(self.mode, EInputMode::SearchQueryEditor | EInputMode::SearchReplaceEditor)
    }

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

    pub fn register_left_click(&mut self, x: u16, y: u16) -> bool {
        const DOUBLE_CLICK_WINDOW: Duration = Duration::from_millis(350);
        const DOUBLE_CLICK_MAX_DELTA: u16 = 1;

        let now = Instant::now();
        let is_double = self
            .last_left_click_at
            .zip(self.last_left_click_pos)
            .is_some_and(|(last_at, (last_x, last_y))| {
                now.duration_since(last_at) <= DOUBLE_CLICK_WINDOW
                    && last_x.abs_diff(x) <= DOUBLE_CLICK_MAX_DELTA
                    && last_y.abs_diff(y) <= DOUBLE_CLICK_MAX_DELTA
            });

        self.last_left_click_at = Some(now);
        self.last_left_click_pos = Some((x, y));
        is_double
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
    pub fn handle_input(&mut self, k: KeyCode, modifiers: KeyModifiers, screen: &ScreenBuf, text_buf: &mut TextBuf) {
        self.is_shift = modifiers.contains(KeyModifiers::SHIFT);
        self.is_ctrl = modifiers.contains(KeyModifiers::CONTROL);
        self.is_alt = modifiers.contains(KeyModifiers::ALT);
        self.text_cursor_move = None;
        self.key_command = None;

        if let Some(command) = self.shortcut_map.resolve(k, modifiers) {
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
            KeyCode::PageUp => {
                if self.mode == EInputMode::FreeMove {
                    let page = screen.h.saturating_sub(1).max(1);
                    self.cursor_y = self.cursor_y.saturating_sub(page);
                } else {
                    let from = (text_buf.current_index, text_buf.line_index);
                    text_buf.change_cursor_page(-1, screen.h.saturating_sub(1).max(1) as usize);
                    let to = (text_buf.current_index, text_buf.line_index);
                    if from != to {
                        self.text_cursor_move = Some((from, to));
                    }
                }
            }
            KeyCode::PageDown => {
                if self.mode == EInputMode::FreeMove {
                    let page = screen.h.saturating_sub(1).max(1);
                    let max_y = screen.h.saturating_sub(1);
                    self.cursor_y = self.cursor_y.saturating_add(page).min(max_y);
                } else {
                    let from = (text_buf.current_index, text_buf.line_index);
                    text_buf.change_cursor_page(1, screen.h.saturating_sub(1).max(1) as usize);
                    let to = (text_buf.current_index, text_buf.line_index);
                    if from != to {
                        self.text_cursor_move = Some((from, to));
                    }
                }
            }
            KeyCode::Home => {
                if self.mode == EInputMode::FreeMove {
                    self.cursor_x = 0;
                } else {
                    let from = (text_buf.current_index, text_buf.line_index);
                    text_buf.move_to_line_start();
                    let to = (text_buf.current_index, text_buf.line_index);
                    if from != to {
                        self.text_cursor_move = Some((from, to));
                    }
                }
            }
            KeyCode::End => {
                if self.mode == EInputMode::FreeMove {
                    self.cursor_x = screen.w.saturating_sub(1);
                } else {
                    let from = (text_buf.current_index, text_buf.line_index);
                    text_buf.move_to_line_end();
                    let to = (text_buf.current_index, text_buf.line_index);
                    if from != to {
                        self.text_cursor_move = Some((from, to));
                    }
                }
            }

            KeyCode::Enter => {
                if self.mode == EInputMode::FreeMove {
                    self.clicked = Some((self.cursor_x, self.cursor_y));
                } else if self.mode == EInputMode::SearchQueryEditor && !self.is_shift {
                    self.key_command = Some(EKeyCommand::FindNext);
                } else {
                    self.pending_text.push('\n');
                }
            }

            KeyCode::Backspace => {
                if self.is_text_mode() {
                    text_buf.remove_char_backspace();
                }
            }

            KeyCode::Delete => {
                if self.is_text_mode() {
                    text_buf.remove_char_delete();
                }
            }


            KeyCode::Tab => {
                if self.is_text_mode() {
                    self.pending_text.push_str(&" ".repeat(4));
                }
            }

            KeyCode::Char(c) => {
                if self.is_text_mode() && !is_ctrl_shortcut {
                    self.pending_text.push(c);
                }
            }

            _ => {}
        }
    }
}
