use crate::app::App;
use crate::app_dialogs::AppDialogs;
use crate::e_actions::Action;
use crate::fs::FileSystem;
use crate::input::{EInputMode, EKeyCommand};
use crate::text_buffer::TextBuf;
use std::path::PathBuf;
use std::process::Command;

const FAQ_URL: &str = "https://github.com/YagirProtect/NOTEPAD";

pub struct AppActions;

impl AppActions {
    pub fn handle_action(app: &mut App, action: Action) {
        if action != Action::None {
            app.pop_up.hide();
        }

        match action {
            Action::None => {}
            Action::Copy => {
                app.text_buffer.copy_selection();
            }
            Action::Cut => {
                app.text_buffer.cut_selection();
            }
            Action::Paste => {
                app.text_buffer.paste_from_clipboard();
            }
            Action::Find => {
                app.search_panel.open_find(&mut app.input, &mut app.text_buffer);
            }
            Action::Undo => {
                app.text_buffer.undo();
            }
            Action::Redo => {
                app.text_buffer.redo();
            }
            Action::Replace => {
                app.search_panel.open_replace(&mut app.input, &mut app.text_buffer);
            }
            Action::ToggleKeywordHighlight => {
                app.config.toggle_highlight_keywords(&mut app.logger);
            }
            Action::NewFile => {
                if AppDialogs::confirm_document_switch(app) {
                    Self::new_document(app);
                }
            }
            Action::Exit => {
                if AppDialogs::confirm_document_switch(app) {
                    app.should_exit = true;
                }
            }
            Action::OpenFile => {
                if AppDialogs::confirm_document_switch(app) {
                    AppDialogs::open_file_dialog(app);
                }
            }
            Action::OpenInExplorer => {
                Self::open_current_in_file_manager(app);
            }
            Action::OpenPath(path) => {
                if AppDialogs::confirm_document_switch(app) {
                    Self::open_document(app, path);
                }
            }
            Action::RemoveRecentPath(path) => {
                app.config
                    .remove_last_file(path.to_string_lossy().as_ref(), &mut app.logger);
            }
            Action::SaveFile => {
                Self::save_current_document(app);
            }
            Action::SaveFileAs => {
                AppDialogs::save_current_document_as(app);
            }
            Action::FAQ => {
                Self::open_faq(app);
            }
            Action::OpenUrl(url) => {
                Self::open_external_target(app, &url, "URL");
            }
            Action::Delete => {
                app.logger.log(format!("Unhandled action: {:?}", action));
            }
        }
    }

    pub fn action_from_key_command(command: Option<EKeyCommand>) -> Option<Action> {
        match command? {
            EKeyCommand::NewFile => Some(Action::NewFile),
            EKeyCommand::OpenFile => Some(Action::OpenFile),
            EKeyCommand::OpenInExplorer => Some(Action::OpenInExplorer),
            EKeyCommand::SaveFile => Some(Action::SaveFile),
            EKeyCommand::SaveFileAs => Some(Action::SaveFileAs),
            EKeyCommand::Replace => Some(Action::Replace),
            _ => None,
        }
    }

    pub fn is_current_document_dirty(app: &App) -> bool {
        app.text_buffer.version() != app.saved_version
    }

    pub fn is_current_document_virtual(app: &App) -> bool {
        !app.current_file_path.exists() && app.saved_version == 0 && app.text_buffer.version() == 0
    }

    pub fn open_document(app: &mut App, path: PathBuf) {
        if path == app.current_file_path {
            return;
        }

        Self::close_search_if_needed(app);

        if path.exists() {
            match FileSystem::read_text_file(&path) {
                Ok(text) => {
                    app.text_buffer.load_text(&text);
                    app.saved_version = app.text_buffer.version();
                }
                Err(error) => {
                    app.logger
                        .log(format!("Failed to read {}: {}", path.display(), error));
                    return;
                }
            }
        } else {
            app.text_buffer = TextBuf::default();
            app.saved_version = app.text_buffer.version();
        }

        app.current_file_path = path;
        app.config
            .ensure_last_file(app.current_file_path.to_string_lossy().as_ref(), &mut app.logger);
        app.input.change_mode(EInputMode::TextEditor);
    }

