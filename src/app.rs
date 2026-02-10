use std::io::Write;
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{Event, KeyCode, MouseButton, MouseEventKind};
use crossterm::execute;
use crate::config::Config;
use crate::controls::c_text::TextBox;
use crate::controls::t_get_rect::GetRect;
use crate::controls::t_render::Render;
use crate::fs::FileSystem;
use crate::input::Input;
use crate::logger::FileLogger;
use crate::screen_buf::ScreenBuf;
use crate::terminal;
use crate::terminal::Terminal;
use crate::ui::c_frame::{EFrameAxis, Frame};
use crate::ui::c_layout::Layout;
use crate::ui::c_rect::Rect;

pub struct App{
    pub logger: FileLogger,
    pub fs: FileSystem,
    pub config: Config,
    pub screen_buf: ScreenBuf,
    pub input: Input,
}

impl App{
    pub fn new() -> App{

        let mut logger = FileLogger::new();
        let fs = FileSystem::new();
        let config: Config = Config::new(&mut logger);
        let (x, y) = config.get_win_size();
        let screen_buf = ScreenBuf::new(x, y);
        let input = Input::default();
        logger.log("App created");
        Self{
            logger,
            fs,
            config,
            screen_buf,
            input
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (_guard, mut term) = Terminal::enter()?;
        let (mut w, mut h) = term.term_size()?;
        self.screen_buf.resize(w, h);

        loop {
            let ev = term.next_event()?;

            let mut dirty = false;

            match ev {
                Event::Key(k) => {
                    self.input.handle_input(k.code, &mut self.screen_buf);
                    dirty = true;
                }

                Event::Resize(nw, nh) => {
                    w = nw;
                    h = nh;
                    self.screen_buf.resize(w, h);

                    // clamp курсора
                    if w > 0 { self.input.cursor_x = self.input.cursor_x.min(w - 1); }
                    if h > 0 { self.input.cursor_y = self.input.cursor_y.min(h - 1); }

                    dirty = true;
                }

                Event::Mouse(m) => {
                    match m.kind {
                        MouseEventKind::Down(MouseButton::Left) => {
                            self.input.cursor_x = m.column.min(w.saturating_sub(1));
                            self.input.cursor_y = m.row.min(h.saturating_sub(1));
                            dirty = true;
                        }
                        _ => {
                            // Мышь двигается/скролл/ап — игнорим, иначе будет перерисовка 1000 раз
                            continue;
                        }
                    }
                }

                _ => continue,
            }

            if !dirty {
                continue;
            }

            // Рендер в буфер
            self.screen_buf.clear();
            self.renderer();

            // Печать: без term.clear()
            execute!(term.out, Hide, MoveTo(0, 0))?;
            self.screen_buf.present(&mut term.out)?;
            execute!(term.out, Show, MoveTo(self.input.cursor_x, self.input.cursor_y))?;
            term.out.flush()?;
        }
    }

    fn renderer(&mut self) {
        let root_rect = Rect::new(0, 0, self.screen_buf.w, self.screen_buf.h);
        let mut layout = Layout::new(root_rect);


        let top_frame = layout.open_frame(Frame::new(EFrameAxis::Horizontal));
        {
            top_frame.draw(&top_frame.area, &mut self.screen_buf);
            TextBox::new(" File |").create_control(top_frame, &mut self.screen_buf);
        }
        layout.close_frame();
    }
}