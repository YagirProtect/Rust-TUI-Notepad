use crate::config::Config;
use crate::controls::c_button::Button;
use crate::controls::t_get_rect::Control;
use crate::controls::t_render::Render;
use crate::e_actions::Action;
use crate::fs::FileSystem;
use crate::input::Input;
use crate::logger::FileLogger;
use crate::pop_up::PopUpPanelFrame;
use crate::screen_buf::ScreenBuf;
use crate::terminal::Terminal;
use crate::ui::c_frame::{EFrameAxis, Frame};
use crate::ui::c_layout::Layout;
use crate::ui::c_rect::Rect;
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{Event, KeyEventKind, MouseButton, MouseEventKind};
use crossterm::{event, execute};
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;
use crate::controls::c_text::TextBox;

pub struct App{
    pub logger: FileLogger,
    pub fs: FileSystem,
    pub config: Config,
    pub screen_buf: ScreenBuf,
    pub input: Input,
    pub pop_up: PopUpPanelFrame
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
            pop_up: PopUpPanelFrame::new()
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
                                self.input.handle_input(k.code, &mut self.screen_buf);
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

            // render
            self.screen_buf.clear();
            let user_action = self.renderer();

            execute!(term.out, Hide, MoveTo(0, 0))?;
            self.screen_buf.present(&mut term.out)?;
            execute!(term.out, Show, MoveTo(self.input.cursor_x, self.input.cursor_y))?;
            term.out.flush()?;

            self.input.clicked = None;
        }
    }

    fn renderer(&mut self) -> Action {
        self.logger.log("Renderer");
        let root_rect = Rect::new(0, 0, self.screen_buf.w, self.screen_buf.h);
        let mut layout = Layout::new(root_rect);

        let mut top_frame = layout.open_frame(Frame::new(EFrameAxis::Horizontal, false));
        {
            let mut fileBtn = Button::new(" FILE ");
            fileBtn.create_control(top_frame, &mut self.screen_buf, &mut self.logger, &self.input);
            let mut delimiter1 = TextBox::new("│");
            delimiter1.create_control(top_frame, &mut self.screen_buf, &mut self.logger, &self.input);
            let mut editBtn = Button::new(" EDIT ");
            editBtn.create_control(top_frame, &mut self.screen_buf, &mut self.logger, &self.input);
            let mut delimiter2 = TextBox::new("│");
            delimiter2.create_control(top_frame, &mut self.screen_buf, &mut self.logger, &self.input);
            let mut infoBtn = Button::new(" INFO ");
            infoBtn.create_control(top_frame, &mut self.screen_buf, &mut self.logger, &self.input);


            if (fileBtn.clicked()) {
                self.pop_up.show(vec![
                    (" New File.. ".to_string(), Action::NewFile),
                    (" Open File..".to_string(), Action::OpenFile),
                    (" Save File..".to_string(), Action::SaveFile),
                ], &mut self.logger, &self.input)
            }

            if (editBtn.clicked()) {
                self.pop_up.show(vec![
                    (" Undo ".to_string(), Action::NewFile),
                    (" Rendo".to_string(), Action::OpenFile),
                    (" Cut".to_string(), Action::SaveFile),
                    (" Copy".to_string(), Action::SaveFile),
                    (" Paste".to_string(), Action::SaveFile),
                    (" Find".to_string(), Action::SaveFile),
                    (" Replace".to_string(), Action::SaveFile),
                ], &mut self.logger, &self.input)
            }

            if (infoBtn.clicked()) {
                self.pop_up.show(vec![
                    (" FAQ ".to_string(), Action::NewFile),
                ], &mut self.logger, &self.input)
            }

            top_frame.add_control(vec![
                Box::new(fileBtn),
                Box::new(delimiter1),
                Box::new(editBtn),
                Box::new(delimiter2),
                Box::new(infoBtn),
            ]);


            top_frame.draw(&Rect::default(), &mut self.screen_buf);
        }
        layout.close_frame();

        let left_frame = layout.open_frame(Frame::new(EFrameAxis::Vertical, false));
        {
            let mut buttons: Vec<Box<dyn Control>> = vec![];
            let max_len = 18;
            for get_last_file in self.config.get_last_files() {
                let mut str = String::from(" ");

                let get_last_file = PathBuf::from(get_last_file.clone());

                self.logger.log(format!("File {}: {}", get_last_file.display(), get_last_file.exists()));
                if (get_last_file.exists()) {
                    if (get_last_file.file_name().is_some()) {
                        let mut file_name = get_last_file.file_name().unwrap().to_str().unwrap().to_string();

                        if (file_name.len() > max_len) {
                            file_name = file_name[0..max_len - 3].to_string();
                            file_name.push_str("... ");
                        } else {
                            file_name.push_str(vec![' '; max_len - file_name.len()].iter().collect::<String>().as_str());
                        }

                        str.push_str(file_name.as_str());

                        let mut btn = Button::new(str.as_str());
                        btn.create_control(left_frame, &mut self.screen_buf, &mut self.logger, &self.input);

                        buttons.push(Box::new(btn));
                    }
                }
            }
            left_frame.add_control(buttons);
            left_frame.draw(&Rect::default(), &mut self.screen_buf);
        }
        layout.close_frame();


        let editor_frame = layout.open_frame(Frame::new(EFrameAxis::Horizontal, false));
        {
            editor_frame.fill(&root_rect);
            editor_frame.draw(&Rect::default(), &mut self.screen_buf);
        }
        layout.close_frame();

        let action = self.pop_up.draw( &mut self.screen_buf, &mut self.logger, &self.input);



        return action;
    }
}