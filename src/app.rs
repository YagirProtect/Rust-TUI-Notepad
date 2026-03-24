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
use crate::recovery_store::{RecoveryDocument, RecoverySnapshot, RecoveryStore};
use crate::panels::text_editor_panel::TextEditorFrame;
use crate::screen_buffer::ScreenBuf;
use crate::shortcuts::ShortcutMap;
use crate::terminal::Terminal;
use crate::text_buffer::TextBuf;
use crate::ui::c_layout::Layout;
use crate::ui::c_rect::Rect;
use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers, ModifierKeyCode, MouseButton, MouseEventKind};
use crossterm::terminal::SetTitle;
use crossterm::{event, execute};
use std::hash::{Hash, Hasher};
use std::collections::{HashMap, HashSet};
use std::collections::hash_map::DefaultHasher;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub struct OpenDocumentState {
    pub text_buffer: TextBuf,
    pub saved_version: u64,
}

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
    pub(crate) files_tabs_scroll_x: u16,
    pub(crate) last_terminal_title: String,
    pub(crate) open_documents: HashMap<PathBuf, OpenDocumentState>,
    pub(crate) last_recovery_signature: u64,
    pub(crate) last_recovery_write_at: Instant,
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
        if start_path.is_none() {
            let current_path_str = current_file_path.to_string_lossy().to_string();
            if !config
                .get_last_files()
                .iter()
                .any(|value| value == &current_path_str)
            {
                config.ensure_last_file(&current_path_str, &mut logger);
            }
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
            files_tabs_scroll_x: 0,
            last_terminal_title: String::new(),
            open_documents: HashMap::new(),
            last_recovery_signature: 0,
            last_recovery_write_at: Instant::now(),
        };
        app.input.change_mode(EInputMode::TextEditor);
        if let Some(path) = start_path {
            AppActions::open_document(&mut app, path);
        } else {
            app.restore_recovery_if_available();
        }
        app
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (_guard, mut term) = Terminal::enter()?;
        self.refresh_terminal_title(&mut term.out)?;
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

            self.maybe_persist_recovery();

            if self.input.mouse_down.is_some()
                && (self.input.mode == EInputMode::TextEditor || self.input.is_search_mode())
            {
                dirty = true;
            }

            if !dirty {
                continue;
            }

            self.screen_buf.clear();
            self.draw_ui();
            self.refresh_terminal_title(&mut term.out)?;

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
            self.input.middle_clicked = None;
            self.input.double_clicked = None;
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
                    if self.input.register_left_click(m.column, m.row) {
                        self.input.double_clicked = Some((m.column, m.row));
                    }
                    self.input.mouse_down = Some((m.column, m.row));
                    self.input.mouse_released = None;
                    true
                }
                MouseEventKind::Down(MouseButton::Middle) => {
                    self.input.cursor_x = m.column.min(w.saturating_sub(1));
                    self.input.cursor_y = m.row.min(h.saturating_sub(1));
                    self.input.is_shift = self.input.is_shift || m.modifiers.contains(KeyModifiers::SHIFT);
                    self.input.is_ctrl = self.input.is_ctrl || m.modifiers.contains(KeyModifiers::CONTROL);
                    self.input.is_alt = m.modifiers.contains(KeyModifiers::ALT);
                    self.input.middle_clicked = Some((m.column, m.row));
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
        let mut dirty_paths: HashSet<PathBuf> = HashSet::new();
        let mut virtual_paths: HashSet<PathBuf> = HashSet::new();

        if AppActions::is_current_document_dirty(self) {
            dirty_paths.insert(self.current_file_path.clone());
        }
        if !self.current_file_path.exists()
            && (AppActions::is_current_document_virtual(self)
                || AppActions::is_current_document_dirty(self))
        {
            virtual_paths.insert(self.current_file_path.clone());
        }

        for (path, state) in &self.open_documents {
            let is_dirty = state.text_buffer.version() != state.saved_version;
            if is_dirty {
                dirty_paths.insert(path.clone());
            }
            if !path.exists()
                && (AppActions::is_managed_new_document_path(path) || is_dirty)
            {
                virtual_paths.insert(path.clone());
            }
        }

        files_panel.set_current_document(
            self.current_file_path.clone(),
            dirty_paths,
            virtual_paths,
        );
        files_panel.set_scroll_x(self.files_tabs_scroll_x);
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

    fn desired_terminal_title(&self) -> String {
        let file_name = self
            .current_file_path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("NOTEPAD");
        let dirty_marker = if AppActions::is_current_document_dirty(self) {
            "*"
        } else {
            ""
        };
        format!("{}{} - NOTEPAD", dirty_marker, file_name)
    }

    fn refresh_terminal_title<W: Write>(&mut self, out: &mut W) -> std::io::Result<()> {
        let title = self.desired_terminal_title();
        if self.last_terminal_title == title {
            return Ok(());
        }

        execute!(out, SetTitle(&title))?;
        self.last_terminal_title = title;
        Ok(())
    }

    fn restore_recovery_if_available(&mut self) {
        let Some(snapshot) = RecoveryStore::load(&mut self.logger) else {
            return;
        };

        if snapshot.documents.is_empty() {
            RecoveryStore::clear(&mut self.logger);
            return;
        }

        let startup_path = self.current_file_path.to_string_lossy().to_string();
        if !snapshot.documents.iter().any(|doc| doc.path == startup_path) {
            self.config.remove_last_file(&startup_path, &mut self.logger);
        }

        self.open_documents.clear();
        let mut loaded_documents: Vec<(PathBuf, OpenDocumentState)> = Vec::new();

        for doc in snapshot.documents {
            let mut text_buffer = TextBuf::default();
            text_buffer.load_text(&doc.text);
            text_buffer.apply_recovery_state(doc.buffer_state);
            let saved_version = text_buffer.version().saturating_sub(1);
            let path = PathBuf::from(&doc.path);
            self.config.ensure_last_file(&doc.path, &mut self.logger);

            loaded_documents.push((
                path,
                OpenDocumentState {
                    text_buffer,
                    saved_version,
                },
            ));
        }

        if loaded_documents.is_empty() {
            RecoveryStore::clear(&mut self.logger);
            return;
        }

        let current_index = loaded_documents
            .iter()
            .position(|(path, _)| path.to_string_lossy().as_ref() == snapshot.current_path)
            .unwrap_or(0);

        for (index, (path, state)) in loaded_documents.into_iter().enumerate() {
            if index == current_index {
                self.current_file_path = path;
                self.text_buffer = state.text_buffer;
                self.saved_version = state.saved_version;
            } else {
                self.open_documents.insert(path, state);
            }
        }

        RecoveryStore::clear(&mut self.logger);
        self.input.change_mode(EInputMode::TextEditor);
        self.logger.log("Recovery snapshot restored");
    }

    fn maybe_persist_recovery(&mut self) {
        const RECOVERY_FLUSH_INTERVAL: Duration = Duration::from_millis(700);
        if self.last_recovery_write_at.elapsed() < RECOVERY_FLUSH_INTERVAL {
            return;
        }

        let mut documents = Vec::new();
        if AppActions::is_current_document_dirty(self) {
            documents.push(RecoveryDocument {
                path: self.current_file_path.to_string_lossy().to_string(),
                text: self.text_buffer.text(),
                buffer_state: self.text_buffer.recovery_state(),
            });
        }

        let mut paths: Vec<PathBuf> = self.open_documents.keys().cloned().collect();
        paths.sort_by_key(|path| path.to_string_lossy().to_string());
        for path in paths {
            let Some(state) = self.open_documents.get(&path) else {
                continue;
            };
            if state.text_buffer.version() == state.saved_version {
                continue;
            }

            documents.push(RecoveryDocument {
                path: path.to_string_lossy().to_string(),
                text: state.text_buffer.text(),
                buffer_state: state.text_buffer.recovery_state(),
            });
        }

        if documents.is_empty() {
            if self.last_recovery_signature != 0 {
                RecoveryStore::clear(&mut self.logger);
                self.last_recovery_signature = 0;
            }
            self.last_recovery_write_at = Instant::now();
            return;
        }

        let current_path = self.current_file_path.to_string_lossy().to_string();
        let signature = Self::recovery_signature(&current_path, &documents);
        if self.last_recovery_signature == signature {
            return;
        }

        let snapshot = RecoverySnapshot {
            current_path,
            documents,
        };
        RecoveryStore::save(&snapshot, &mut self.logger);
        self.last_recovery_signature = signature;
        self.last_recovery_write_at = Instant::now();
    }

    fn recovery_signature(current_path: &str, documents: &[RecoveryDocument]) -> u64 {
        let mut hasher = DefaultHasher::new();
        current_path.hash(&mut hasher);
        for doc in documents {
            doc.path.hash(&mut hasher);
            doc.text.hash(&mut hasher);
            doc.buffer_state.hash(&mut hasher);
        }
        hasher.finish()
    }
}
