use crate::config::Config;
use crate::controls::t_render::Render;
use crate::fs::FileSystem;
use crate::input::Input;
use crate::logger::FileLogger;
use crate::panels::files_panel::FilesFrame;
use crate::panels::menu_panel::{LayoutPanel, MenuFrame};
use crate::panels::pop_up_panel::PopUpPanelFrame;
use crate::panels::text_editor_panel::TextEditorFrame;
use crate::screen_buf::ScreenBuf;
use crate::terminal::Terminal;
use crate::ui::c_layout::Layout;
use crate::ui::c_rect::Rect;
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{Event, KeyEventKind, MouseButton, MouseEventKind};
use crossterm::{event, execute};
use std::io::Write;
use std::time::Duration;
use crate::text_buffer::TextBuf;

pub struct App{
    pub logger: FileLogger,
    pub fs: FileSystem,
    pub config: Config,
    pub screen_buf: ScreenBuf,
    pub input: Input,
    pub pop_up: PopUpPanelFrame,
    pub text_buffer: TextBuf,
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
            input,
            pop_up: PopUpPanelFrame::new(),
            text_buffer: TextBuf::default()
        }
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (_guard, mut term) = Terminal::enter()?;
        let (mut w, mut h) = term.term_size()?;
        self.screen_buf.resize(w, h);

        let mut first_draw = true;
        loop {
            let mut dirty = false;
            if (first_draw){
                first_draw = false;
                dirty = true;
            }

            if event::poll(Duration::from_millis(16))? {
                let ev = event::read()?;

                match ev {
                    Event::Key(k) => {

                        match k.kind {
                            KeyEventKind::Press => {
                                self.input.handle_input(k.code, &mut self.screen_buf, &mut self.text_buffer);
                                dirty = true;

                            }
                            KeyEventKind::Repeat => {}
                            KeyEventKind::Release => {
                                dirty = true;
                            }
                        }
                    }

                    Event::Resize(nw, nh) => {
                        w = nw; h = nh;
                        self.screen_buf.resize(w, h);

                        if w > 0 { self.input.cursor_x = self.input.cursor_x.min(w - 1); }
                        if h > 0 { self.input.cursor_y = self.input.cursor_y.min(h - 1); }

                        dirty = true;
                    }

                    Event::Mouse(m) => match m.kind {
                        MouseEventKind::Down(MouseButton::Left) => {
                            self.input.cursor_x = m.column.min(w.saturating_sub(1));
                            self.input.cursor_y = m.row.min(h.saturating_sub(1));
                            self.input.clicked = Some((m.column, m.row));
                            dirty = true;
                        }
                        MouseEventKind::Up(MouseButton::Left) => {
                            dirty = true;
                        }
                        _ => {} // moved/scroll игнорим
                    },

                    _ => {}
                }
            }

            // если событий нет, но нужно “додёрнуть” кадры — тоже рисуем
            if !dirty {
                continue;
            }

            self.screen_buf.clear();


            self.draw_ui();

            execute!(term.out, Hide, MoveTo(0, 0))?;
            self.screen_buf.present(&mut term.out)?;
            execute!(term.out, Show, MoveTo(self.input.cursor_x, self.input.cursor_y))?;
            term.out.flush()?;

            self.input.clicked = None;
            self.input.last_character = None;
        }
    }

    fn create_layout(&mut self, layout: &mut Layout){

        self.pop_up.create_layout(layout, &mut self.config);

        let mut menu_panel = MenuFrame::default();
        menu_panel.create_layout(layout, &mut self.config);


        let mut files_panel = FilesFrame::default();
        files_panel.create_layout(layout, &mut self.config);

        let mut text_editor = TextEditorFrame::default();
        text_editor.create_layout(layout, &mut self.config);


        layout.add_panel(Box::new(files_panel));
        layout.add_panel(Box::new(menu_panel));
        layout.add_panel(Box::new(text_editor));
    }




    fn draw_ui(&mut self) {
        let root_rect = Rect::new(0, 0, self.screen_buf.w, self.screen_buf.h);
        let mut layout = Layout::new(root_rect);
        self.create_layout(&mut layout);

        layout.interact(&mut self.logger, &mut self.input, &mut self.pop_up, &mut self.text_buffer);

        layout.draw(&mut self.screen_buf, &mut self.pop_up, &mut self.logger, &mut self.text_buffer);
    }
}