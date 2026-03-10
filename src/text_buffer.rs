use cli_clipboard::{ClipboardContext, ClipboardProvider};

type CursorPos = (usize, usize);
type Selection = (CursorPos, CursorPos);

#[derive(Clone)]
enum EditKind {
    Insert,
    Delete,
}

#[derive(Clone)]
struct EditCommand {
    kind: EditKind,
    start: CursorPos,
    text: String,
    before_cursor: CursorPos,
    before_selection: Selection,
    after_cursor: CursorPos,
    after_selection: Selection,
}

pub struct TextBuf{
    pub lines: Vec<Vec<char>>,
    pub current_index: usize,
    pub line_index: usize,

    pub selection_start : CursorPos,
    pub selection_end : CursorPos,

    undo_stack: Vec<EditCommand>,
    redo_stack: Vec<EditCommand>,
    scroll_x: usize,
    scroll_y: usize,
}

const TAB_WIDTH: usize = 4;

impl TextBuf {
    pub fn ensure_cursor_visible(&mut self, view_width: u16, view_height: u16) {
        let view_width = view_width as usize;
        let view_height = view_height as usize;

        if view_width == 0 || view_height == 0 {
            return;
        }

        if self.line_index < self.scroll_y {
            self.scroll_y = self.line_index;
        } else if self.line_index >= self.scroll_y + view_height {
            self.scroll_y = self.line_index + 1 - view_height;
        }

        if self.current_index < self.scroll_x {
            self.scroll_x = self.current_index;
        } else if self.current_index >= self.scroll_x + view_width {
            self.scroll_x = self.current_index + 1 - view_width;
        }
    }

    pub fn scroll_offset(&self) -> (usize, usize) {
        (self.scroll_x, self.scroll_y)
    }

    fn cursor(&self) -> CursorPos {
        (self.current_index, self.line_index)
    }

    fn selection(&self) -> Selection {
        (self.selection_start, self.selection_end)
    }

    fn normalize_newlines(text: &str) -> String {
        text.replace("\r\n", "\n")
    }

    fn position_after_text(start: CursorPos, text: &str) -> CursorPos {
        let mut x = start.0;
        let mut y = start.1;

        for ch in text.chars() {
            if ch == '\n' {
                y += 1;
                x = 0;
            } else {
                x += 1;
            }
        }

        (x, y)
    }

    fn ensure_invariants(&mut self) {
        if self.lines.is_empty() {
            self.lines.push(Vec::new());
        }

        if self.line_index >= self.lines.len() {
            self.line_index = self.lines.len() - 1;
        }

        let len = self.lines[self.line_index].len();
        if self.current_index > len {
            self.current_index = len;
        }
    }

    fn insert_text_raw(&mut self, start: CursorPos, text: &str) -> CursorPos {
        let text = Self::normalize_newlines(text);
        if text.is_empty() {
            return start;
        }

        let (x, y) = start;
        if y >= self.lines.len() {
            self.lines.resize_with(y + 1, Vec::new);
        }

        let parts: Vec<Vec<char>> = text.split('\n').map(|line| line.chars().collect()).collect();
        let trailing_lines = self.lines.split_off(y + 1);
        let tail = self.lines[y].split_off(x);
        self.lines[y].extend(parts[0].iter().copied());

        if parts.len() == 1 {
            self.lines[y].extend(tail);
            self.lines.extend(trailing_lines);
            return (x + parts[0].len(), y);
        }

        let mut inserted_lines = Vec::with_capacity(parts.len() - 1 + trailing_lines.len());
        for line in parts.iter().skip(1).take(parts.len().saturating_sub(2)) {
            inserted_lines.push(line.clone());
        }
        let mut last_line = parts.last().unwrap().clone();
        last_line.extend(tail);
        inserted_lines.push(last_line);
        inserted_lines.extend(trailing_lines);
        self.lines.extend(inserted_lines);

        (parts.last().unwrap().len(), y + parts.len() - 1)
    }

