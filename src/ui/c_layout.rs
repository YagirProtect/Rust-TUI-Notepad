use crate::input::Input;
use crate::logger::FileLogger;
use crate::panels::menu_panel::{LayoutPanel, MenuFrame};
use crate::panels::pop_up_panel::PopUpPanelFrame;
use crate::screen_buf::ScreenBuf;
use crate::ui::c_frame::{EFrameAxis, Frame};
use crate::ui::c_rect::Rect;

pub struct Layout{
    root: Rect,
    last_frame: u16,

    frames: Vec<Frame>,
    layout_panels: Vec<Box<dyn LayoutPanel>>,
    frame_cursor_x: u16,
    frame_cursor_y: u16,
}

impl Layout {
    pub fn interact(&mut self, file_logger: &mut FileLogger, input: &Input, pop_up_panel_frame: &mut PopUpPanelFrame) {

        for l in self.layout_panels.iter_mut() {
            l.interact(file_logger, input, pop_up_panel_frame);
        }
    }

    pub fn draw(&mut self, screen: &mut ScreenBuf, pop_pup: &mut PopUpPanelFrame) {
        let mut panels = std::mem::take(&mut self.layout_panels);

        for item in panels.iter_mut() {
            item.draw(self, screen);
        }

        if (pop_pup.active) {
            pop_pup.draw(self, screen);
        }
        self.layout_panels = panels;
    }
}

impl Layout {
    pub fn add_panel(&mut self, panel: Box<dyn LayoutPanel>) {
        self.layout_panels.push(panel);
    }
}

impl Layout {
    pub fn open_frame(&mut self, frame: Frame) -> &mut Frame {

        self.frames.push(frame);

        let val: &mut Frame = self.frames.last_mut().unwrap();
        val.set_area(Rect::new(self.frame_cursor_x, self.frame_cursor_y, self.root.w - self.frame_cursor_x, self.root.h - self.frame_cursor_y));
        return val;
    }

    pub fn get_frame(&mut self, id: u16)-> Option<&mut Frame>{

        for frame in self.frames.iter_mut() {
            if (id == frame.frame_id){
                return Some(frame);
            }
        }

        return None;
    }

    pub fn close_frame(&mut self){

        let frame = self.get_frame(self.last_frame).unwrap();

        match frame.axis{
            EFrameAxis::Vertical => {
                self.frame_cursor_x += frame.expand + 1;
            }
            EFrameAxis::Horizontal => {
                self.frame_cursor_y += frame.expand + 1;
            }
        }
    }
}

impl Layout {
    pub fn new(rect: Rect) -> Layout {
        Self {
            root: rect,
            last_frame: 0,
            frames: vec![],
            layout_panels: vec![],
            frame_cursor_x: 0,
            frame_cursor_y: 0,
        }
    }
}