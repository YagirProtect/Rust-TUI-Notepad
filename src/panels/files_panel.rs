use std::path::PathBuf;
use crate::config::Config;
use crate::controls::c_button::Button;
use crate::controls::t_get_rect::Control;
use crate::controls::t_render::Render;
use crate::e_actions::Action;
use crate::input::Input;
use crate::logger::FileLogger;
use crate::panels::menu_panel::LayoutPanel;
use crate::panels::pop_up_panel::PopUpPanelFrame;
use crate::screen_buffer::ScreenBuf;
use crate::text_buffer::TextBuf;
use crate::ui::c_frame::{EFrameAxis, Frame};
use crate::ui::c_layout::Layout;
use crate::ui::c_rect::Rect;

#[derive(Default)]
pub struct FilesFrame {

    buttons: Vec<Button>,
    frame: u16,
}

impl LayoutPanel for FilesFrame {
    fn get_order(&self) -> u16 {
        0
    }

    fn get_frame_id(&self) -> u16 {
        return self.frame;
    }

    fn create_layout(&mut self, layout: &mut Layout, config: &mut Config) {
        let frame = Frame::new(EFrameAxis::Vertical, false);
        let mut open_frame = layout.open_frame(frame);
        self.frame = open_frame.frame_id;


        self.buttons.clear();

        let max_len = 18;
        for get_last_file in config.get_last_files() {
            let mut str = String::from(" ");

            let get_last_file = PathBuf::from(get_last_file.clone());
            if (get_last_file.exists()) {
                if (get_last_file.file_name().is_some()) {
                    let mut file_name = get_last_file.file_name().unwrap().to_str().unwrap().to_string();

                    if (file_name.len() > max_len) {
                        file_name = file_name[0..max_len - 3].to_string();
                        file_name.push_str("... ");
                    } else {
                        file_name.push_str(vec![' '; max_len - file_name.len()].iter().collect::<String>().as_str());
                    }

                    str.push_str(file_name.as_str());

                    let mut btn = Button::new(str.as_str());
                    btn.create_control(open_frame);
                    self.buttons.push(btn);
                }
            }
        }
        layout.close_frame();
    }

    fn interact(&mut self, file_logger: &mut FileLogger, input: &mut Input, pop_pup: &mut PopUpPanelFrame, text_buf: &mut TextBuf) -> Action {
        for button in self.buttons.iter_mut() {
            button.calculate_control(file_logger, input);
            if (button.clicked()){

            }
        }




        Action::None
    }

    fn draw(&mut self, layout: &mut Layout, screen: &mut ScreenBuf, text_buf: &mut TextBuf) {
        let frame = layout.get_frame(self.frame).unwrap();
        for button in self.buttons.drain(..) {
            frame.add_control(Box::new(button));
        }
        frame.draw(&Rect::default(), screen);
    }


}

impl FilesFrame {

}