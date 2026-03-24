use crate::characters::BORDER_ROUNDED;
use crate::controls::c_button::Button;
use crate::controls::t_get_rect::Control;
use crate::controls::t_render::Render;
use crate::input::{EInputMode, EKeyCommand, Input};
use crate::logger::FileLogger;
use crate::screen_buffer::{Color, ScreenBuf};
use crate::text_buffer::TextBuf;
use crate::ui::c_rect::Rect;

#[derive(Copy, Clone, Eq, PartialEq)]
enum SearchPanelMode {
    Find,
    Replace,
}

pub struct SearchPanelFrame {
    pub active: bool,
    mode: SearchPanelMode,
    query_buffer: TextBuf,
    replace_buffer: TextBuf,
    previous_mode: EInputMode,
    last_query_text: String,
    last_document_version: u64,
    prev_button: Button,
    next_button: Button,
    replace_button: Button,
    replace_all_button: Button,
    close_button: Button,
}

#[derive(Copy, Clone)]
struct SearchRects {
    panel: Rect,
    content: Rect,
    query_frame: Rect,
    query_content: Rect,
    replace_frame: Rect,
    replace_content: Rect,
    prev_button: Rect,
    next_button: Rect,
    replace_button: Rect,
    replace_all_button: Rect,
    close_button: Rect,
}

impl SearchPanelFrame {
    pub fn new() -> Self {
        Self {
            active: false,
            mode: SearchPanelMode::Find,
            query_buffer: TextBuf::default(),
            replace_buffer: TextBuf::default(),
            previous_mode: EInputMode::FreeMove,
            last_query_text: String::new(),
            last_document_version: 0,
            prev_button: Button::new("[Prev]"),
            next_button: Button::new("[Next]"),
            replace_button: Button::new("[Replace]"),
            replace_all_button: Button::new("[Replace All]"),
            close_button: Button::new("[Close]"),
        }
    }

    pub fn open_find(&mut self, input: &mut Input, text_buf: &mut TextBuf) {
        self.open(SearchPanelMode::Find, input, text_buf);
    }

    pub fn open_replace(&mut self, input: &mut Input, text_buf: &mut TextBuf) {
        self.open(SearchPanelMode::Replace, input, text_buf);
    }

    fn open(&mut self, mode: SearchPanelMode, input: &mut Input, text_buf: &mut TextBuf) {
        if !input.is_search_mode() {
            self.previous_mode = input.mode;
        }

        self.mode = mode;
        self.active = true;
        input.change_mode(EInputMode::SearchQueryEditor);

        if let Some(selection_text) = text_buf.selected_text() {
            self.query_buffer = TextBuf::default();
            self.query_buffer.paste_text(&selection_text);
            self.query_buffer.clear_selection();
        }

        self.last_query_text.clear();
        self.last_document_version = u64::MAX;
        self.sync_matches(text_buf);
        self.scroll_to_current_match(text_buf);
    }

    pub fn close(&mut self, input: &mut Input, text_buf: &mut TextBuf) {
        self.active = false;
        text_buf.clear_search_matches();
        if input.is_search_mode() {
            input.change_mode(self.previous_mode);
        }
    }

    pub fn hit(&self, root: Rect, x: u16, y: u16) -> bool {
        self.active && self.layout_rects(root).panel.contains(x, y)
    }

    pub fn active_buffer_mut(&mut self, mode: EInputMode) -> &mut TextBuf {
        match mode {
            EInputMode::SearchReplaceEditor => &mut self.replace_buffer,
            _ => &mut self.query_buffer,
        }
    }

