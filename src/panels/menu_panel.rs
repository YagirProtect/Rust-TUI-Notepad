use crate::config::Config;
use crate::controls::c_button::Button;
use crate::controls::c_delimiter::Delimiter;
use crate::controls::c_text::TextBox;
use crate::controls::t_get_rect::Control;
use crate::controls::t_render::Render;
use crate::e_actions::Action;
use crate::input::Input;
use crate::logger::FileLogger;
use crate::panels::pop_up_panel::PopUpPanelFrame;
use crate::screen_buf::ScreenBuf;
use crate::text_buffer::TextBuf;
use crate::ui::c_frame::{EFrameAxis, Frame};
use crate::ui::c_layout::Layout;
use crate::ui::c_rect::Rect;

#[derive(Default)]
pub struct MenuFrame{
    file_button: Option<Button>,
    edit_button: Option<Button>,
    info_button: Option<Button>,

    frame: u16
}

impl MenuFrame {

}

impl LayoutPanel for MenuFrame {
    fn get_order(&self) -> u16 {
        0
    }

    fn get_frame_id(&self) -> u16 {
        return self.frame;
    }

    fn create_layout(&mut self, layout: &mut Layout, config: &mut Config){
        let frame = Frame::new(EFrameAxis::Horizontal, false);
        let mut open_frame = layout.open_frame(frame);

        let mut fileBtn = Button::new(" FILE ");
        fileBtn.create_control(open_frame);
        let mut delimiter1 = Delimiter::new();
        delimiter1.create_control(open_frame);
        let mut editBtn = Button::new(" EDIT ");
        editBtn.create_control(open_frame);
        let mut delimiter2 = Delimiter::new();
        delimiter2.create_control(open_frame);
        let mut infoBtn = Button::new(" INFO ");
        infoBtn.create_control(open_frame);
        let mut delimiter3 = Delimiter::new();
        delimiter3.create_control(open_frame);

        open_frame.add_control(Box::new(delimiter1));
        open_frame.add_control(Box::new(delimiter2));
        open_frame.add_control(Box::new(delimiter3));

        let frame_id = open_frame.frame_id;

        layout.close_frame();

        self.file_button = Some(fileBtn);
        self.edit_button = Some(editBtn);
        self.info_button = Some(infoBtn);
        self.frame = frame_id;
    }
    fn interact(&mut self, file_logger: &mut FileLogger, input: &mut Input, pop_pup: &mut PopUpPanelFrame, text_buf: &mut TextBuf) -> Action {

        if let Some(btn) = &mut self.file_button {

            btn.calculate_control(file_logger, input);

            if (btn.clicked()) {
                pop_pup.show(vec![
                    (" New File.. ".to_string(), Action::NewFile),
                    (" Open File..".to_string(), Action::OpenFile),
                    (" Save File..".to_string(), Action::SaveFile),
                ], file_logger, input)
            }
        }


        if let Some(btn) = &mut self.edit_button {
            btn.calculate_control(file_logger, input);
            if (btn.clicked()) {
                pop_pup.show(vec![
                    (" Undo ".to_string(), Action::Undo),
                    (" Redo".to_string(), Action::Redo),
                    (" Cut".to_string(), Action::Cut),
                    (" Copy".to_string(), Action::Copy),
                    (" Paste".to_string(), Action::Paste),
                    (" Find".to_string(), Action::Find),
                    (" Replace".to_string(), Action::Replace),
                ], file_logger, input)
            }
        }
        
        if let Some(btn) = &mut self.info_button {
            btn.calculate_control(file_logger, input);
            if (btn.clicked()) {
                pop_pup.show(vec![
                    (" FAQ ".to_string(), Action::FAQ),
                ], file_logger, input)
            }
        }


        Action::None
    }
    fn draw(&mut self, layout: &mut Layout, screen: &mut ScreenBuf, text_buf: &mut TextBuf) {
        let open_frame = layout.get_frame(self.frame).unwrap();

        open_frame.add_control(Box::new(self.file_button.take().unwrap()));
        open_frame.add_control(Box::new(self.edit_button.take().unwrap()));
        open_frame.add_control(Box::new(self.info_button.take().unwrap()));


        open_frame.draw(&Rect::default(), screen);
    }
}

pub trait LayoutPanel {
    fn get_order(&self) -> u16;
    fn get_frame_id(&self) -> u16;
    fn create_layout(&mut self, layout: &mut Layout, config: &mut Config);
    fn interact(&mut self, file_logger: &mut FileLogger, input: &mut Input, pop_pup: &mut PopUpPanelFrame, text_buf: &mut TextBuf) -> Action;
    fn draw(&mut self, layout: &mut Layout, screen: &mut ScreenBuf, text_buf: &mut TextBuf);
    fn try_hit(&mut self, layout: &mut Layout, input: &Input) -> bool {
        let frame = layout.get_frame(self.get_frame_id()).unwrap();
        return frame.hit(input.cursor_x, input.cursor_y);
    }
}