    fn delete_range_raw(&mut self, start: CursorPos, end: CursorPos) -> String {
        if start == end {
            return String::new();
        }

        let ((sx, sy), (ex, ey)) = if syx_order(start, end) { (start, end) } else { (end, start) };

        let mut deleted = String::new();

        if sy == ey {
            let removed: Vec<char> = self.lines[sy].drain(sx..ex).collect();
            deleted.extend(removed);
            return deleted;
        }

        deleted.extend(self.lines[sy][sx..].iter().copied());
        deleted.push('\n');

        for line in (sy + 1)..ey {
            deleted.extend(self.lines[line].iter().copied());
            deleted.push('\n');
        }

        deleted.extend(self.lines[ey][..ex].iter().copied());

        let mut merged = self.lines[sy][..sx].to_vec();
        merged.extend_from_slice(&self.lines[ey][ex..]);
        self.lines[sy] = merged;
        self.lines.drain((sy + 1)..=ey);

        if self.lines.is_empty() {
            self.lines.push(Vec::new());
        }

        deleted
    }

    fn apply_insert(&mut self, start: CursorPos, text: &str) -> CursorPos {
        let end = self.insert_text_raw(start, text);
        self.current_index = end.0;
        self.line_index = end.1;
        self.clear_selection();
        self.ensure_invariants();
        end
    }

    fn apply_delete(&mut self, start: CursorPos, end: CursorPos) -> String {
        let deleted = self.delete_range_raw(start, end);
        self.current_index = start.0;
        self.line_index = start.1;
        self.clear_selection();
        self.ensure_invariants();
        deleted
    }

    fn push_command(&mut self, command: EditCommand) {
        self.undo_stack.push(command);
        self.redo_stack.clear();
    }

    fn apply_command(&mut self, command: &EditCommand) {
        match command.kind {
            EditKind::Insert => {
                self.apply_insert(command.start, &command.text);
            }
            EditKind::Delete => {
                let end = Self::position_after_text(command.start, &command.text);
                self.apply_delete(command.start, end);
            }
        }

        self.current_index = command.after_cursor.0;
        self.line_index = command.after_cursor.1;
        self.selection_start = command.after_selection.0;
        self.selection_end = command.after_selection.1;
        self.ensure_invariants();
    }

    fn apply_inverse_command(&mut self, command: &EditCommand) {
        match command.kind {
            EditKind::Insert => {
                let end = Self::position_after_text(command.start, &command.text);
                self.apply_delete(command.start, end);
            }
            EditKind::Delete => {
                self.apply_insert(command.start, &command.text);
            }
        }

        self.current_index = command.before_cursor.0;
        self.line_index = command.before_cursor.1;
        self.selection_start = command.before_selection.0;
        self.selection_end = command.before_selection.1;
        self.ensure_invariants();
    }

    fn record_insert(&mut self, start: CursorPos, text: String, before_cursor: CursorPos, before_selection: Selection) {
        let after_cursor = self.cursor();
        let after_selection = self.selection();
        self.push_command(EditCommand {
            kind: EditKind::Insert,
            start,
            text,
            before_cursor,
            before_selection,
            after_cursor,
            after_selection,
        });
    }

    fn record_delete(&mut self, start: CursorPos, text: String, before_cursor: CursorPos, before_selection: Selection) {
        let after_cursor = self.cursor();
        let after_selection = self.selection();
        self.push_command(EditCommand {
            kind: EditKind::Delete,
            start,
            text,
            before_cursor,
            before_selection,
            after_cursor,
            after_selection,
        });
    }

    pub fn undo(&mut self) -> bool {
        let Some(command) = self.undo_stack.pop() else {
            return false;
        };

        self.apply_inverse_command(&command);
        self.redo_stack.push(command);
        true
    }

    pub fn redo(&mut self) -> bool {
        let Some(command) = self.redo_stack.pop() else {
            return false;
        };

        self.apply_command(&command);
        self.undo_stack.push(command);
        true
    }

    pub fn select_all(&mut self) -> bool {
        if self.lines.is_empty() {
            self.clear_selection();
            return false;
        }

        let last_line = self.lines.len() - 1;
        let last_index = self.lines[last_line].len();

        if last_line == 0 && last_index == 0 {
            self.clear_selection();
            return false;
        }

        self.selection_start = (0, 0);
        self.selection_end = (last_index, last_line);
        true
    }

    pub fn selection_range(&self) -> Option<(CursorPos, CursorPos)> {
        let start = self.selection_start;
        let end = self.selection_end;

        if start == end {
            return None;
        }

        if syx_order(start, end) {
            Some((start, end))
        } else {
            Some((end, start))
        }
    }

    pub fn has_selection(&self) -> bool {
        self.selection_range().is_some()
    }

    pub fn clear_selection(&mut self) {
        self.selection_start = (0, 0);
        self.selection_end = (0, 0);
    }

