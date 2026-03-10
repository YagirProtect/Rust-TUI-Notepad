use crate::config::Config;
use crate::controls::t_render::Render;
use crate::e_actions::Action;
use crate::fs::FileSystem;
use crate::input::Input;
use crate::logger::FileLogger;
use crate::panels::files_panel::FilesFrame;
use crate::panels::menu_panel::{LayoutPanel, MenuFrame};
use crate::panels::pop_up_panel::PopUpPanelFrame;
use crate::panels::text_editor_panel::TextEditorFrame;
use crate::screen_buffer::ScreenBuf;
use crate::terminal::Terminal;
use crate::text_buffer::TextBuf;
use crate::ui::c_layout::Layout;
use crate::ui::c_rect::Rect;
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{Event, KeyEventKind, MouseButton, MouseEventKind};
use crossterm::{event, execute};
use std::io::Write;
use std::time::Duration;

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
                loop {
                    let ev = event::read()?;
                    dirty |= self.handle_event(ev, &mut w, &mut h);
                    if !event::poll(Duration::from_millis(0))? {
                        break;
                    }
                }
            }

            if !dirty {
                continue;
            }

            self.screen_buf.clear();

            let action = self.draw_ui();
            self.handle_action(action);

            execute!(term.out, Hide, MoveTo(0, 0))?;
            self.screen_buf.present(&mut term.out)?;
            execute!(term.out, Show, MoveTo(self.input.cursor_x, self.input.cursor_y))?;
            term.out.flush()?;

            self.input.clicked = None;
            self.input.pending_text.clear();
            self.input.text_cursor_move = None;
            self.input.key_command = None;
        }
    }

    fn handle_event(&mut self, ev: Event, w: &mut u16, h: &mut u16) -> bool {
        match ev {
            Event::Paste(text) => {
                self.input.arm_paste_suppression(&text);
                self.input.pending_text.push_str(&text);
                true
            }
            Event::Key(k) => {
                if k.kind == KeyEventKind::Press
                    && self.input.consume_paste_suppression_key(k.code, k.modifiers)
                {
                    return false;
                }

                match k.kind {
                    KeyEventKind::Press => {
                        self.input.handle_input(k.code, k.modifiers, &mut self.screen_buf, &mut self.text_buffer);
                        true
                    }
                    KeyEventKind::Repeat => false,
                    KeyEventKind::Release => true,
                }
            }
            Event::Resize(nw, nh) => {
                *w = nw;
                *h = nh;
                self.screen_buf.resize(*w, *h);

                if *w > 0 { self.input.cursor_x = self.input.cursor_x.min(*w - 1); }
                if *h > 0 { self.input.cursor_y = self.input.cursor_y.min(*h - 1); }

                true
            }
            Event::Mouse(m) => match m.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    self.input.cursor_x = m.column.min(w.saturating_sub(1));
                    self.input.cursor_y = m.row.min(h.saturating_sub(1));
                    self.input.clicked = Some((m.column, m.row));
                    true
                }
                MouseEventKind::Up(MouseButton::Left) => true,
                _ => false,
            },
            _ => false,
        }
    }

    fn handle_action(&mut self, action: Action) {
        match action {
            Action::None => {}
            Action::Copy => {
                self.text_buffer.copy_selection();
            }
            Action::Cut => {
                self.text_buffer.cut_selection();
            }
            Action::Paste => {
                self.text_buffer.paste_from_clipboard();
            }
            Action::Undo => {
                self.text_buffer.undo();
            }
            Action::Redo => {
                self.text_buffer.redo();
            }
            Action::NewFile => {
                self.text_buffer = TextBuf::default();
            }
            Action::OpenFile | Action::SaveFile | Action::Delete | Action::Find | Action::Replace | Action::FAQ => {
                self.logger.log(format!("Unhandled action: {:?}", action));
            }
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

    fn draw_ui(&mut self) -> Action {
        let root_rect = Rect::new(0, 0, self.screen_buf.w, self.screen_buf.h);
        let mut layout = Layout::new(root_rect);
        self.create_layout(&mut layout);

        let action = layout.interact(&mut self.logger, &mut self.input, &mut self.pop_up, &mut self.text_buffer);
        layout.draw(&mut self.screen_buf, &mut self.pop_up, &mut self.logger, &mut self.text_buffer);
        action
    }
}
