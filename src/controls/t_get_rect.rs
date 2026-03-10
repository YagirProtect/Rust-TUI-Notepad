use crate::controls::t_render::Render;
use crate::input::Input;
use crate::logger::FileLogger;
use crate::screen_buffer::ScreenBuf;
use crate::ui::c_frame::Frame;
use crate::ui::c_rect::Rect;

pub trait GetRect{
    fn get_bounds(&self) -> Rect;

}
pub trait Control: GetRect + Render{

    fn create_control(&mut self, frame: &mut Frame)
    where
        Self: Sized
    {
        let rect = frame.add_layout_changes(self.get_bounds());
        self.save_rect(rect);
    }

    fn get_rect(&self) -> &Rect;
    fn save_rect(&mut self, rect: Rect);
    fn calculate_control(&mut self, logger: &mut FileLogger, input: &Input){

    }
}

