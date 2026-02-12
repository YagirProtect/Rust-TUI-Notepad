use crate::screen_buf::ScreenBuf;
use crate::ui::c_rect::Rect;

pub trait Render{ 
    fn draw(&mut self, rect: &Rect, screen: &mut ScreenBuf);
}