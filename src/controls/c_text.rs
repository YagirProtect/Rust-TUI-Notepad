use crate::controls::t_get_rect::GetRect;
use crate::controls::t_render::Render;
use crate::screen_buf::ScreenBuf;
use crate::ui::c_frame::Frame;
use crate::ui::c_rect::Rect;

pub struct TextBox{
    text: Vec<char>,
}

impl GetRect for TextBox {
    fn get_bounds(&self) -> Rect {
        return Rect::new(0, 0, self.text.len() as u16, 1);
    }

    fn create_control(&mut self, frame: &mut Frame, screen: &mut ScreenBuf) {
        frame.add(self.get_bounds());
        self.draw(&frame.get_area_with_offsets(), screen);
    }
}

impl Render for TextBox {
    fn draw(&self, rect: &Rect, screen: &mut ScreenBuf) {
        let bounds = self.get_bounds();


        for i in 0..bounds.w {
            screen.set(rect.x + i, rect.y, self.text[i as usize]);
        }
    }
}

impl TextBox{
    pub fn new(text: &str) -> TextBox{
        TextBox{
            text: String::from(text).chars().collect(),
        }
    }
}