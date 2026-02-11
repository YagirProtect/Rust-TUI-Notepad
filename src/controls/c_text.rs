use crossterm::style::Stylize;
use crate::controls::t_get_rect::{Control, GetRect};
use crate::controls::t_render::Render;
use crate::screen_buf::{Color, ScreenBuf};
use crate::ui::c_rect::Rect;

pub struct TextBox{
    text: Vec<char>,
    color: Color,
}

impl TextBox {
    pub fn set_color(&mut self, p0: Color) {
        self.color = p0;
    }
}

impl GetRect for TextBox {
    fn get_bounds(&self) -> Rect {
        return Rect::new(0, 0, self.text.len() as u16, 1);
    }
}

impl Control for TextBox {
}

impl Render for TextBox {
    fn draw(&self, rect: &Rect, screen: &mut ScreenBuf) {
        let bounds = self.get_bounds();





        for i in 0..bounds.w {
            screen.set(rect.x + i, rect.y,  self.text[i as usize], self.color);
        }
    }
}

impl TextBox{
    pub fn new(text: &str) -> TextBox{
        TextBox{
            text: String::from(text).chars().collect(),
            color: Color::White,
        }
    }
}