    pub fn is_selected(&self, x: usize, line: usize) -> bool {
        let Some((start, end)) = self.selection_range() else {
            return false;
        };

        if line < start.1 || line > end.1 {
            return false;
        }

        if start.1 == end.1 {
            return line == start.1 && x >= start.0 && x < end.0;
        }

        if line == start.1 {
            return x >= start.0;
        }

        if line == end.1 {
            return x < end.0;
        }

        true
    }

    fn selected_text(&self) -> Option<String> {
        let Some((start, end)) = self.selection_range() else {
            return None;
        };

        let mut text = String::new();

        if start.1 == end.1 {
            text.extend(self.lines[start.1][start.0..end.0].iter().copied());
            return Some(text);
        }

        text.extend(self.lines[start.1][start.0..].iter().copied());
        text.push('\n');

        for line in (start.1 + 1)..end.1 {
            text.extend(self.lines[line].iter().copied());
            text.push('\n');
        }

        text.extend(self.lines[end.1][..end.0].iter().copied());
        Some(text)
    }

    fn delete_selection(&mut self) -> bool {
        let Some((start, end)) = self.selection_range() else {
            return false;
        };

        let before_cursor = self.cursor();
        let before_selection = self.selection();
        let deleted = self.apply_delete(start, end);
        self.record_delete(start, deleted, before_cursor, before_selection);
        true
    }

    pub fn copy_selection(&self) -> bool {
        let Some(text) = self.selected_text() else {
            return false;
        };

        let Ok(mut clipboard) = ClipboardContext::new() else {
            return false;
        };

        clipboard.set_contents(text).is_ok()
    }

    pub fn cut_selection(&mut self) -> bool {
        if !self.copy_selection() {
            return false;
        }

        self.delete_selection()
    }

    pub fn paste_text(&mut self, text: &str) -> bool {
        let text = Self::normalize_newlines(text);
        if text.is_empty() {
            return false;
        }

        let before_cursor = self.cursor();
        let before_selection = self.selection();

        if self.has_selection() {
            let Some((start, end)) = self.selection_range() else {
                return false;
            };
            let deleted = self.apply_delete(start, end);
            self.record_delete(start, deleted, before_cursor, before_selection);
        }

        let insert_start = self.cursor();
        self.apply_insert(insert_start, &text);
        self.record_insert(insert_start, text, before_cursor, before_selection);
        true
    }

    pub fn paste_from_clipboard_text(&mut self) -> Option<String> {
        let Ok(mut clipboard) = ClipboardContext::new() else {
            return None;
        };
        let Ok(text) = clipboard.get_contents() else {
            return None;
        };
        let text = Self::normalize_newlines(&text);

        self.paste_text(&text).then_some(text)
    }

    pub fn paste_from_clipboard(&mut self) -> bool {
        self.paste_from_clipboard_text().is_some()
    }
}

impl TextBuf {
    pub fn add_tab(&mut self) {
        let before_cursor = self.cursor();
        let before_selection = self.selection();

        if self.has_selection() {
            let Some((start, end)) = self.selection_range() else {
                return;
            };
            let deleted = self.apply_delete(start, end);
            self.record_delete(start, deleted, before_cursor, before_selection);
        }

        let start = self.cursor();
        let text = " ".repeat(TAB_WIDTH);
        self.apply_insert(start, &text);
        self.record_insert(start, text, before_cursor, before_selection);
    }
}

impl TextBuf {
    pub fn change_cursor_horizontal(&mut self, dir: i32) {
        if (dir < 0){
            if (self.current_index > 0){
                self.current_index -= 1;
            }else{
                if (self.line_index > 0){
                    self.line_index -= 1;

                    if !self.lines[self.line_index].is_empty() {
                        self.current_index = self.lines[self.line_index].len();
                    }else{
                        self.current_index = 0;
                    }
                }
            }
        }
        if (dir > 0){
            if (self.current_index + 1 > self.lines[self.line_index].len()){
                if (self.line_index + 1 < self.lines.len()){
                    self.line_index += 1;
                    self.current_index = 0;
                }
            }else{
                self.current_index += 1;
            }
        }
    }

    pub fn change_cursor_vertical(&mut self, dir: i32) {
        let line: i32 = self.line_index as i32 + dir;

        if line < 0 {
            return;
        }else if line >= self.lines.len() as i32 {
            return;
        }

        self.line_index = line as usize;

        if self.lines[self.line_index].len() < self.current_index {
            self.current_index = self.lines[self.line_index].len();
        };
    }
}

