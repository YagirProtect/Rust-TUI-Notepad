use crate::controls::t_render::Render;
use crate::input::Input;
use crate::logger::FileLogger;
use crate::screen_buf::ScreenBuf;
use crate::ui::c_frame::Frame;
use crate::ui::c_rect::Rect;

pub trait GetRect{
    fn get_bounds(&self) -> Rect;

}
pub trait Control: GetRect + Render{

    fn create_control(&mut self, frame: &mut Frame, screen: &mut ScreenBuf, logger: &mut FileLogger, input: &Input) -> &Self {
        let rect = frame.add(self.get_bounds());
        self.calculate_control(rect, logger, input); 
        self.draw(&rect, screen);
        self
    }

    fn calculate_control(&mut self, rect: Rect, logger: &mut FileLogger, input: &Input){

    }
}

