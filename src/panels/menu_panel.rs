use crate::controls::c_button::Button;
use crate::controls::c_text::TextBox;
use crate::controls::t_get_rect::Control;
use crate::controls::t_render::Render;
use crate::e_actions::Action;
use crate::input::Input;
use crate::logger::FileLogger;
use crate::panels::pop_up_panel::PopUpPanelFrame;
use crate::screen_buf::ScreenBuf;
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

    fn create_layout(&mut self, layout: &mut Layout){
        let frame = Frame::new(EFrameAxis::Horizontal, false);
        let mut open_frame = layout.open_frame(frame);

        let mut fileBtn = Button::new(" FILE ");
        fileBtn.create_control(open_frame);
        let mut delimiter1 = TextBox::new("│");
        delimiter1.create_control(open_frame);
        let mut editBtn = Button::new(" EDIT ");
        editBtn.create_control(open_frame);
        let mut delimiter2 = TextBox::new("│");
        delimiter2.create_control(open_frame);
        let mut infoBtn = Button::new(" INFO ");
        infoBtn.create_control(open_frame);


        open_frame.add_control(Box::new(delimiter1));
        open_frame.add_control(Box::new(delimiter2));

        let frame_id = open_frame.frame_id;

        layout.close_frame();

        self.file_button = Some(fileBtn);
        self.edit_button = Some(editBtn);
        self.info_button = Some(infoBtn);
        self.frame = frame_id;
    }
    fn interact(&mut self, file_logger: &mut FileLogger, input: &Input, pop_pup: &mut PopUpPanelFrame) -> Action {

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
                    (" Undo ".to_string(), Action::NewFile),
                    (" Redo".to_string(), Action::OpenFile),
                    (" Cut".to_string(), Action::SaveFile),
                    (" Copy".to_string(), Action::SaveFile),
                    (" Paste".to_string(), Action::SaveFile),
                    (" Find".to_string(), Action::SaveFile),
                    (" Replace".to_string(), Action::SaveFile),
                ], file_logger, input)
            }
        }
        
        if let Some(btn) = &mut self.info_button {
            btn.calculate_control(file_logger, input);
            if (btn.clicked()) {
                pop_pup.show(vec![
                    (" FAQ ".to_string(), Action::NewFile),
                ], file_logger, input)
            }
        }


        Action::None
    }
    fn draw(&mut self, layout: &mut Layout, screen: &mut ScreenBuf) {
        let open_frame = layout.get_frame(self.frame).unwrap();

        open_frame.add_control(Box::new(self.file_button.take().unwrap()));
        open_frame.add_control(Box::new(self.edit_button.take().unwrap()));
        open_frame.add_control(Box::new(self.info_button.take().unwrap()));


        open_frame.draw(&Rect::default(), screen);
    }
}

pub trait LayoutPanel {
    fn get_order(&self) -> u16;
    fn create_layout(&mut self, layout: &mut Layout);
    fn interact(&mut self, file_logger: &mut FileLogger, input: &Input, pop_pup: &mut PopUpPanelFrame) -> Action;
    fn draw(&mut self, layout: &mut Layout, screen: &mut ScreenBuf);
}