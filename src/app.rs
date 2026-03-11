use crate::config::Config;
use crate::controls::t_render::Render;
use crate::app_actions::AppActions;
use crate::e_actions::Action;
use crate::fs::FileSystem;
use crate::input::{EInputMode, EKeyCommand, Input};
use crate::logger::FileLogger;
use crate::panels::files_panel::FilesFrame;
use crate::panels::menu_panel::{LayoutPanel, MenuFrame};
use crate::panels::pop_up_panel::PopUpPanelFrame;
use crate::panels::search_panel::SearchPanelFrame;
use crate::panels::text_editor_panel::TextEditorFrame;
use crate::screen_buffer::ScreenBuf;
use crate::shortcuts::ShortcutMap;
use crate::terminal::Terminal;
use crate::text_buffer::TextBuf;
use crate::ui::c_layout::Layout;
use crate::ui::c_rect::Rect;
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers, ModifierKeyCode, MouseButton, MouseEventKind};
use crossterm::{event, execute};
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

pub struct App{
    pub logger: FileLogger,
    pub fs: FileSystem,
    pub config: Config,
    pub screen_buf: ScreenBuf,
    pub input: Input,
    pub pop_up: PopUpPanelFrame,
    pub search_panel: SearchPanelFrame,
    pub text_buffer: TextBuf,
    pub(crate) current_file_path: PathBuf,
    pub(crate) saved_version: u64,
    pub(crate) should_exit: bool,
}

