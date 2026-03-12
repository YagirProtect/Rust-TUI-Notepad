use std::collections::HashSet;
use std::path::PathBuf;

use crate::config::Config;
use crate::controls::c_button::Button;
use crate::controls::c_delimiter::Delimiter;
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

const TAB_GAP: u16 = 1;
const TAB_MAX_LABEL_LEN: usize = 24;
const TAB_ACTIVE_FG: Color = Color::Black;
const TAB_ACTIVE_BG: Color = Color::Yellow;
const TAB_MISSING_FG: Color = Color::White;
const TAB_MISSING_BG: Color = Color::DarkRed;
const CLOSE_FG: Color = Color::White;
const CLOSE_BG: Color = Color::DarkRed;
const CLOSE_MISSING_BG: Color = Color::Red;

struct VisibleTab {
    index: usize,
    tab_button: Button,
    close_button: Option<Button>,
}

#[derive(Default)]
pub struct FilesFrame {
    visible_tabs: Vec<VisibleTab>,
    delimiters: Vec<Delimiter>,
    recent_paths: Vec<PathBuf>,
    missing_paths: Vec<bool>,
    frame: u16,
    content_rect: Rect,
    scroll_x: u16,
    max_scroll_x: u16,
    current_path: Option<PathBuf>,
    dirty_paths: HashSet<PathBuf>,
    virtual_paths: HashSet<PathBuf>,
}

impl FilesFrame {
    fn build_tab_text(&self, file_name: &str, is_dirty: bool, is_missing: bool) -> String {
        let prefix = if is_missing { "x " } else { "" };
        let suffix = if is_dirty { " *" } else { "" };

        let mut text = String::from(prefix);
        let max_name_len = TAB_MAX_LABEL_LEN
            .saturating_sub(prefix.chars().count())
            .saturating_sub(suffix.chars().count());
        if file_name.chars().count() > max_name_len {
            let trimmed: String = file_name
                .chars()
                .take(max_name_len.saturating_sub(3))
                .collect();
            text.push_str(&trimmed);
            text.push_str("...");
        } else {
            text.push_str(file_name);
        }
        text.push_str(suffix);

        // keep room for close "x" on top of the trailing padding
        text.push_str("   ");
        text
    }

}

impl LayoutPanel for FilesFrame {
    fn get_order(&self) -> u16 {
        0
    }

    fn get_frame_id(&self) -> u16 {
        self.frame
    }

    fn create_layout(&mut self, layout: &mut Layout, config: &mut Config) {
        let frame = Frame::new(EFrameAxis::Horizontal, false);
        let open_frame = layout.open_frame(frame);
        self.frame = open_frame.frame_id;
        self.content_rect = open_frame.content_rect();
        layout.close_frame();

        self.visible_tabs.clear();
        self.delimiters.clear();
        self.recent_paths.clear();
        self.missing_paths.clear();

        let mut total_width = 0u16;
        let mut widths: Vec<u16> = Vec::new();

        for recent_file in config.get_last_files() {
            let path = PathBuf::from(recent_file);
            let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
                continue;
            };

            let is_virtual = self.virtual_paths.contains(&path);
            let is_dirty = self.dirty_paths.contains(&path);
            let is_missing = !path.exists() && !is_virtual;
            let label = self.build_tab_text(file_name, is_dirty, is_missing);
            let width = label.chars().count() as u16;

            widths.push(width);
            self.recent_paths.push(path);
            self.missing_paths.push(is_missing);

            total_width = total_width.saturating_add(width).saturating_add(TAB_GAP);
        }

        self.max_scroll_x = total_width.saturating_sub(self.content_rect.w);
        self.scroll_x = self.scroll_x.min(self.max_scroll_x);

        let widths_len = widths.len();
        let mut cursor_x = self.content_rect.x as i32 - self.scroll_x as i32;
        let view_start = self.content_rect.x as i32;
        let view_end = view_start + self.content_rect.w as i32;
        for (index, width) in widths.into_iter().enumerate() {
            let path = &self.recent_paths[index];
            let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
                continue;
            };

            let is_current = self.current_path.as_ref().is_some_and(|value| value == path);
            let is_missing = self.missing_paths[index];
            let is_dirty = self.dirty_paths.contains(path);
            let label = self.build_tab_text(file_name, is_dirty, is_missing);

