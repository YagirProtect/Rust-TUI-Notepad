use crate::controls::t_get_rect::{Control, GetRect};
use crate::controls::t_render::Render;
use crate::screen_buf::{Color, ScreenBuf};
use crate::ui::c_rect::Rect;

#[derive(Debug)]
pub struct Delimiter{
    color: Color,
    rect: Rect,
}

impl Delimiter {
    pub fn set_color(&mut self, p0: Color) {
        self.color = p0;
    }
}

impl GetRect for Delimiter {
    fn get_bounds(&self) -> Rect {
        return Rect::new(1, 0, 1, 1);
    }
}

impl Control for Delimiter {
    fn get_rect(&self) -> &Rect {
        return &self.rect;
    }
    fn save_rect(&mut self, rect: Rect) {
        self.rect = rect;
    }
}

impl Render for Delimiter {
    fn draw(&mut self, rect: &Rect, screen: &mut ScreenBuf) {
        screen.set(rect.x, rect.y-1,  '┬', self.color);
        screen.set(rect.x, rect.y,  '│', self.color);
        screen.set(rect.x, rect.y+1,  '┴', self.color);
    }
}

impl Delimiter{
    pub fn new() -> Delimiter{
        Delimiter{
            color: Color::Blue,
            rect: Rect::default()
        }
    }
}