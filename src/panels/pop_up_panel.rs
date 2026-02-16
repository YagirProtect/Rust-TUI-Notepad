use log::log;
use crate::config::Config;
use crate::controls::c_button::Button;
use crate::controls::t_get_rect::Control;
use crate::controls::t_render::Render;
use crate::e_actions::Action;
use crate::input::Input;
use crate::logger::FileLogger;
use crate::panels::menu_panel::LayoutPanel;
use crate::screen_buf::ScreenBuf;
use crate::text_buffer::TextBuf;
use crate::ui::c_frame::{EFrameAxis, Frame};
use crate::ui::c_layout::Layout;
use crate::ui::c_rect::Rect;

pub struct PopUpPanelFrame{
    items: Vec<(String, Action)>,

    pub active: bool,
    is_free: bool,
    pos: (u16, u16),

    pub buttons: Vec<Button>,

    frame: Frame,

    show: bool,

    is_clicked: bool,
}

impl LayoutPanel for PopUpPanelFrame {
    fn get_order(&self) -> u16 {
        999
    }

    fn get_frame_id(&self) -> u16 {
        return self.frame.frame_id;
    }

    fn try_hit(&mut self, layout: &mut Layout, input: &Input) -> bool {

        return self.frame.hit(input.cursor_x, input.cursor_y);
    }

    fn create_layout(&mut self, layout: &mut Layout, config: &mut Config) {
        if (self.active){
            self.frame = Frame::new(EFrameAxis::Vertical, true);
            self.frame.set_area(Rect::new(self.pos.0 + 1, self.pos.1 + 1, 2, self.items.len() as u16));
            self.frame.get_available_rect();


            self.buttons.clear();
            for (str, ac) in self.items.iter() {
                let mut btn = Button::new(str.as_str());
                btn.create_control(&mut self.frame);
                self.buttons.push(btn);
            }
        }
    }

    fn interact(&mut self, file_logger: &mut FileLogger, input: &mut Input, pop_pup: &mut PopUpPanelFrame, text_buf: &mut TextBuf) -> Action {

        if (!self.active){
            return Action::None;
        }

        if (input.clicked.is_some() && self.is_free){
            if !self.frame.hit(input.cursor_x, input.cursor_y)
            {
                file_logger.log("not contains in rect");
                self.active = false;
                self.is_clicked = false;
                return Action::None;
            }
        }else if (input.clicked.is_none() && self.is_clicked){
            self.active = false;
            self.is_clicked = false;
        }


        let mut id = 0;
        for btn in self.buttons.iter_mut() {
            btn.calculate_control(file_logger, input);
            if (btn.clicked()){
                self.is_clicked = true;

                return self.items[id].1;
            }
            id += 1;
        }

        self.is_free = true;


        return Action::None;
    }

    fn draw(&mut self, layout: &mut Layout, screen: &mut ScreenBuf, text_buf: &mut TextBuf) {

        if (!self.active){
            return;
        }
        if (self.show) {
            for button in self.buttons.drain(..) {
                self.frame.add_control(Box::new(button));
            }
            self.frame.draw(&Rect::default(), screen);
        }
        self.show = true;
    }
}

impl PopUpPanelFrame{
    pub fn new() -> PopUpPanelFrame{
        Self{
            items: vec![],

            active: false,
            pos: (0, 0),
            is_free: false,
            frame: Frame::new(EFrameAxis::Vertical, true),
            buttons: vec![],
            is_clicked: false,
            show: false,
        }
    }


    pub fn show(&mut self, items: Vec<(String, Action)>, file_logger: &mut FileLogger, input: &Input){
        self.items = items;
        self.active = true;
        self.is_free = false;
        self.show = false;
        file_logger.log(format!("show {}", self.items.len()));
        self.pos = (input.cursor_x, input.cursor_y);
    }
}