            let tab_start = cursor_x;
            let tab_end = cursor_x + width as i32;
            let visible_start = tab_start.max(view_start);
            let visible_end = tab_end.min(view_end);

            if visible_start < visible_end {
                let left_cut = (visible_start - tab_start) as usize;
                let visible_width = (visible_end - visible_start) as u16;
                let visible_text: String = label
                    .chars()
                    .skip(left_cut)
                    .take(visible_width as usize)
                    .collect();

                let tab_rect = Rect::new(visible_start as u16, self.content_rect.y, visible_width, 1);
                let mut tab_button = Button::new(&visible_text);
                if is_missing {
                    tab_button.set_persistent_color(Some(TAB_MISSING_FG));
                    tab_button.set_persistent_background(Some(TAB_MISSING_BG));
                } else if is_current {
                    tab_button.set_persistent_color(Some(TAB_ACTIVE_FG));
                    tab_button.set_persistent_background(Some(TAB_ACTIVE_BG));
                }
                tab_button.save_rect(tab_rect);

                let close_x = tab_end - 2;
                let close_button = if close_x >= view_start && close_x < view_end {
                    let close_rect = Rect::new(close_x as u16, self.content_rect.y, 1, 1);
                    let mut button = Button::new("x");
                    button.set_persistent_color(Some(CLOSE_FG));
                    button.set_persistent_background(Some(if is_missing {
                        CLOSE_MISSING_BG
                    } else {
                        CLOSE_BG
                    }));
                    button.save_rect(close_rect);
                    Some(button)
                } else {
                    None
                };

                self.visible_tabs.push(VisibleTab {
                    index,
                    tab_button,
                    close_button,
                });
            }

            if index + 1 < widths_len {
                let delimiter_x = tab_end;
                if delimiter_x >= view_start && delimiter_x < view_end {
                    let mut delimiter = Delimiter::new();
                    delimiter.save_rect(Rect::new(delimiter_x as u16, self.content_rect.y, 1, 1));
                    self.delimiters.push(delimiter);
                }
            }

            cursor_x += width as i32 + TAB_GAP as i32;
        }
    }

    fn interact(
        &mut self,
        file_logger: &mut FileLogger,
        input: &mut Input,
        _pop_pup: &mut PopUpPanelFrame,
        _text_buf: &mut TextBuf,
    ) -> Action {
        if self.content_rect.contains(input.cursor_x, input.cursor_y) {
            if let Some((dx, dy)) = input.mouse_scroll {
                let delta = if dx != 0 { dx } else { dy };
                if delta != 0 {
                    let prev = self.scroll_x;
                    let next = (self.scroll_x as i32 + delta)
                        .clamp(0, self.max_scroll_x as i32);
                    self.scroll_x = next as u16;
                    if self.scroll_x != prev {
                        return Action::SetFilesTabsScroll(self.scroll_x);
                    }
                }
            }
        }

        for tab in &mut self.visible_tabs {
            if let Some(close_button) = &mut tab.close_button {
                close_button.calculate_control(file_logger, input);
                if close_button.clicked() {
                    if let Some(path) = self.recent_paths.get(tab.index) {
                        return Action::RemoveRecentPath(path.clone());
                    }
                }
            }

            tab.tab_button.calculate_control(file_logger, input);
            if tab.tab_button.clicked() {
                if let Some(path) = self.recent_paths.get(tab.index) {
                    let is_missing = self.missing_paths.get(tab.index).copied().unwrap_or(false);
                    let is_dirty = self.dirty_paths.contains(path);
                    if is_missing && !is_dirty {
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
        for delimiter in self.delimiters.drain(..) {
            frame.add_control(Box::new(delimiter));
        }
        for tab in self.visible_tabs.drain(..) {
            frame.add_control(Box::new(tab.tab_button));
            if let Some(close_button) = tab.close_button {
                frame.add_control(Box::new(close_button));
            }
        }
        frame.draw(&Rect::default(), screen);
    }
}

impl FilesFrame {
    pub fn set_current_document(
        &mut self,
        path: PathBuf,
        dirty_paths: HashSet<PathBuf>,
        virtual_paths: HashSet<PathBuf>,
    ) {
        self.current_path = Some(path);
        self.dirty_paths = dirty_paths;
        self.virtual_paths = virtual_paths;
    }

    pub fn set_scroll_x(&mut self, value: u16) {
        self.scroll_x = value;
    }
}
