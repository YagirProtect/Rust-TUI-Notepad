use crate::config::Config;
use crate::controls::t_render::Render;
use crate::e_actions::Action;
use crate::input::{EInputMode, EKeyCommand, Input};
use crate::logger::FileLogger;
use crate::panels::menu_panel::LayoutPanel;
use crate::panels::pop_up_panel::PopUpPanelFrame;
use crate::screen_buffer::{Color, ScreenBuf};
use crate::syntax_highlight;
use crate::text_buffer::TextBuf;
use crate::ui::c_frame::{EFrameAxis, Frame};
use crate::ui::c_layout::Layout;
use crate::ui::c_rect::Rect;

const RENDER_TAB_WIDTH: usize = 4;

pub struct TextEditorFrame {
    available_rect: Rect,
    frame: u16,
    offset: u16,
    show_link_hints: bool,
    hovered_link: Option<(usize, usize, usize)>,
    highlight_keywords: bool,
}

impl TextEditorFrame {
    fn render_char(ch: char) -> char {
        if ch.is_control() { '?' } else { ch }
    }

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

    fn screen_to_text_pos(&self, text_rect: Rect, screen_x: u16, screen_y: u16, text_buf: &TextBuf) -> (usize, usize) {
        let (scroll_x, scroll_y) = text_buf.scroll_offset();
        let line = scroll_y
            + screen_y
                .saturating_sub(text_rect.y)
                .min(text_rect.h.saturating_sub(1)) as usize;
        let line = line.min(text_buf.lines.len().saturating_sub(1));

        let text_x = screen_x.max(text_rect.x).saturating_sub(text_rect.x) as usize;
        let column = (scroll_x + text_x).min(text_buf.lines[line].len());
        (column, line)
    }

    fn drag_screen_to_text_pos(
        &self,
        text_rect: Rect,
        screen_x: u16,
        screen_y: u16,
        text_buf: &TextBuf,
    ) -> (usize, usize) {
        let (scroll_x, scroll_y) = text_buf.scroll_offset();
        let line_count = text_buf.lines.len().max(1);

        let line = if screen_y <= text_rect.y {
            scroll_y.saturating_sub(1)
        } else if screen_y >= text_rect.y + text_rect.h.saturating_sub(1) {
            (scroll_y + text_rect.h as usize).min(line_count.saturating_sub(1))
        } else {
            scroll_y + screen_y.saturating_sub(text_rect.y) as usize
        }
        .min(line_count.saturating_sub(1));

        let column = if screen_x <= text_rect.x {
            scroll_x.saturating_sub(1)
        } else if screen_x >= text_rect.x + text_rect.w.saturating_sub(1) {
            scroll_x + text_rect.w as usize
        } else {
            scroll_x + screen_x.saturating_sub(text_rect.x) as usize
        };
        let column = column.min(text_buf.lines[line].len());

        (column, line)
    }

    fn visible_link_at_screen_pos(
        &self,
        text_rect: Rect,
        screen_x: u16,
        screen_y: u16,
        text_buf: &TextBuf,
    ) -> Option<(usize, usize, usize, String)> {
        if !text_rect.contains(screen_x, screen_y) {
            return None;
        }

        let (scroll_x, scroll_y) = text_buf.scroll_offset();
        let draw_row = screen_y.saturating_sub(text_rect.y) as usize;
        let line_index = scroll_y + draw_row;
        let links = text_buf.links_in_line(line_index);

        for (start, end, url) in links {
            let visible_start = start.max(scroll_x);
            let visible_end = end.min(scroll_x + text_rect.w as usize);
            if visible_start >= visible_end {
                continue;
            }

            let screen_start = text_rect.x + (visible_start - scroll_x) as u16;
            let screen_end = text_rect.x + (visible_end - scroll_x) as u16;
            if screen_x >= screen_start && screen_x < screen_end {
                return Some((line_index, start, end, url));
            }
        }

        None
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
        self.highlight_keywords = _config.highlight_keywords();

        layout.close_frame();
    }

