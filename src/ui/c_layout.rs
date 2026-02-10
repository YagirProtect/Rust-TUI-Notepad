use crate::screen_buf::ScreenBuf;
use crate::ui::c_frame::{EFrameAxis, Frame};
use crate::ui::c_rect::Rect;

pub struct Layout{
    root: Rect,
    last_frame: Frame,

    frame_cursor_x: u16,
    frame_cursor_y: u16,
}

impl Layout {
    pub fn open_frame(&mut self, frame: Frame) -> &mut Frame {
        self.last_frame = frame;

        self.last_frame.set_area(Rect::new(self.frame_cursor_x, self.frame_cursor_y, self.root.w - self.frame_cursor_x, self.root.h - self.frame_cursor_y));

        return &mut self.last_frame
    }

    pub fn close_frame(&mut self){
        match self.last_frame.axis{
            EFrameAxis::Vertical => {
                self.frame_cursor_x += self.last_frame.expand;
            }
            EFrameAxis::Horizontal => {
                self.frame_cursor_y += self.last_frame.expand;
            }
        }
    }
}

impl Layout {
    pub fn new(rect: Rect) -> Layout {
        Self {
            root: rect,
            last_frame: Frame::new(EFrameAxis::Horizontal),
            frame_cursor_x: 0,
            frame_cursor_y: 0,
        }
    }
}