impl App{
    pub fn new(start_path: Option<PathBuf>) -> App{

        let mut logger = FileLogger::new();
        let fs = FileSystem::new();
        let mut config: Config = Config::new(&mut logger);
        let (x, y) = config.get_win_size();
        let screen_buf = ScreenBuf::new(x, y);
        let input = Input::new(ShortcutMap::from_bindings(config.hotkeys(), &mut logger));
        let current_file_path = FileSystem::next_new_document_path(None);
        if config.get_last_files().is_empty() && start_path.is_none() {
            config.ensure_last_file(current_file_path.to_string_lossy().as_ref(), &mut logger);
        }
        logger.log("App created");
        let mut app = Self{
            logger,
            fs,
            config,
            screen_buf,
            input,
            pop_up: PopUpPanelFrame::new(),
            search_panel: SearchPanelFrame::new(),
            text_buffer: TextBuf::default(),
            current_file_path,
            saved_version: 0,
            should_exit: false,
        };
        app.input.change_mode(EInputMode::TextEditor);
        if let Some(path) = start_path {
            AppActions::open_document(&mut app, path);
        }
        app
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

            if self.input.mode == EInputMode::TextEditor
                && self.input.mouse_down.is_some()
                && self.input.text_mouse_anchor.is_some()
            {
                dirty = true;
            }

            if !dirty {
                continue;
            }

            self.screen_buf.clear();
            self.draw_ui();

            execute!(term.out, Hide, MoveTo(0, 0))?;
            self.screen_buf.present(&mut term.out)?;
            execute!(term.out, Show, MoveTo(self.input.cursor_x, self.input.cursor_y))?;
            term.out.flush()?;

            if self.should_exit {
                break Ok(());
            }

            if self.input.mouse_released.is_some() {
                self.input.mouse_down = None;
                self.input.text_mouse_anchor = None;
            }
            self.input.clicked = None;
            self.input.mouse_released = None;
            self.input.mouse_scroll = None;
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
                match k.code {
                    KeyCode::Modifier(ModifierKeyCode::LeftShift)
                    | KeyCode::Modifier(ModifierKeyCode::RightShift) => {
                        self.input.is_shift = k.kind != KeyEventKind::Release;
                    }
                    KeyCode::Modifier(ModifierKeyCode::LeftControl)
                    | KeyCode::Modifier(ModifierKeyCode::RightControl) => {
                        self.input.is_ctrl = k.kind != KeyEventKind::Release;
                    }
                    KeyCode::Modifier(ModifierKeyCode::LeftAlt)
                    | KeyCode::Modifier(ModifierKeyCode::RightAlt) => {
                        self.input.is_alt = k.kind != KeyEventKind::Release;
                    }
                    _ => {
                        self.input.is_shift = k.modifiers.contains(KeyModifiers::SHIFT);
                        self.input.is_ctrl = k.modifiers.contains(KeyModifiers::CONTROL);
                        self.input.is_alt = k.modifiers.contains(KeyModifiers::ALT);
                    }
                }

                if self.search_panel.active && k.kind == KeyEventKind::Press && k.code == KeyCode::Esc {
                    self.search_panel.close(&mut self.input, &mut self.text_buffer);
                    return true;
                }

                if k.kind == KeyEventKind::Press
                    && self.input.consume_paste_suppression_key(k.code, k.modifiers)
                {
                    return false;
                }

                match k.kind {
                    KeyEventKind::Press => {
                        match self.input.mode {
                            EInputMode::SearchQueryEditor | EInputMode::SearchReplaceEditor => {
                                self.input.handle_input(
                                    k.code,
                                    k.modifiers,
                                    &mut self.screen_buf,
                                    self.search_panel.active_buffer_mut(self.input.mode),
                                );
                            }
                            _ => {
                                self.input.handle_input(
                                    k.code,
                                    k.modifiers,
                                    &mut self.screen_buf,
                                    &mut self.text_buffer,
                                );
                            }
                        }

                        if let Some(action) = AppActions::action_from_key_command(self.input.key_command) {
                            AppActions::handle_action(self, action);
                            self.input.key_command = None;
                        } else if self.input.key_command == Some(EKeyCommand::Find) {
                            self.search_panel.open_find(&mut self.input, &mut self.text_buffer);
                            self.input.key_command = None;
                        }
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
                MouseEventKind::Moved => {
                    self.input.cursor_x = m.column.min(w.saturating_sub(1));
                    self.input.cursor_y = m.row.min(h.saturating_sub(1));
                    self.input.is_shift = self.input.is_shift || m.modifiers.contains(KeyModifiers::SHIFT);
                    self.input.is_ctrl = self.input.is_ctrl || m.modifiers.contains(KeyModifiers::CONTROL);
                    self.input.is_alt = m.modifiers.contains(KeyModifiers::ALT);
                    true
                }
                MouseEventKind::Down(MouseButton::Left) => {
                    self.input.cursor_x = m.column.min(w.saturating_sub(1));
                    self.input.cursor_y = m.row.min(h.saturating_sub(1));
                    self.input.is_shift = self.input.is_shift || m.modifiers.contains(KeyModifiers::SHIFT);
                    self.input.is_ctrl = self.input.is_ctrl || m.modifiers.contains(KeyModifiers::CONTROL);
                    self.input.is_alt = m.modifiers.contains(KeyModifiers::ALT);
                    self.input.clicked = Some((m.column, m.row));
                    self.input.mouse_down = Some((m.column, m.row));
                    self.input.mouse_released = None;
                    true
                }
                MouseEventKind::Up(MouseButton::Left) => {
                    self.input.cursor_x = m.column.min(w.saturating_sub(1));
                    self.input.cursor_y = m.row.min(h.saturating_sub(1));
                    self.input.is_shift = self.input.is_shift || m.modifiers.contains(KeyModifiers::SHIFT);
                    self.input.is_ctrl = self.input.is_ctrl || m.modifiers.contains(KeyModifiers::CONTROL);
                    self.input.is_alt = m.modifiers.contains(KeyModifiers::ALT);
                    self.input.mouse_released = Some((m.column, m.row));
                    true
                }
                MouseEventKind::Drag(MouseButton::Left) => {
                    self.input.cursor_x = m.column.min(w.saturating_sub(1));
                    self.input.cursor_y = m.row.min(h.saturating_sub(1));
                    self.input.is_shift = self.input.is_shift || m.modifiers.contains(KeyModifiers::SHIFT);
                    self.input.is_ctrl = self.input.is_ctrl || m.modifiers.contains(KeyModifiers::CONTROL);
                    self.input.is_alt = m.modifiers.contains(KeyModifiers::ALT);
                    true
                }
                MouseEventKind::ScrollUp => {
                    self.input.cursor_x = m.column.min(w.saturating_sub(1));
                    self.input.cursor_y = m.row.min(h.saturating_sub(1));
                    self.input.is_shift = self.input.is_shift || m.modifiers.contains(KeyModifiers::SHIFT);
                    self.input.is_ctrl = self.input.is_ctrl || m.modifiers.contains(KeyModifiers::CONTROL);
                    self.input.is_alt = m.modifiers.contains(KeyModifiers::ALT);
                    self.input.mouse_scroll = Some(if self.input.is_alt { (-3, 0) } else { (0, -3) });
                    true
                }
                MouseEventKind::ScrollDown => {
                    self.input.cursor_x = m.column.min(w.saturating_sub(1));
                    self.input.cursor_y = m.row.min(h.saturating_sub(1));
                    self.input.is_shift = self.input.is_shift || m.modifiers.contains(KeyModifiers::SHIFT);
                    self.input.is_ctrl = self.input.is_ctrl || m.modifiers.contains(KeyModifiers::CONTROL);
                    self.input.is_alt = m.modifiers.contains(KeyModifiers::ALT);
                    self.input.mouse_scroll = Some(if self.input.is_alt { (3, 0) } else { (0, 3) });
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    fn create_layout(&mut self, layout: &mut Layout){

        self.pop_up.create_layout(layout, &mut self.config);

        let mut menu_panel = MenuFrame::default();
        menu_panel.create_layout(layout, &mut self.config);


        let mut files_panel = FilesFrame::default();
        files_panel.set_current_document(
            self.current_file_path.clone(),
            AppActions::is_current_document_dirty(self),
            AppActions::is_current_document_virtual(self),
        );
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

        let search_consumed = self.search_panel.interact(
            root_rect,
            &mut self.input,
            &mut self.text_buffer,
            &mut self.logger,
        );
        let action = if search_consumed
            || self.search_panel.hit(root_rect, self.input.cursor_x, self.input.cursor_y)
        {
            Action::None
        } else {
            layout.interact(&mut self.logger, &mut self.input, &mut self.pop_up, &mut self.text_buffer)
        };

        let refresh_layout = action != Action::None;
        AppActions::handle_action(self, action);

        if refresh_layout {
            layout = Layout::new(root_rect);
            self.create_layout(&mut layout);
        }

        if self.pop_up.active && self.pop_up.needs_layout() {
            self.pop_up.create_layout(&mut layout, &mut self.config);
        }

        layout.draw(&mut self.screen_buf, &mut self.pop_up, &mut self.logger, &mut self.text_buffer);
        self.search_panel.draw(root_rect, &mut self.screen_buf, &mut self.input, &mut self.text_buffer);
    }
}