    pub fn interact(&mut self, root: Rect, input: &mut Input, text_buf: &mut TextBuf, logger: &mut FileLogger) -> bool {
        if !self.active {
            return false;
        }

        let rects = self.layout_rects(root);
        self.update_buttons(rects, input, logger);

        if input.mouse_down.is_none() && input.is_search_mode() {
            self.active_buffer_mut(input.mode).clear_word_selection_anchor();
        }

        if let Some((double_x, double_y)) = input.double_clicked {
            if rects.query_content.contains(double_x, double_y) {
                input.change_mode(EInputMode::SearchQueryEditor);
                let pos = Self::buffer_pos_at(&self.query_buffer, rects.query_content, double_x, double_y);
                self.query_buffer.start_word_selection_at(pos);
                self.query_buffer.ensure_cursor_visible(rects.query_content.w, rects.query_content.h);
                input.text_mouse_anchor = None;
                return true;
            }

            if self.mode == SearchPanelMode::Replace
                && rects.replace_content.contains(double_x, double_y)
            {
                input.change_mode(EInputMode::SearchReplaceEditor);
                let pos = Self::buffer_pos_at(&self.replace_buffer, rects.replace_content, double_x, double_y);
                self.replace_buffer.start_word_selection_at(pos);
                self.replace_buffer.ensure_cursor_visible(rects.replace_content.w, rects.replace_content.h);
                input.text_mouse_anchor = None;
                return true;
            }
        }

        if input.clicked.is_some() && input.double_clicked.is_none() {
            if rects.query_content.contains(input.cursor_x, input.cursor_y) {
                input.change_mode(EInputMode::SearchQueryEditor);
                self.query_buffer.clear_word_selection_anchor();
                Self::place_cursor(&mut self.query_buffer, rects.query_content, input.cursor_x, input.cursor_y);
                return true;
            }

            if self.mode == SearchPanelMode::Replace
                && rects.replace_content.contains(input.cursor_x, input.cursor_y)
            {
                input.change_mode(EInputMode::SearchReplaceEditor);
                self.replace_buffer.clear_word_selection_anchor();
                Self::place_cursor(&mut self.replace_buffer, rects.replace_content, input.cursor_x, input.cursor_y);
                return true;
            }

            if input.is_search_mode() && !rects.panel.contains(input.cursor_x, input.cursor_y) {
                input.change_mode(EInputMode::FreeMove);
            }
        }

        if let Some((down_x, down_y)) = input.mouse_down {
            let target_mode = if rects.query_content.contains(down_x, down_y) {
                Some(EInputMode::SearchQueryEditor)
            } else if self.mode == SearchPanelMode::Replace
                && rects.replace_content.contains(down_x, down_y)
            {
                Some(EInputMode::SearchReplaceEditor)
            } else {
                None
            };

            if let Some(mode) = target_mode {
                if input.mode != mode {
                    input.change_mode(mode);
                }

                let input_rect = self.active_input_rect(rects, mode);
                let active_buffer = self.active_buffer_mut(mode);
                let current_pos =
                    Self::drag_buffer_pos_at(active_buffer, input_rect, input.cursor_x, input.cursor_y);

                if active_buffer.update_word_selection_to(current_pos) {
                    active_buffer.ensure_cursor_visible(input_rect.w, input_rect.h);
                } else {
                    let anchor = *input.text_mouse_anchor.get_or_insert(current_pos);
                    active_buffer.set_cursor(current_pos);
                    active_buffer.ensure_cursor_visible(input_rect.w, input_rect.h);

                    if current_pos != anchor {
                        active_buffer.selection_start = anchor;
                        active_buffer.selection_end = current_pos;
                    } else if input.mouse_released.is_some() {
                        active_buffer.clear_selection();
                    }
                }

                return true;
            }
        }

        if let Some((dx, dy)) = input.mouse_scroll {
            if rects.query_content.contains(input.cursor_x, input.cursor_y) {
                Self::scroll_buffer(&mut self.query_buffer, dx, dy);
                return true;
            }

            if self.mode == SearchPanelMode::Replace
                && rects.replace_content.contains(input.cursor_x, input.cursor_y)
            {
                Self::scroll_buffer(&mut self.replace_buffer, dx, dy);
                return true;
            }
        }

        if self.close_button.clicked() {
            self.close(input, text_buf);
            return true;
        }
        if self.prev_button.clicked() {
            self.select_prev(text_buf);
            return true;
        }
        if self.next_button.clicked() {
            self.select_next(text_buf);
            return true;
        }
        if self.mode == SearchPanelMode::Replace && self.replace_button.clicked() {
            self.replace_current(text_buf);
            self.sync_matches(text_buf);
            self.scroll_to_current_match(text_buf);
            return true;
        }
        if self.mode == SearchPanelMode::Replace && self.replace_all_button.clicked() {
            self.replace_all(text_buf);
            self.sync_matches(text_buf);
            self.scroll_to_current_match(text_buf);
            return true;
        }

        if input.mouse_released.is_some() {
            if rects.close_button.contains(input.cursor_x, input.cursor_y) {
                self.close(input, text_buf);
                return true;
            }
        }

        if !input.is_search_mode() {
            self.sync_matches(text_buf);
            return false;
        }

        if let Some((from, to)) = input.text_cursor_move {
            let active_buffer = self.active_buffer_mut(input.mode);
            if input.is_shift {
                if !active_buffer.has_selection() {
                    active_buffer.selection_start = from;
                }
                active_buffer.selection_end = to;
            } else {
                active_buffer.clear_selection();
            }
        }

        if let Some(command) = input.key_command {
            match command {
                EKeyCommand::Find => {}
                EKeyCommand::Replace => {}
                EKeyCommand::FindNext => {
                    self.select_next(text_buf);
                }
                EKeyCommand::NewFile => {}
                EKeyCommand::OpenFile => {}
                EKeyCommand::OpenInExplorer => {}
                EKeyCommand::SaveFile => {}
                EKeyCommand::SaveFileAs => {}
                EKeyCommand::Undo => {
                    self.active_buffer_mut(input.mode).undo();
                }
                EKeyCommand::Redo => {
                    self.active_buffer_mut(input.mode).redo();
                }
                EKeyCommand::SelectAll => {
                    self.active_buffer_mut(input.mode).select_all();
                }
                EKeyCommand::Copy => {
                    self.active_buffer_mut(input.mode).copy_selection();
                }
                EKeyCommand::Cut => {
                    self.active_buffer_mut(input.mode).cut_selection();
                }
                EKeyCommand::Paste => {
                    self.active_buffer_mut(input.mode).paste_from_clipboard();
                }
            }
        }

        if !input.pending_text.is_empty() {
            self.active_buffer_mut(input.mode).paste_text(&input.pending_text);
        }

        let query_changed = self.query_buffer.text() != self.last_query_text;
        self.sync_matches(text_buf);
        if query_changed {
            self.scroll_to_current_match(text_buf);
        }
        let active_rect = self.active_input_rect(rects, input.mode);
        self.active_buffer_mut(input.mode)
            .ensure_cursor_visible(active_rect.w, active_rect.h);
        self.update_input_cursor(input, active_rect);
        true
    }