    pub fn save_current_document(app: &mut App) -> bool {
        let path = app.current_file_path.clone();
        Self::save_document_to(app, path)
    }

    pub fn save_document_to(app: &mut App, path: PathBuf) -> bool {
        match FileSystem::write_text_file(&path, &app.text_buffer.text()) {
            Ok(_) => {
                app.current_file_path = path;
                app.saved_version = app.text_buffer.version();
                app.config.push_last_file(
                    app.current_file_path.to_string_lossy().as_ref(),
                    &mut app.logger,
                );
                true
            }
            Err(error) => {
                app.logger.log(format!(
                    "Failed to save {}: {}",
                    path.display(),
                    error
                ));
                false
            }
        }
    }

    fn new_document(app: &mut App) {
        Self::close_search_if_needed(app);
        app.text_buffer = TextBuf::default();
        app.current_file_path = FileSystem::next_new_document_path(Some(&app.current_file_path));
        app.saved_version = app.text_buffer.version();
        app.config
            .ensure_last_file(app.current_file_path.to_string_lossy().as_ref(), &mut app.logger);
        app.input.change_mode(EInputMode::TextEditor);
    }

    fn open_current_in_file_manager(app: &mut App) {
        #[cfg(target_os = "windows")]
        {
            let result = if app.current_file_path.exists() {
                Command::new("explorer.exe")
                    .arg("/select,")
                    .arg(&app.current_file_path)
                    .spawn()
            } else {
                let dir = app
                    .current_file_path
                    .parent()
                    .map(PathBuf::from)
                    .unwrap_or_else(FileSystem::get_documents_dir);
                Command::new("explorer.exe").arg(dir).spawn()
            };

            if let Err(error) = result {
                app.logger.log(format!("Failed to open Explorer: {}", error));
            }
            return;
        }

        #[cfg(target_os = "linux")]
        {
            let target = app
                .current_file_path
                .parent()
                .map(PathBuf::from)
                .unwrap_or_else(FileSystem::get_documents_dir);

            if let Err(error) = Command::new("xdg-open").arg(target).spawn() {
                app.logger.log(format!("Failed to open file manager: {}", error));
            }
            return;
        }

        #[cfg(target_os = "macos")]
        {
            let result = if app.current_file_path.exists() {
                Command::new("open").arg("-R").arg(&app.current_file_path).spawn()
            } else {
                let dir = app
                    .current_file_path
                    .parent()
                    .map(PathBuf::from)
                    .unwrap_or_else(FileSystem::get_documents_dir);
                Command::new("open").arg(dir).spawn()
            };

            if let Err(error) = result {
                app.logger.log(format!("Failed to open Finder: {}", error));
            }
        }
    }

    fn open_faq(app: &mut App) {
        Self::open_external_target(app, FAQ_URL, "FAQ");
    }

    fn open_external_target(app: &mut App, target: &str, label: &str) {
        #[cfg(target_os = "windows")]
        {
            if let Err(error) = Command::new("explorer.exe").arg(target).spawn() {
                app.logger.log(format!("Failed to open {}: {}", label, error));
            }
            return;
        }

        #[cfg(target_os = "linux")]
        {
            if let Err(error) = Command::new("xdg-open").arg(target).spawn() {
                app.logger.log(format!("Failed to open {}: {}", label, error));
            }
            return;
        }

        #[cfg(target_os = "macos")]
        {
            if let Err(error) = Command::new("open").arg(target).spawn() {
                app.logger.log(format!("Failed to open {}: {}", label, error));
            }
        }
    }

    fn close_search_if_needed(app: &mut App) {
        if app.search_panel.active {
            app.search_panel.close(&mut app.input, &mut app.text_buffer);
        }
    }
}
