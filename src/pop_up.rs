use log::log;
use crate::controls::c_button::Button;
use crate::controls::t_get_rect::Control;
use crate::controls::t_render::Render;
use crate::e_actions::Action;
use crate::input::Input;
use crate::logger::FileLogger;
use crate::screen_buf::ScreenBuf;
use crate::ui::c_frame::{EFrameAxis, Frame};
use crate::ui::c_rect::Rect;

pub struct PopUpPanelFrame{
    items: Vec<(String, Action)>,

    active: bool,
    is_free: bool,
    pos: (u16, u16),


    frame: Frame
}


impl PopUpPanelFrame{
    pub fn new() -> PopUpPanelFrame{
        Self{
            items: vec![],

            active: false,
            pos: (0, 0),
            is_free: false,
            frame: Frame::new(EFrameAxis::Vertical, true)
        }
    }


    pub fn show(&mut self, items: Vec<(String, Action)>, file_logger: &mut FileLogger){
        self.items = items;
        self.active = true;
        self.is_free = false;

        file_logger.log("show popup");
    }


    pub fn draw(&mut self, screen_buff: &mut ScreenBuf, file_logger: &mut FileLogger, input: &Input) -> Action{
        if (!self.active){
            return Action::None;
        }
        if (input.clicked.is_some() && self.is_free){
            file_logger.log(format!("clicked: {:?}", input.clicked));
            file_logger.log(format!("rect: {:?}", self.frame.content_rect()));
            if !self.frame.hit(input.cursor_x, input.cursor_y)
            {
                file_logger.log("not contains in rect");
                self.active = false;
                return Action::None;
            }
        }
        self.frame.set_area(Rect::new(self.pos.0 + 3, self.pos.1 + 2, 2, self.items.len() as u16));

        self.frame.get_available_rect();




        file_logger.log("draw popup");

        let mut target_action = Action::None;
        for (str, ac) in self.items.iter() {
            if (Button::new(str.as_str()).create_control(&mut self.frame, screen_buff, file_logger, input).clicked()){
                target_action = *ac;
                self.active = false;
                file_logger.log(format!("clicked: {:?}", str));
            }
        }
        self.frame.draw(&self.frame.area, screen_buff);
        self.is_free = true;

        return target_action;
    }
}