    pub fn draw(&mut self, root: Rect, screen: &mut ScreenBuf, input: &mut Input, text_buf: &mut TextBuf) {
        if !self.active {
            return;
        }

        let rects = self.layout_rects(root);
        self.draw_box(screen, rects.panel, Color::Blue, Color::Black);

        Self::draw_text(
            screen,
            rects.content.x,
            rects.content.y,
            rects.content.w,
            match self.mode {
                SearchPanelMode::Find => "Find",
                SearchPanelMode::Replace => "Replace",
            },
            Color::White,
            Color::Black,
        );

        let count_text = match text_buf.search_match_count() {
            0 => "0 matches".to_string(),
            total => {
                if let Some(current) = text_buf.current_search_match_number() {
                    format!("{current} / {total}")
                } else {
                    format!("0 / {total}")
                }
            }
        };
        let count_x = rects
            .content
            .x
            .saturating_add(rects.content.w.saturating_sub(count_text.chars().count() as u16));
        Self::draw_text(screen, count_x, rects.content.y, rects.content.w, &count_text, Color::Gray, Color::Black);

        Self::draw_text(screen, rects.query_frame.x, rects.query_frame.y.saturating_sub(1), rects.content.w, "Find:", Color::Gray, Color::Black);
        self.draw_box(
            screen,
            rects.query_frame,
            if input.mode == EInputMode::SearchQueryEditor { Color::Yellow } else { Color::Gray },
            Color::Black,
        );
        Self::draw_buffer(
            screen,
            &mut self.query_buffer,
            rects.query_content,
            "Type to find...",
        );

        if self.mode == SearchPanelMode::Replace {
            Self::draw_text(
                screen,
                rects.replace_frame.x,
                rects.replace_frame.y.saturating_sub(1),
                rects.content.w,
                "Replace:",
                Color::Gray,
                Color::Black,
            );
            self.draw_box(
                screen,
                rects.replace_frame,
                if input.mode == EInputMode::SearchReplaceEditor { Color::Yellow } else { Color::Gray },
                Color::Black,
            );
            Self::draw_buffer(
                screen,
                &mut self.replace_buffer,
                rects.replace_content,
                "Replace with...",
            );
        }

        if input.is_search_mode() {
            self.update_input_cursor(input, self.active_input_rect(rects, input.mode));
        }

        let prev_rect = *self.prev_button.get_rect();
        let next_rect = *self.next_button.get_rect();
        self.prev_button.draw(&prev_rect, screen);
        self.next_button.draw(&next_rect, screen);
        if self.mode == SearchPanelMode::Replace {
            let replace_rect = *self.replace_button.get_rect();
            let replace_all_rect = *self.replace_all_button.get_rect();
            self.replace_button.draw(&replace_rect, screen);
            self.replace_all_button.draw(&replace_all_rect, screen);
        }
        let close_rect = *self.close_button.get_rect();
        self.close_button.draw(&close_rect, screen);
    }

