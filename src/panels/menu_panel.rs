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
use crate::screen_buffer::ScreenBuf;
use crate::text_buffer::TextBuf;
use crate::ui::c_frame::{EFrameAxis, Frame};
use crate::ui::c_layout::Layout;
use crate::ui::c_rect::Rect;

#[derive(Default)]
pub struct MenuFrame{
    file_button: Option<Button>,
    edit_button: Option<Button>,
    view_button: Option<Button>,
    info_button: Option<Button>,
    file_items: Vec<(String, Action)>,
    edit_items: Vec<(String, Action)>,
    view_items: Vec<(String, Action)>,
    info_items: Vec<(String, Action)>,

    frame: u16
}

impl MenuFrame {
    fn menu_item(label: &str, shortcut: String, action: Action) -> (String, Action) {
        if shortcut.is_empty() {
            (format!(" {}", label), action)
        } else {
            (format!(" {:<20} {}", label, shortcut), action)
        }
    }

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
        let mut viewBtn = Button::new(" VIEW ");
        viewBtn.create_control(open_frame);
        let mut delimiter_view = Delimiter::new();
        delimiter_view.create_control(open_frame);
        let mut infoBtn = Button::new(" INFO ");
        infoBtn.create_control(open_frame);
        let mut delimiter3 = Delimiter::new();
        delimiter3.create_control(open_frame);

        open_frame.add_control(Box::new(delimiter1));
        open_frame.add_control(Box::new(delimiter2));
        open_frame.add_control(Box::new(delimiter_view));
        open_frame.add_control(Box::new(delimiter3));

        let frame_id = open_frame.frame_id;

        layout.close_frame();

        self.file_button = Some(fileBtn);
        self.edit_button = Some(editBtn);
        self.view_button = Some(viewBtn);
        self.info_button = Some(infoBtn);
        self.file_items = vec![
            Self::menu_item("New File..", config.shortcuts_label_for("new_file"), Action::NewFile),
            Self::menu_item("Open File..", config.shortcuts_label_for("open_file"), Action::OpenFile),
            Self::menu_item("Save File..", config.shortcuts_label_for("save_file"), Action::SaveFile),
            Self::menu_item("Save As..", config.shortcuts_label_for("save_file_as"), Action::SaveFileAs),
            Self::menu_item("Open in Explorer..", config.shortcuts_label_for("open_in_explorer"), Action::OpenInExplorer),
            Self::menu_item("Exit", String::new(), Action::Exit),
        ];
        self.edit_items = vec![
            Self::menu_item("Undo", config.shortcuts_label_for("undo"), Action::Undo),
            Self::menu_item("Redo", config.shortcuts_label_for("redo"), Action::Redo),
            Self::menu_item("Cut", config.shortcuts_label_for("cut"), Action::Cut),
            Self::menu_item("Copy", config.shortcuts_label_for("copy"), Action::Copy),
            Self::menu_item("Paste", config.shortcuts_label_for("paste"), Action::Paste),
            Self::menu_item("Find", config.shortcuts_label_for("find"), Action::Find),
            Self::menu_item("Replace", config.shortcuts_label_for("replace"), Action::Replace),
        ];
        self.view_items = vec![(
            format!(
                " {} Highlight Keywords",
                if config.highlight_keywords() { "[x]" } else { "[ ]" }
            ),
            Action::ToggleKeywordHighlight,
        )];
        self.info_items = vec![
            Self::menu_item("FAQ ", String::new(), Action::FAQ),
        ];
        self.frame = frame_id;
    }
    fn interact(&mut self, file_logger: &mut FileLogger, input: &mut Input, pop_pup: &mut PopUpPanelFrame, text_buf: &mut TextBuf) -> Action {

        if let Some(btn) = &mut self.file_button {

            btn.calculate_control(file_logger, input);

            if (btn.clicked()) {
                pop_pup.show(self.file_items.clone(), file_logger, input)
            }
        }


        if let Some(btn) = &mut self.edit_button {
            btn.calculate_control(file_logger, input);
            if (btn.clicked()) {
                pop_pup.show(self.edit_items.clone(), file_logger, input)
            }
        }

        if let Some(btn) = &mut self.view_button {
            btn.calculate_control(file_logger, input);
            if btn.clicked() {
                pop_pup.show(self.view_items.clone(), file_logger, input)
            }
        }
        
        if let Some(btn) = &mut self.info_button {
            btn.calculate_control(file_logger, input);
            if (btn.clicked()) {
                pop_pup.show(self.info_items.clone(), file_logger, input)
            }
        }


        Action::None
    }
    fn draw(&mut self, layout: &mut Layout, screen: &mut ScreenBuf, text_buf: &mut TextBuf) {
        let open_frame = layout.get_frame(self.frame).unwrap();

        open_frame.add_control(Box::new(self.file_button.take().unwrap()));
        open_frame.add_control(Box::new(self.edit_button.take().unwrap()));
        open_frame.add_control(Box::new(self.view_button.take().unwrap()));
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
