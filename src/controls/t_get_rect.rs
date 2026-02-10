use crate::screen_buf::ScreenBuf;
use crate::ui::c_frame::Frame;
use crate::ui::c_rect::Rect;

pub trait GetRect{
    fn get_bounds(&self) -> Rect;

    fn create_control(&mut self, frame: &mut Frame, screen: &mut ScreenBuf);
}