    fn sync_matches(&mut self, text_buf: &mut TextBuf) {
        let query_text = self.query_buffer.text();
        let query_changed = query_text != self.last_query_text;
        let document_changed = text_buf.version() != self.last_document_version;

        if !query_changed && !document_changed {
            return;
        }

        let matches = text_buf.find_all(&query_text);
        let current_match = if query_changed {
            self.pick_match_from_cursor(text_buf, &matches)
        } else {
            text_buf
                .current_search_match_index()
                .filter(|&index| index < matches.len())
                .or_else(|| self.pick_match_from_cursor(text_buf, &matches))
        };

        text_buf.set_search_matches(matches, current_match);

        self.last_query_text = query_text;
        self.last_document_version = text_buf.version();
    }

    fn replace_current(&mut self, text_buf: &mut TextBuf) {
        text_buf.replace_current_search_match(&self.replace_buffer.text());
    }

    fn replace_all(&mut self, text_buf: &mut TextBuf) {
        let query = self.query_buffer.text();
        if query.is_empty() {
            return;
        }

        text_buf.replace_all_matches(&query, &self.replace_buffer.text());
    }

    fn pick_match_from_cursor(&self, text_buf: &TextBuf, matches: &[((usize, usize), (usize, usize))]) -> Option<usize> {
        if matches.is_empty() {
            return None;
        }

        let cursor = (text_buf.current_index, text_buf.line_index);
        for (index, (start, _)) in matches.iter().enumerate() {
            if !is_before(*start, cursor) {
                return Some(index);
            }
        }

        Some(0)
    }

    fn scroll_to_current_match(&self, text_buf: &mut TextBuf) {
        if let Some((start, _)) = text_buf.current_search_match_range() {
            text_buf.set_cursor(start);
            text_buf.ensure_cursor_visible_in_viewport();
            text_buf.clear_selection();
        }
    }

    fn select_next(&mut self, text_buf: &mut TextBuf) {
        let count = text_buf.search_match_count();
        if count == 0 {
            return;
        }

        let next_index = text_buf
            .current_search_match_index()
            .map(|index| (index + 1) % count)
            .unwrap_or(0);

        text_buf.set_current_search_match(Some(next_index));
        self.scroll_to_current_match(text_buf);
    }

    fn select_prev(&mut self, text_buf: &mut TextBuf) {
        let count = text_buf.search_match_count();
        if count == 0 {
            return;
        }

        let prev_index = text_buf
            .current_search_match_index()
            .map(|index| if index == 0 { count - 1 } else { index - 1 })
            .unwrap_or(0);

        text_buf.set_current_search_match(Some(prev_index));
        self.scroll_to_current_match(text_buf);
    }

    fn place_cursor(buffer: &mut TextBuf, input_rect: Rect, x: u16, y: u16) {
        let (column, line) = Self::buffer_pos_at(buffer, input_rect, x, y);
        buffer.set_cursor((column, line));
        buffer.clear_selection();
    }

