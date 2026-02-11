use crate::controls::c_text::TextBox;
use crate::controls::t_get_rect::{Control, GetRect};
use crate::controls::t_render::Render;
use crate::input::Input;
use crate::logger::FileLogger;
use crate::screen_buf::{Color, ScreenBuf};
use crate::ui::c_rect::Rect;



pub struct Button{
    text_box: TextBox,
    is_clicked: bool,
}

impl Button {
    pub fn clicked(&self) -> bool {
        return self.is_clicked;
    }
}

impl GetRect for Button{
    fn get_bounds(&self) -> Rect {
        return self.text_box.get_bounds();
    }
}

impl Render for Button {
    fn draw(&self, rect: &Rect, screen: &mut ScreenBuf) {
        self.text_box.draw(rect, screen);
    }
}

impl Control for Button {
    fn calculate_control(&mut self, rect: Rect, logger: &mut FileLogger, input: &Input) {
        self.text_box.set_color(Color::White);
        self.is_clicked = false;
        if rect.contains(input.cursor_x, input.cursor_y) {
            if (input.clicked.is_some()) {
                self.text_box.set_color(Color::Blue);
                self.is_clicked = true;
            } else {
                self.text_box.set_color(Color::Yellow);
            }
        }
    }
}

impl Button {
    pub fn new(text: &str) -> Button{
        Self{
            text_box: TextBox::new(text),
            is_clicked: false,
        }
    }
}