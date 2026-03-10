use crate::screen_buffer::ScreenBuf;
use crate::ui::c_rect::Rect;

pub trait Render{ 
    fn draw(&mut self, rect: &Rect, screen: &mut ScreenBuf);
}