    fn drag_buffer_pos_at(buffer: &TextBuf, input_rect: Rect, x: u16, y: u16) -> (usize, usize) {
        let (scroll_x, scroll_y) = buffer.scroll_offset();
        let line_count = buffer.lines.len().max(1);

        let line = if y <= input_rect.y {
            scroll_y.saturating_sub(1)
        } else if y >= input_rect.y + input_rect.h.saturating_sub(1) {
            (scroll_y + input_rect.h as usize).min(line_count.saturating_sub(1))
        } else {
            scroll_y + y.saturating_sub(input_rect.y) as usize
        }
        .min(line_count.saturating_sub(1));

        let column = if x <= input_rect.x {
            scroll_x.saturating_sub(1)
        } else if x >= input_rect.x + input_rect.w.saturating_sub(1) {
            scroll_x + input_rect.w as usize
        } else {
            scroll_x + x.saturating_sub(input_rect.x) as usize
        };
        let column = column.min(buffer.lines[line].len());
        (column, line)
    }

    fn buffer_pos_at(buffer: &TextBuf, input_rect: Rect, x: u16, y: u16) -> (usize, usize) {
        let (scroll_x, scroll_y) = buffer.scroll_offset();
        let line = scroll_y + y.saturating_sub(input_rect.y) as usize;
        let line = line.min(buffer.lines.len().saturating_sub(1));
        let column = scroll_x + x.saturating_sub(input_rect.x) as usize;
        let column = column.min(buffer.lines[line].len());
        (column, line)
    }

    fn scroll_buffer(buffer: &mut TextBuf, dx: i32, dy: i32) {
        if dy != 0 {
            buffer.scroll_vertical(dy);
        }
        if dx != 0 {
            buffer.scroll_horizontal(dx);
        }
    }

    fn update_input_cursor(&mut self, input: &mut Input, input_rect: Rect) {
        let active_buffer = self.active_buffer_mut(input.mode);
        let (scroll_x, scroll_y) = active_buffer.scroll_offset();
        let max_x = input_rect.x + input_rect.w.saturating_sub(1);
        let max_y = input_rect.y + input_rect.h.saturating_sub(1);

        input.cursor_x = input_rect.x + active_buffer.current_index.saturating_sub(scroll_x) as u16;
        input.cursor_y = input_rect.y + active_buffer.line_index.saturating_sub(scroll_y) as u16;
        input.clamp_cursor(input_rect.x, max_x, input_rect.y, max_y);
    }

    fn active_input_rect(&self, rects: SearchRects, mode: EInputMode) -> Rect {
        match mode {
            EInputMode::SearchReplaceEditor => rects.replace_content,
            _ => rects.query_content,
        }
    }

    fn layout_rects(&self, root: Rect) -> SearchRects {
        let max_width = root.w.saturating_sub(2).max(16);
        let panel_w = if max_width >= 58 { 58 } else { max_width };
        let desired_h = match self.mode {
            SearchPanelMode::Find => 10,
            SearchPanelMode::Replace => 16,
        };
        let panel_h = desired_h.min(root.h.saturating_sub(1).max(8));
        let panel_x = root.x + root.w.saturating_sub(panel_w + 1);
        let panel_y = root.y.saturating_add(1).min(root.y + root.h.saturating_sub(panel_h));
        let panel = Rect::new(panel_x, panel_y, panel_w, panel_h);
        let content = Rect::new(
            panel.x.saturating_add(1),
            panel.y.saturating_add(1),
            panel.w.saturating_sub(2),
            panel.h.saturating_sub(2),
        );

        let query_frame = Rect::new(content.x, content.y.saturating_add(2), content.w, 4);
        let query_content = Rect::new(
            query_frame.x.saturating_add(1),
            query_frame.y.saturating_add(1),
            query_frame.w.saturating_sub(2),
            query_frame.h.saturating_sub(2),
        );

        let (replace_frame, replace_content, buttons_y) = match self.mode {
            SearchPanelMode::Find => (
                Rect::default(),
                Rect::default(),
                query_frame.y.saturating_add(query_frame.h).saturating_add(1),
            ),
            SearchPanelMode::Replace => {
                let frame = Rect::new(content.x, query_frame.y.saturating_add(query_frame.h).saturating_add(2), content.w, 4);
                let inner = Rect::new(
                    frame.x.saturating_add(1),
                    frame.y.saturating_add(1),
                    frame.w.saturating_sub(2),
                    frame.h.saturating_sub(2),
                );
                let y = frame.y.saturating_add(frame.h).saturating_add(1);
                (frame, inner, y)
            }
        };

        let prev_button = Rect::new(content.x, buttons_y, 6, 1);
        let next_button = Rect::new(prev_button.x.saturating_add(prev_button.w).saturating_add(1), buttons_y, 6, 1);
        let replace_button = Rect::new(next_button.x.saturating_add(next_button.w).saturating_add(1), buttons_y, 9, 1);
        let replace_all_button = Rect::new(
            replace_button.x.saturating_add(replace_button.w).saturating_add(1),
            buttons_y,
            13,
            1,
        );
        let close_button = Rect::new(
            content.x + content.w.saturating_sub(7),
            buttons_y,
            7,
            1,
        );

        SearchRects {
            panel,
            content,
            query_frame,
            query_content,
            replace_frame,
            replace_content,
            prev_button,
            next_button,
            replace_button,
            replace_all_button,
            close_button,
        }
    }

