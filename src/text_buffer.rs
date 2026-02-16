pub struct TextBuf{
    pub lines: Vec<Vec<char>>,
    pub current_index: usize,
    pub line_index: usize,

    scroll_x: u16,
    scroll_y: u16,
}
impl TextBuf {
    pub fn get_current_line(&self) -> &Vec<char> {
        &self.lines[self.line_index]
    }

    pub fn add_char(&mut self, char: char) {
        self.lines[self.line_index].insert(self.current_index, char);
        self.current_index += 1;
    }

    pub fn remove_char(&mut self) -> bool {
        if (self.lines[self.line_index].len() == 0){
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
        }else{
            if (self.current_index > 0) {
                self.lines[self.line_index].remove(self.current_index - 1);
                self.current_index -= 1;
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