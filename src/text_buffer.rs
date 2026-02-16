pub struct TextBuf{
    pub lines: Vec<Vec<char>>,
}

impl Default for TextBuf {
    fn default() -> Self {
        Self{
            lines: vec![
                Vec::new(),
            ],
        }
    }
}