    fn draw_box(&self, screen: &mut ScreenBuf, rect: Rect, border_color: Color, fill_color: Color) {
        if rect.w < 2 || rect.h < 2 {
            return;
        }

        let x0 = rect.x;
        let y0 = rect.y;
        let x1 = rect.x + rect.w - 1;
        let y1 = rect.y + rect.h - 1;

        for y in y0..=y1 {
            for x in x0..=x1 {
                screen.set_with_bg(x, y, ' ', Color::White, fill_color);
            }
        }

        screen.set(x0, y0, BORDER_ROUNDED.tl, border_color);
        screen.set(x1, y0, BORDER_ROUNDED.tr, border_color);
        screen.set(x0, y1, BORDER_ROUNDED.bl, border_color);
        screen.set(x1, y1, BORDER_ROUNDED.br, border_color);

        for x in (x0 + 1)..x1 {
            screen.set(x, y0, BORDER_ROUNDED.h, border_color);
            screen.set(x, y1, BORDER_ROUNDED.h, border_color);
        }

        for y in (y0 + 1)..y1 {
            screen.set(x0, y, BORDER_ROUNDED.v, border_color);
            screen.set(x1, y, BORDER_ROUNDED.v, border_color);
        }
    }

    fn draw_text(screen: &mut ScreenBuf, x: u16, y: u16, max_width: u16, text: &str, foreground: Color, background: Color) {
        for (offset, ch) in text.chars().take(max_width as usize).enumerate() {
            screen.set_with_bg(x + offset as u16, y, ch, foreground, background);
        }
    }

    fn draw_buffer(screen: &mut ScreenBuf, buffer: &mut TextBuf, rect: Rect, placeholder: &str) {
        if buffer.lines.len() > 0 {
            buffer.ensure_cursor_visible(rect.w, rect.h);
        }

        let (scroll_x, scroll_y) = buffer.scroll_offset();
        for draw_row in 0..rect.h as usize {
            let line_index = scroll_y + draw_row;
            if line_index >= buffer.lines.len() {
                break;
            }

            let draw_y = rect.y + draw_row as u16;
            for draw_col in 0..rect.w as usize {
                let x = scroll_x + draw_col;
                if x >= buffer.lines[line_index].len() {
                    break;
                }

                let ch = buffer.lines[line_index][x];
                if buffer.is_selected(x, line_index) {
                    screen.set_with_bg(rect.x + draw_col as u16, draw_y, ch, Color::White, Color::Blue);
                } else {
                    screen.set_with_bg(rect.x + draw_col as u16, draw_y, ch, Color::White, Color::Black);
                }
            }
        }

        if buffer.text().is_empty() {
            Self::draw_text(screen, rect.x, rect.y, rect.w, placeholder, Color::Gray, Color::Black);
        }
    }

    fn update_buttons(&mut self, rects: SearchRects, input: &Input, logger: &mut FileLogger) {
        self.prev_button.save_rect(rects.prev_button);
        self.prev_button.calculate_control(logger, input);

        self.next_button.save_rect(rects.next_button);
        self.next_button.calculate_control(logger, input);

        self.close_button.save_rect(rects.close_button);
        self.close_button.calculate_control(logger, input);

        if self.mode == SearchPanelMode::Replace {
            self.replace_button.save_rect(rects.replace_button);
            self.replace_button.calculate_control(logger, input);

            self.replace_all_button.save_rect(rects.replace_all_button);
            self.replace_all_button.calculate_control(logger, input);
        }
    }
}

fn is_before(a: (usize, usize), b: (usize, usize)) -> bool {
    a.1 < b.1 || (a.1 == b.1 && a.0 < b.0)
}