impl TextBuf {
    pub fn get_current_line(&self) -> &Vec<char> {
        &self.lines[self.line_index]
    }

    pub fn add_char(&mut self, char: char) {
        let before_cursor = self.cursor();
        let before_selection = self.selection();

        if self.has_selection() {
            let Some((start, end)) = self.selection_range() else {
                return;
            };
            let deleted = self.apply_delete(start, end);
            self.record_delete(start, deleted, before_cursor, before_selection);
        }

        let start = self.cursor();
        let text = char.to_string();
        self.apply_insert(start, &text);
        self.record_insert(start, text, before_cursor, before_selection);
    }

    pub fn remove_char_delete(&mut self) -> bool {
        let before_cursor = self.cursor();
        let before_selection = self.selection();

        if self.has_selection() {
            return self.delete_selection();
        }

        if self.lines.is_empty() { return false; }
        if self.line_index >= self.lines.len() { return false; }

        let len = self.lines[self.line_index].len();
        if self.current_index > len {
            self.current_index = len;
        }

        if self.current_index < len {
            if self.current_index + TAB_WIDTH <= len {
                let mut is_tab = true;
                for i in 0..TAB_WIDTH {
                    if self.lines[self.line_index][self.current_index + i] != ' ' {
                        is_tab = false;
                        break;
                    }
                }
                if is_tab {
                    let start = self.cursor();
                    let end = (self.current_index + TAB_WIDTH, self.line_index);
                    let deleted = self.apply_delete(start, end);
                    self.record_delete(start, deleted, before_cursor, before_selection);
                    return false;
                }
            }

            let start = self.cursor();
            let end = (self.current_index + 1, self.line_index);
            let deleted = self.apply_delete(start, end);
            self.record_delete(start, deleted, before_cursor, before_selection);
            return false;
        }

        if self.line_index + 1 < self.lines.len() {
            let start = self.cursor();
            let end = (0, self.line_index + 1);
            let deleted = self.apply_delete(start, end);
            self.record_delete(start, deleted, before_cursor, before_selection);
            return true;
        }

        false
    }

    pub fn remove_char_backspace(&mut self) -> bool {
        let before_cursor = self.cursor();
        let before_selection = self.selection();

        if self.has_selection() {
            return self.delete_selection();
        }

        if self.lines[self.line_index].is_empty() {
            if self.line_index > 0 {
                let prev_line = self.line_index - 1;
                let prev_len = self.lines[prev_line].len();
                let start = (prev_len, prev_line);
                let end = (0, self.line_index);
                let deleted = self.apply_delete(start, end);
                self.record_delete(start, deleted, before_cursor, before_selection);
                return true;
            }
            return false;
        }

        if self.current_index >= TAB_WIDTH {
            let mut is_tab = true;
            for i in 0..TAB_WIDTH {
                if self.lines[self.line_index][self.current_index - i - 1] != ' ' {
                    is_tab = false;
                    break;
                }
            }

            if is_tab {
                let start = (self.current_index - TAB_WIDTH, self.line_index);
                let end = (self.current_index, self.line_index);
                let deleted = self.apply_delete(start, end);
                self.record_delete(start, deleted, before_cursor, before_selection);
                return false;
            }
        }

        if self.current_index > 0 {
            let start = (self.current_index - 1, self.line_index);
            let end = (self.current_index, self.line_index);
            let deleted = self.apply_delete(start, end);
            self.record_delete(start, deleted, before_cursor, before_selection);
            return false;
        }

        self.change_cursor_horizontal(-1);
        false
    }

    pub fn add_line(&mut self) {
        let before_cursor = self.cursor();
        let before_selection = self.selection();

        if self.has_selection() {
            let Some((start, end)) = self.selection_range() else {
                return;
            };
            let deleted = self.apply_delete(start, end);
            self.record_delete(start, deleted, before_cursor, before_selection);
        }

        let start = self.cursor();
        let text = "\n".to_string();
        self.apply_insert(start, &text);
        self.record_insert(start, text, before_cursor, before_selection);
    }
}

impl Default for TextBuf {
    fn default() -> Self {
        Self{
            lines: vec![
                Vec::new(),
            ],
            current_index: 0,
            line_index: 0,
            selection_start: (0, 0),
            selection_end: (0, 0),
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            scroll_x: 0,
            scroll_y: 0,
        }
    }
}

fn syx_order(a: CursorPos, b: CursorPos) -> bool {
    a.1 < b.1 || (a.1 == b.1 && a.0 <= b.0)
}
