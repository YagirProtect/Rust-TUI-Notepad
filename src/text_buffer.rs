pub struct TextBuf{
    pub lines: Vec<Vec<char>>,
    pub current_index: usize,
    pub line_index: usize,

    scroll_x: u16,
    scroll_y: u16,
}

const TAB_WIDTH: usize = 4;

impl TextBuf {
    pub fn add_tab(&mut self) {
        for i in 0..TAB_WIDTH {
            self.add_char(' ');
        }
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

                    if (self.lines[self.line_index].len() > 0) {
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

        if (line < 0){
            return;
        }else if (line >= self.lines.len() as i32){
            return;
        }

        self.line_index = line as usize;

        if (self.lines[self.line_index].len() < self.current_index){
            if (self.lines[self.line_index].len() > 0) {
                self.current_index = self.lines[self.line_index].len();
            }else{
                self.current_index = 0;
            }
        };
    }
}

impl TextBuf {
    pub fn get_current_line(&self) -> &Vec<char> {
        &self.lines[self.line_index]
    }

    pub fn add_char(&mut self, char: char) {
        self.lines[self.line_index].insert(self.current_index, char);
        self.current_index += 1;
    }

    pub fn remove_char_delete(&mut self) -> bool {
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
                    for _ in 0..TAB_WIDTH {
                        self.lines[self.line_index].remove(self.current_index); // курсор не двигаем
                    }
                    return false;
                }
            }

            self.lines[self.line_index].remove(self.current_index);
            return false;
        }
        if self.line_index + 1 < self.lines.len() {
            let next = self.lines.remove(self.line_index + 1);
            self.lines[self.line_index].extend(next);
            return true;
        }

        false
    }

    pub fn remove_char_backspace(&mut self) -> bool {
        if (self.lines[self.line_index].len() == 0) {
            if (self.line_index > 0) {
                self.lines.remove(self.line_index);
                self.line_index -= 1;

                let len = self.lines[self.line_index].len();
                if (len > 0) {
                    self.lines[self.line_index].remove(len - 1);
                    self.current_index = self.lines[self.line_index].len();
                }
                return true;
            }
        } else {
            if (self.current_index >= TAB_WIDTH) {
                let mut is_tab = true;
                for i in 0..TAB_WIDTH {
                    if (self.lines[self.line_index][self.current_index - i - 1] != ' ') {
                        is_tab = false;
                        break;
                    }
                }

                if (is_tab) {
                    for i in 0..TAB_WIDTH {
                        self.lines[self.line_index].remove(self.current_index - 1);
                        self.current_index -= 1;
                    }
                    return false;
                }
            }
            if (self.current_index > 0) {
                self.lines[self.line_index].remove(self.current_index - 1);
                self.current_index -= 1;
            }else{
                self.change_cursor_horizontal(-1);
            }
        }

        return false;
    }

    pub fn add_line(&mut self) {
        if self.lines.is_empty() {
            self.lines.push(Vec::new());
            self.line_index = 0;
            self.current_index = 0;
        }

        if self.line_index >= self.lines.len() {
            self.line_index = self.lines.len() - 1;
        }

        let len = self.lines[self.line_index].len();
        if self.current_index > len {
            self.current_index = len;
        }
        let right = self.lines[self.line_index].split_off(self.current_index);
        self.lines.insert(self.line_index + 1, right);

        self.line_index += 1;
        self.current_index = 0;
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
            scroll_x: 0,
            scroll_y: 0,
        }
    }
}