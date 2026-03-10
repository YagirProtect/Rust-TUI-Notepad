use crate::config::Config;
use crate::controls::t_render::Render;
use crate::e_actions::Action;
use crate::input::{EInputMode, EKeyCommand, Input};
use crate::logger::FileLogger;
use crate::panels::menu_panel::LayoutPanel;
use crate::panels::pop_up_panel::PopUpPanelFrame;
use crate::screen_buffer::{Color, ScreenBuf};
use crate::text_buffer::TextBuf;
use crate::ui::c_frame::{EFrameAxis, Frame};
use crate::ui::c_layout::Layout;
use crate::ui::c_rect::Rect;

pub struct TextEditorFrame {
    available_rect: Rect,
    frame: u16,
    offset: u16,
}

impl TextEditorFrame {
    fn update_offset(&mut self, text_buf: &TextBuf) {
        self.offset = 0;
        if text_buf.lines.len() < 10 {
            self.offset = 2;
        } else if text_buf.lines.len() < 100 {
            self.offset = 3;
        } else if text_buf.lines.len() < 1000 {
            self.offset = 4;
        } else if text_buf.lines.len() < 10000 {
            self.offset = 5;
        }
    }

    fn text_rect(&self) -> Rect {
        Rect::new(
            self.available_rect.x.saturating_add(self.offset),
            self.available_rect.y,
            self.available_rect.w.saturating_sub(self.offset),
            self.available_rect.h,
        )
    }
}

impl LayoutPanel for TextEditorFrame {
    fn get_order(&self) -> u16 {
        0
    }

    fn get_frame_id(&self) -> u16 {
        return self.frame;
    }

    fn create_layout(&mut self, layout: &mut Layout, _config: &mut Config) {
        let frame = Frame::new(EFrameAxis::Vertical, false);

        let root_rect = layout.get_root_rect();

        let mut open_frame = layout.open_frame(frame);
        open_frame.fill(root_rect);

        self.frame = open_frame.frame_id;
        self.available_rect = open_frame.content_rect();

        layout.close_frame();
    }

    fn interact(&mut self, _file_logger: &mut FileLogger, input: &mut Input, _pop_pup: &mut PopUpPanelFrame, text_buf: &mut TextBuf) -> Action {
        self.update_offset(text_buf);
        let text_rect = self.text_rect();

        if (input.mode == EInputMode::TextEditor){
            if (input.cursor_x < text_rect.x){
                input.cursor_x = text_rect.x;
            }
        }

        if (text_rect.contains(input.cursor_x, input.cursor_y) && input.mode == EInputMode::FreeMove) {
            if (input.clicked.is_some()){
                input.change_mode(EInputMode::TextEditor);
            }
        }else if (!text_rect.contains(input.cursor_x, input.cursor_y) && input.mode == EInputMode::TextEditor) {
            input.change_mode(EInputMode::FreeMove);
        }

        if (input.mode == EInputMode::TextEditor) {
            if let Some((from, to)) = input.text_cursor_move {
                if input.is_shift {
                    if !text_buf.has_selection() {
                        text_buf.selection_start = from;
                    }
                    text_buf.selection_end = to;
                } else {
                    text_buf.clear_selection();
                }
            }

            text_buf.ensure_cursor_visible(text_rect.w, text_rect.h);

            if let Some(command) = input.key_command {
                match command {
                    EKeyCommand::Undo => {
                        text_buf.undo();
                    }
                    EKeyCommand::Redo => {
                        text_buf.redo();
                    }
                    EKeyCommand::SelectAll => {
                        text_buf.select_all();
                    }
                    EKeyCommand::Copy => {
                        text_buf.copy_selection();
                    }
                    EKeyCommand::Cut => {
                        text_buf.cut_selection();
                    }
                    EKeyCommand::Paste => {
                        if let Some(text) = text_buf.paste_from_clipboard_text() {
                            input.arm_paste_suppression(&text);
                        }
                    }
                }
            }

            if !input.pending_text.is_empty() {
                if text_buf.lines.is_empty() {
                    text_buf.lines.push(Vec::new());
                }

                text_buf.paste_text(&input.pending_text);
            }

            if (text_buf.lines.len() > 0){
                text_buf.ensure_cursor_visible(text_rect.w, text_rect.h);
                let (scroll_x, scroll_y) = text_buf.scroll_offset();
                let max_x = text_rect.x + text_rect.w.saturating_sub(1);
                let min_x = text_rect.x;
                let max_y = text_rect.y + text_rect.h.saturating_sub(1);
                let min_y = text_rect.y;

                input.cursor_y = text_rect.y + text_buf.line_index.saturating_sub(scroll_y) as u16;
                input.cursor_x = text_rect.x + text_buf.current_index.saturating_sub(scroll_x) as u16;

                input.clamp_cursor(min_x, max_x, min_y, max_y);
            }
        }

        Action::None
    }

    fn draw(&mut self, layout: &mut Layout, screen: &mut ScreenBuf, text_buf: &mut TextBuf) {
        self.update_offset(text_buf);
        let text_rect = self.text_rect();
        let frame = layout.get_frame(self.frame).unwrap();

        let min_x = text_rect.x;
        let min_y = text_rect.y;
        let max_x = text_rect.x + text_rect.w;
        let index_x = min_x.saturating_sub(self.offset);
        let (scroll_x, scroll_y) = text_buf.scroll_offset();

        frame.draw(&Rect::default(), screen);

        for draw_row in 0..text_rect.h as usize {
            let line_index = scroll_y + draw_row;
            if line_index >= text_buf.lines.len() {
                break;
            }
            let draw_y = min_y + draw_row as u16;

            let index: Vec<char> = (line_index + 1).to_string().chars().collect();
            for i in 0..index.len() {
                screen.set(index_x + i as u16, draw_y, index[i], Color::Gray);
            }

            for draw_col in 0..text_rect.w as usize {
                let x = scroll_x + draw_col;
                if x >= text_buf.lines[line_index].len() {
                    break;
                }

                let ch = text_buf.lines[line_index][x];
                if text_buf.is_selected(x, line_index) {
                    screen.set_with_bg(min_x + draw_col as u16, draw_y, ch, Color::White, Color::Blue);
                } else {
                    screen.set(min_x + draw_col as u16, draw_y, ch, Color::White);
                }
            }
        }
    }
}

impl Default for TextEditorFrame {
    fn default() -> Self {
        Self{
            available_rect: Default::default(),
            frame: 0,
            offset: 0
        }
    }
}
