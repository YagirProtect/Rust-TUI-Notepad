use crate::controls::c_text::TextBox;
use crate::controls::t_get_rect::{Control, GetRect};
use crate::controls::t_render::Render;
use crate::input::Input;
use crate::logger::FileLogger;
use crate::screen_buffer::{Color, ScreenBuf};
use crate::ui::c_rect::Rect;



#[derive(Debug)]
pub struct Button{
    text_box: TextBox,
    is_clicked: bool,
    rect: Rect,
    persistent_color: Option<Color>,
    persistent_background: Option<Color>,
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
    fn draw(&mut self, rect: &Rect, screen: &mut ScreenBuf) {
        self.text_box.draw(rect, screen);
    }
}

impl Control for Button {
    fn get_rect(&self) -> &Rect {
        return &self.rect;
    }

    fn save_rect(&mut self, rect: Rect) {
        self.rect = rect;
    }


    fn calculate_control(&mut self, logger: &mut FileLogger, input: &Input) {
        let _ = logger;
        let base_background = self.persistent_background.unwrap_or(Color::Black);
        let is_light_background = matches!(
            base_background,
            Color::Yellow | Color::White | Color::Gray | Color::Green
        );
        let hover_color = if is_light_background { Color::Black } else { Color::Yellow };
        let pressed_color = if is_light_background { Color::Black } else { Color::White };

        self.text_box
            .set_color(self.persistent_color.unwrap_or(Color::White));
        self.text_box.set_background(base_background);
        self.is_clicked = false;
        if self.get_rect().contains(input.cursor_x, input.cursor_y) {
            if input.mouse_down.is_some() {
                self.text_box.set_color(pressed_color);
            } else {
                self.text_box.set_color(hover_color);
            }

            if input.mouse_down.is_some() && input.mouse_released.is_some() {
                self.is_clicked = true;
            }
        } else if input.mouse_down.is_some() {
            self.text_box
                .set_color(self.persistent_color.unwrap_or(Color::White));
            self.text_box.set_background(base_background);
        }
    }
}

impl Button {
    pub fn new(text: &str) -> Button{
        Self{
            text_box: TextBox::new(text),
            is_clicked: false,
            rect: Rect::default(),
            persistent_color: None,
            persistent_background: None,
        }
    }

    pub fn set_persistent_color(&mut self, color: Option<Color>) {
        self.persistent_color = color;
        self.text_box
            .set_color(self.persistent_color.unwrap_or(Color::White));
    }

    pub fn set_persistent_background(&mut self, color: Option<Color>) {
        self.persistent_background = color;
        self.text_box
            .set_background(self.persistent_background.unwrap_or(Color::Black));
    }
}
