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
use crate::screen_buffer::{Color, ScreenBuf};
use crate::text_buffer::TextBuf;
use crate::ui::c_frame::{EFrameAxis, Frame};
use crate::ui::c_layout::Layout;
use crate::ui::c_rect::Rect;

#[derive(Default)]
pub struct FilesFrame {
    buttons: Vec<Button>,
    recent_paths: Vec<PathBuf>,
    missing_paths: Vec<bool>,
    frame: u16,
    current_path: Option<PathBuf>,
    current_dirty: bool,
    current_virtual: bool,
}

impl LayoutPanel for FilesFrame {
    fn get_order(&self) -> u16 {
        0
    }

    fn get_frame_id(&self) -> u16 {
        self.frame
    }

    fn create_layout(&mut self, layout: &mut Layout, config: &mut Config) {
        let frame = Frame::new(EFrameAxis::Vertical, false);
        let open_frame = layout.open_frame(frame);
        self.frame = open_frame.frame_id;

        self.buttons.clear();
        self.recent_paths.clear();
        self.missing_paths.clear();

        let max_len: usize = 18;
        for recent_file in config.get_last_files() {
            let path = PathBuf::from(recent_file);
            let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
                continue;
            };

            let is_current = self.current_path.as_ref().is_some_and(|value| value == &path);
            let is_missing = !path.exists() && !(is_current && self.current_virtual);
            let prefix = if is_missing {
                "x "
            } else if is_current && self.current_dirty {
                "* "
            } else if is_current {
                "> "
            } else {
                "  "
            };

            let label_len = max_len.saturating_sub(prefix.chars().count());
            let mut text = String::from(prefix);
            if file_name.chars().count() > label_len {
                let trimmed: String = file_name.chars().take(label_len.saturating_sub(3)).collect();
                text.push_str(&trimmed);
                text.push_str("...");
            } else {
                text.push_str(file_name);
                let padding = label_len.saturating_sub(file_name.chars().count());
                text.push_str(&" ".repeat(padding));
            }

            let mut button = Button::new(&text);
            if is_missing {
                button.set_persistent_color(Some(Color::Red));
            } else if is_current {
                button.set_persistent_color(Some(Color::Green));
            }
            button.create_control(open_frame);
            self.buttons.push(button);
            self.recent_paths.push(path);
            self.missing_paths.push(is_missing);
        }

        layout.close_frame();
    }

    fn interact(
        &mut self,
        file_logger: &mut FileLogger,
        input: &mut Input,
        _pop_pup: &mut PopUpPanelFrame,
        _text_buf: &mut TextBuf,
    ) -> Action {
        for (index, button) in self.buttons.iter_mut().enumerate() {
            button.calculate_control(file_logger, input);
            if button.clicked() {
                if let Some(path) = self.recent_paths.get(index) {
                    if self.missing_paths.get(index).copied().unwrap_or(false) {
                        return Action::RemoveRecentPath(path.clone());
                    }
                    return Action::OpenPath(path.clone());
                }
            }
        }

        Action::None
    }

    fn draw(&mut self, layout: &mut Layout, screen: &mut ScreenBuf, _text_buf: &mut TextBuf) {
        let frame = layout.get_frame(self.frame).unwrap();
        for button in self.buttons.drain(..) {
            frame.add_control(Box::new(button));
        }
        frame.draw(&Rect::default(), screen);
    }
}

impl FilesFrame {
    pub fn set_current_document(&mut self, path: PathBuf, is_dirty: bool, is_virtual: bool) {
        self.current_path = Some(path);
        self.current_dirty = is_dirty;
        self.current_virtual = is_virtual;
    }
}
