use std::sync::atomic::AtomicU16;
use std::sync::atomic::Ordering::Relaxed;
use crate::characters::{BORDER_DOUBLE, BORDER_ROUNDED};
use crate::controls::t_get_rect::Control;
use crate::controls::t_render::Render;
use crate::logger::FileLogger;
use crate::screen_buf::{Color, ScreenBuf};
use crate::ui::c_rect::Rect;

static ID: AtomicU16 = AtomicU16::new(0);

pub enum EFrameAxis{
    Vertical,
    Horizontal,
}
pub struct Frame{
    pub frame_id: u16,
    pub axis: EFrameAxis,
    pub area: Rect,
    pub cursor: u16,
    pub expand: u16,

    auto_size: bool,

    list: Vec<Box<dyn Control>>
}

impl Frame {
    pub fn fill(&mut self, root_rect: &Rect) {
        match self.axis {
            EFrameAxis::Vertical => {
                let delta_x = root_rect.w - self.area.x - 1;
                self.expand = delta_x;
            }
            EFrameAxis::Horizontal => {
                let delta_y = root_rect.h - self.area.y - 1;
                self.expand = delta_y;
            }
        }
    }
}

impl Frame {
    pub fn get_available_rect(&self) -> Rect {
        let mut content_rect =  self.content_rect();


        match self.axis {
            EFrameAxis::Vertical => {
                content_rect.set_y(content_rect.y + self.cursor);
            }
            EFrameAxis::Horizontal => {
                content_rect.set_x(content_rect.x + self.cursor);
            }
        }

        return content_rect;
    }
}

impl Frame {
    pub fn add_layout_changes(&mut self, control: Rect) -> Rect {
        let content_rect = self.content_rect();

        match self.axis {
            EFrameAxis::Vertical => {
                self.expand = self.expand.max(control.w.saturating_add(1)); // +2 бордер
                let r = Rect::new(
                    content_rect.x,
                    content_rect.y + self.cursor,
                    control.w,
                    control.h,
                );
                self.cursor = self.cursor.saturating_add(control.h); // чтобы не забывать add_cursor
                r
            }
            EFrameAxis::Horizontal => {
                self.expand = self.expand.max(control.h.saturating_add(1));
                let r = Rect::new(
                    content_rect.x + self.cursor,
                    content_rect.y,
                    control.w,
                    control.h,
                );
                self.cursor = self.cursor.saturating_add(control.w);
                r
            }
        }
    }
}

impl Frame {
    pub fn border_bounds(&self) -> (u16, u16, u16, u16) {
        let x0 = self.area.x;
        let y0 = self.area.y;

        let mut x1 = x0;
        let mut y1 = y0;

        match self.axis {
            EFrameAxis::Vertical => {
                x1 = x0.saturating_add(self.expand);
                y1 = y0.saturating_add(self.area.h.saturating_sub(1));

                if self.auto_size {
                    y1 = y0.saturating_add(self.cursor).saturating_add(1);
                }
            }
            EFrameAxis::Horizontal => {
                y1 = y0.saturating_add(self.expand);
                x1 = x0.saturating_add(self.area.w.saturating_sub(1));

                if self.auto_size {
                    x1 = x0.saturating_add(self.cursor).saturating_add(1);
                }
            }
        }

        (x0, y0, x1, y1)
    }

    pub fn border_rect(&self) -> Rect {
        let (x0, y0, x1, y1) = self.border_bounds();
        Rect::new(
            x0,
            y0,
            x1.saturating_sub(x0).saturating_add(1),
            y1.saturating_sub(y0).saturating_add(1),
        )
    }

    pub fn content_rect(&self) -> Rect {
        let r = self.border_rect();
        Rect::new(
            r.x.saturating_add(1),
            r.y.saturating_add(1),
            r.w.saturating_sub(2),
            r.h.saturating_sub(2),
        )
    }

    pub fn hit(&self, mx: u16, my: u16) -> bool {
        return self.content_rect().contains(mx, my);
    }
}


impl Render for Frame {
    fn draw(&mut self, _rect: &Rect, screen: &mut ScreenBuf) {
        let (x0, y0, x1, y1) = self.border_bounds();
        if x1 <= x0 || y1 <= y0 { return; }

        let border = BORDER_ROUNDED;


        for i in x0..x1 {
            for k in y0..y1 {
                screen.set(i, k, ' ', Color::White);
            }
        }




        screen.set(x0, y0, border.tl, Color::Blue);
        screen.set(x1, y0, border.tr, Color::Blue);
        screen.set(x0, y1, border.bl, Color::Blue);
        screen.set(x1, y1, border.br, Color::Blue);

        for x in (x0 + 1)..x1 {
            screen.set(x, y0, border.h, Color::Blue);
            screen.set(x, y1, border.h, Color::Blue);
        }
        for y in (y0 + 1)..y1 {
            screen.set(x0, y, border.v, Color::Blue);
            screen.set(x1, y, border.v, Color::Blue);
        }

        for li in self.list.iter_mut() {
            let rect = li.get_rect().clone();
            li.draw(&rect, screen);
        }
    }
}

impl Frame{
    pub fn new(axis: EFrameAxis, auto_size: bool) -> Frame{
        Self{
            area: Rect::new(0,0,0,0),
            axis: axis,
            expand: 2,
            cursor: 0,
            auto_size: auto_size,
            list: Vec::new(),
            frame_id: ID.fetch_add(1, Relaxed)
        }
    }

    pub fn add_control(&mut self, control: Box<dyn Control>) {
        self.list.push(control);
    }

    pub fn set_area(&mut self, area: Rect){
        self.cursor = 0;
        self.area = area;
    }
}