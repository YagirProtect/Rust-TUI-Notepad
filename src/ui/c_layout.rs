use std::cmp::Reverse;
use crate::input::Input;
use crate::logger::FileLogger;
use crate::panels::menu_panel::{LayoutPanel, MenuFrame};
use crate::panels::pop_up_panel::PopUpPanelFrame;
use crate::screen_buf::ScreenBuf;
use crate::text_buffer::TextBuf;
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
    pub fn interact(
        &mut self,
        file_logger: &mut FileLogger,
        input: &mut Input,
        pop_up_panel_frame: &mut PopUpPanelFrame,
        text_buf: &mut TextBuf
    ) {

        pop_up_panel_frame.interact(file_logger, input, &mut PopUpPanelFrame::new(), text_buf);
        if (pop_up_panel_frame.active){
            if (pop_up_panel_frame.try_hit(self, input)){
                return;
            }
        }

        let mut panels = std::mem::take(&mut self.layout_panels);

        let mut idx: Vec<usize> = (0..panels.len()).collect();
        idx.sort_unstable_by_key(|&i| Reverse(panels[i].get_order()));

        let mut hit: Option<usize> = None;
        for i in idx {
            if panels[i].try_hit(self, input) {
                hit = Some(i);
                break;
            }
        }

        if let Some(i) = hit {
            panels[i].interact(file_logger, input, pop_up_panel_frame, text_buf);
        }

        self.layout_panels = panels;
    }



    pub fn draw(&mut self, screen: &mut ScreenBuf, pop_pup: &mut PopUpPanelFrame, file_logger: &mut FileLogger, text_buf: &mut TextBuf) {
        let mut panels = std::mem::take(&mut self.layout_panels);

        file_logger.log("=======");

        for item in panels.iter_mut() {
            item.draw(self, screen, text_buf);


            file_logger.log("draw panel");
        }

        if (pop_pup.active) {
            pop_pup.draw(self, screen, text_buf);
        }
        self.layout_panels = panels;
    }
}

impl Layout {
    pub fn add_panel(&mut self, panel: Box<dyn LayoutPanel>) {
        self.layout_panels.push(panel);
    }
    
    
    pub fn get_root_rect(&self) -> Rect {
        self.root
    }
}

impl Layout {
    pub fn open_frame(&mut self, frame: Frame) -> &mut Frame {

        self.frames.push(frame);

        let val: &mut Frame = self.frames.last_mut().unwrap();
        val.set_area(Rect::new(self.frame_cursor_x, self.frame_cursor_y, self.root.w - self.frame_cursor_x, self.root.h - self.frame_cursor_y));
        self.last_frame = val.frame_id;

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