use crate::config::Config;
use crate::controls::t_render::Render;
use crate::e_actions::Action;
use crate::input::{EInputMode, Input};
use crate::logger::FileLogger;
use crate::panels::menu_panel::LayoutPanel;
use crate::panels::pop_up_panel::PopUpPanelFrame;
use crate::screen_buf::{Color, ScreenBuf};
use crate::text_buffer::TextBuf;
use crate::ui::c_frame::{EFrameAxis, Frame};
use crate::ui::c_layout::Layout;
use crate::ui::c_rect::Rect;

pub struct TextEditorFrame {
    available_rect: Rect,
    frame: u16,
}

impl LayoutPanel for TextEditorFrame {
    fn get_order(&self) -> u16 {
        0
    }

    fn get_frame_id(&self) -> u16 {
        return self.frame;
    }

    fn create_layout(&mut self, layout: &mut Layout, config: &mut Config) {
        let frame = Frame::new(EFrameAxis::Vertical, false);

        let root_rect = layout.get_root_rect();

        let mut open_frame = layout.open_frame(frame);
        open_frame.fill(root_rect);

        self.frame = open_frame.frame_id;
        self.available_rect = open_frame.content_rect();

        layout.close_frame();
    }

    fn interact(&mut self, file_logger: &mut FileLogger, input: &mut Input, pop_pup: &mut PopUpPanelFrame, text_buf: &mut TextBuf) -> Action {

        if (self.available_rect.contains(input.cursor_x, input.cursor_y) && input.mode == EInputMode::FreeMove) {
            if (input.clicked.is_some()){
                input.change_mode(EInputMode::TextEditor);
            }
        }else if (!self.available_rect.contains(input.cursor_x, input.cursor_y) && input.mode == EInputMode::TextEditor) {
            input.change_mode(EInputMode::FreeMove);
        }



        if (input.mode == EInputMode::TextEditor) {
            if (input.last_character.is_some()){
                let char = input.last_character.unwrap();

                if (char == '\n'){
                    text_buf.add_line();
                    input.cursor_y += 1;
                }else if (char == '\x08'){
                    text_buf.remove_char();
                }else{
                    if (text_buf.lines.len() == 0) {
                        text_buf.lines.push(Vec::new());
                    }

                    text_buf.add_char(char);

                    input.cursor_x += 1;
                }
            }


            if (text_buf.lines.len() > 0){
                let line = text_buf.get_current_line();


                let max_x = self.available_rect.x + line.len() as u16;
                let min_x = self.available_rect.x;

                let max_y = self.available_rect.y + (text_buf.lines.len()-1) as u16;
                let min_y = self.available_rect.y;

                input.cursor_y = self.available_rect.y + text_buf.line_index as u16;
                input.cursor_x = self.available_rect.x + text_buf.current_index as u16;

                input.clamp_cursor(min_x, max_x, min_y, max_y);
            }

        }

        Action::None
    }

    fn draw(&mut self, layout: &mut Layout, screen: &mut ScreenBuf, text_buf: &mut TextBuf) {
        let frame = layout.get_frame(self.frame).unwrap();

        let min_x = self.available_rect.x;
        let min_y = self.available_rect.y;
        let max_x = self.available_rect.x + self.available_rect.w;
        let max_y = self.available_rect.y + self.available_rect.h;

        frame.draw(&Rect::default(), screen);


        for line_index in 0..text_buf.lines.len() {
            if (self.available_rect.y as usize + line_index >= max_y as usize){
                break;
            }

            for x in 0..text_buf.lines[line_index].len() {
                if (x >= max_x as usize){
                    break;
                }
                screen.set(min_x + x as u16, min_y + line_index as u16, text_buf.lines[line_index][x], Color::White);
            }
        }
    }


}

impl Default for TextEditorFrame {
    fn default() -> Self {
        Self{
            available_rect: Default::default(),
            frame: 0,
        }
    }
}