    fn interact(&mut self, _file_logger: &mut FileLogger, input: &mut Input, _pop_pup: &mut PopUpPanelFrame, text_buf: &mut TextBuf) -> Action {
        self.update_offset(text_buf);
        self.show_link_hints = input.is_shift || input.is_ctrl;
        let text_rect = self.text_rect();
        let editor_rect = self.available_rect;
        let mut handled_mouse_scroll = false;
        text_buf.set_viewport_size(text_rect.w, text_rect.h);
        self.hovered_link = None;
        if input.mouse_down.is_none() {
            text_buf.clear_word_selection_anchor();
        }
        if input.clicked.is_some() && input.double_clicked.is_none() {
            text_buf.clear_word_selection_anchor();
        }

        if self.show_link_hints {
            self.hovered_link = self
                .visible_link_at_screen_pos(text_rect, input.cursor_x, input.cursor_y, text_buf)
                .map(|(line, start, end, _)| (line, start, end));
        }

        if let Some((up_x, up_y)) = input.mouse_released {
            if (input.is_shift || input.is_ctrl) && editor_rect.contains(up_x, up_y) {
                if let Some((_, _, _, url)) =
                    self.visible_link_at_screen_pos(text_rect, up_x, up_y, text_buf)
                {
                    return Action::OpenUrl(url);
                }
            }
        }

        if input.mode == EInputMode::TextEditor && input.mouse_down.is_none() {
            if input.cursor_x < text_rect.x {
                input.cursor_x = text_rect.x;
            }
        }

        if let Some((double_x, double_y)) = input.double_clicked {
            if editor_rect.contains(double_x, double_y) {
                if input.mode != EInputMode::TextEditor {
                    input.change_mode(EInputMode::TextEditor);
                }

                let pos = self.screen_to_text_pos(text_rect, double_x, double_y, text_buf);
                text_buf.start_word_selection_at(pos);
                text_buf.ensure_cursor_visible(text_rect.w, text_rect.h);
            }
        }

        if let Some((down_x, down_y)) = input.mouse_down {
            if editor_rect.contains(down_x, down_y) {
                if input.mode != EInputMode::TextEditor {
                    input.change_mode(EInputMode::TextEditor);
                }

                let current_pos = self.drag_screen_to_text_pos(text_rect, input.cursor_x, input.cursor_y, text_buf);
                if text_buf.update_word_selection_to(current_pos) {
                    text_buf.ensure_cursor_visible(text_rect.w, text_rect.h);
                } else {
                    let anchor = *input.text_mouse_anchor.get_or_insert(current_pos);
                    text_buf.set_cursor(current_pos);
                    text_buf.ensure_cursor_visible(text_rect.w, text_rect.h);

                    if current_pos != anchor {
                        text_buf.selection_start = anchor;
                        text_buf.selection_end = current_pos;
                    } else if input.mouse_released.is_some() {
                        text_buf.clear_selection();
                    }
                }
            }
        }

        if let Some((dx, dy)) = input.mouse_scroll {
            if editor_rect.contains(input.cursor_x, input.cursor_y) {
                text_buf.scroll_with_cursor(dx, dy);
                handled_mouse_scroll = true;
            }
        }

        if (editor_rect.contains(input.cursor_x, input.cursor_y) && input.mode != EInputMode::TextEditor) {
            if (input.clicked.is_some()){
                input.change_mode(EInputMode::TextEditor);
            }
        }else if (!editor_rect.contains(input.cursor_x, input.cursor_y) && input.mode == EInputMode::TextEditor && input.mouse_down.is_none()) {
            input.change_mode(EInputMode::FreeMove);
        }

        if text_buf.lines.len() > 0 && !handled_mouse_scroll {
            text_buf.ensure_cursor_visible(text_rect.w, text_rect.h);
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

            if let Some(command) = input.key_command {
                match command {
                    EKeyCommand::Find => {}
                    EKeyCommand::Replace => {}
                    EKeyCommand::FindNext => {}
                    EKeyCommand::NewFile => {}
                    EKeyCommand::OpenFile => {}
                    EKeyCommand::OpenInExplorer => {}
                    EKeyCommand::SaveFile => {}
                    EKeyCommand::SaveFileAs => {}
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
                if !handled_mouse_scroll {
                    text_buf.ensure_cursor_visible(text_rect.w, text_rect.h);
                }
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
        text_buf.set_viewport_size(text_rect.w, text_rect.h);
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
            let line_links = text_buf.links_in_line(line_index);
            let syntax_colors = if self.highlight_keywords {
                let line_state = text_buf.syntax_state_before_line(line_index);
                let (colors, _) =
                    syntax_highlight::line_colors_with_state(&text_buf.lines[line_index], line_state);
                Some(colors)
            } else {
                None
            };

            let index: Vec<char> = (line_index + 1).to_string().chars().collect();
            for i in 0..index.len() {
                screen.set(index_x + i as u16, draw_y, index[i], Color::Gray);
            }

            let mut draw_col = 0usize;
            let mut x = scroll_x;
            while draw_col < text_rect.w as usize {
                if x >= text_buf.lines[line_index].len() {
                    break;
                }

                let ch = text_buf.lines[line_index][x];
                let draw_x = min_x + draw_col as u16;
                if ch == '\t' {
                    let tab_fill = RENDER_TAB_WIDTH.min(text_rect.w as usize - draw_col);
                    for offset in 0..tab_fill {
                        screen.set(draw_x + offset as u16, draw_y, ' ', Color::White);
                    }
                    draw_col += tab_fill;
                } else if text_buf.is_selected(x, line_index) {
                    screen.set_with_bg(draw_x, draw_y, Self::render_char(ch), Color::White, Color::Blue);
                    draw_col += 1;
                } else if let Some(is_current_match) = text_buf.search_highlight_at(x, line_index) {
                    let background = if is_current_match { Color::Green } else { Color::Yellow };
                    screen.set_with_bg(draw_x, draw_y, Self::render_char(ch), Color::Black, background);
                    draw_col += 1;
                } else if line_links
                    .iter()
                    .any(|(start, end, _)| x >= *start && x < *end)
                {
                    if self.hovered_link == Some((line_index, x, x + 1))
                        || self.hovered_link
                            .is_some_and(|(hover_line, start, end)| hover_line == line_index && x >= start && x < end)
                    {
                        screen.set_with_bg(draw_x, draw_y, Self::render_char(ch), Color::White, Color::Blue);
                        draw_col += 1;
                    } else {
                        screen.set(draw_x, draw_y, Self::render_char(ch), Color::Blue);
                        draw_col += 1;
                    }
                } else if let Some(color) = syntax_colors
                    .as_ref()
                    .and_then(|colors| colors.get(x))
                    .and_then(|color| *color)
                {
                    screen.set(draw_x, draw_y, Self::render_char(ch), color);
                    draw_col += 1;
                } else {
                    screen.set(draw_x, draw_y, Self::render_char(ch), Color::White);
                    draw_col += 1;
                }
                x += 1;
            }
        }
    }
}

impl Default for TextEditorFrame {
    fn default() -> Self {
        Self{
            available_rect: Default::default(),
            frame: 0,
            offset: 0,
            show_link_hints: false,
            hovered_link: None,
            highlight_keywords: true,
        }
    }
}
