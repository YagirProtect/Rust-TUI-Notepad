use crate::controls::t_render::Render;
use crate::screen_buf::ScreenBuf;
use crate::ui::c_rect::Rect;

pub enum EFrameAxis{
    Vertical,
    Horizontal,
}
pub struct Frame{
    pub axis: EFrameAxis,
    pub area: Rect,
    pub cursor: u16,

    pub expand: u16,
}

impl Frame {
    pub fn add(&mut self, control: Rect) {
        match self.axis {
            EFrameAxis::Vertical => {
                self.cursor += control.h;
                if (self.expand < control.w){
                    self.expand = control.w + 2;
                }
            }
            EFrameAxis::Horizontal => {
                self.cursor += control.w;

                if (self.expand < control.h){
                    self.expand = control.h + 2;
                }
            }
        }
    }

    pub fn get_area_with_offsets(&self) -> Rect {
        return Rect::new(self.area.x + 1, self.area.y + 1, self.area.w, self.area.h);
    }
}

impl Render for Frame {
    fn draw(&self, rect: &Rect, screen: &mut ScreenBuf) {
        if self.area.w < 2 || self.area.h < 2 { return; }

        let x0 = self.area.x;
        let y0 = self.area.y;
        let mut x1 = self.area.x;
        let mut y1 = self.area.y;

        match self.axis {
            EFrameAxis::Vertical => {
                x1 += self.expand;
                y1 += self.area.h - 1;
            }
            EFrameAxis::Horizontal => {
                y1 += self.expand;
                x1 += self.area.w - 1;
            }
        }


        screen.set(x0, y0, '┌');
        screen.set(x1, y0, '┐');
        screen.set(x0, y1, '└');
        screen.set(x1, y1, '┘');

        for x in (x0 + 1)..x1 {
            screen.set(x, y0, '─');
            screen.set(x, y1, '─');
        }

        for y in (y0 + 1)..y1 {
            screen.set(x0, y, '│');
            screen.set(x1, y, '│');
        }
    }
}

impl Frame{
    pub fn new(axis: EFrameAxis) -> Frame{
        Self{
            area: Rect::new(0,0,0,0),
            axis: axis,
            expand: 2,
            cursor: 0,
        }
    }

    pub fn set_area(&mut self, area: Rect){
        self.area = area;
    }
}