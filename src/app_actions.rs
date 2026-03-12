use crate::app::{App, OpenDocumentState};
use crate::app_dialogs::AppDialogs;
use crate::e_actions::Action;
use crate::fs::FileSystem;
use crate::input::{EInputMode, EKeyCommand};
use crate::recovery_store::RecoveryStore;
use crate::text_buffer::TextBuf;
use std::path::PathBuf;
use std::process::Command;

const FAQ_URL: &str = "https://github.com/YagirProtect/NOTEPAD";

pub struct AppActions;

impl AppActions {
    pub fn handle_action(app: &mut App, action: Action) {
        if action != Action::None && !matches!(action, Action::SetFilesTabsScroll(_)) {
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
                if Self::is_current_document_unsaved_empty(app) {
                    Self::close_search_if_needed(app);
                    app.input.change_mode(EInputMode::TextEditor);
                } else {
                    Self::stash_current_document(app);
                    Self::new_document(app);
                }
            }
            Action::Exit => {
                if Self::confirm_all_dirty_documents(app) {
                    let current_path = app.current_file_path.clone();
                    Self::cleanup_empty_managed_new_document(app, &current_path);
                    RecoveryStore::clear(&mut app.logger);
                    app.should_exit = true;
                }
            }
            Action::OpenFile => {
                AppDialogs::open_file_dialog(app);
            }
            Action::OpenInExplorer => {
                Self::open_current_in_file_manager(app);
            }
            Action::OpenPath(path) => {
                Self::open_document(app, path);
            }
            Action::RemoveRecentPath(path) => {
                Self::remove_recent_path(app, path);
            }
            Action::SetFilesTabsScroll(scroll_x) => {
                app.files_tabs_scroll_x = scroll_x;
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
        !app.current_file_path.exists() && Self::is_managed_new_document_path(&app.current_file_path)
    }

    pub fn is_current_document_unsaved_empty(app: &App) -> bool {
        !app.current_file_path.exists() && app.text_buffer.text().is_empty()
    }

    pub fn open_document(app: &mut App, path: PathBuf) {
        Self::open_document_internal(app, path, true);
    }

    fn open_document_internal(app: &mut App, path: PathBuf, stash_current: bool) {
        if path == app.current_file_path {
            return;
        }

        let previous_path = app.current_file_path.clone();
        Self::close_search_if_needed(app);
        if stash_current {
            Self::stash_current_document(app);
        }

        if let Some(state) = app.open_documents.remove(&path) {
            app.text_buffer = state.text_buffer;
            app.saved_version = state.saved_version;
        } else {
            if path.exists() {
                match FileSystem::read_text_file(&path) {
                    Ok(text) => {
                        app.text_buffer.load_text(&text);
                        app.saved_version = app.text_buffer.version();
                    }
                    Err(error) => {
                        if let Some(previous_state) = app.open_documents.remove(&previous_path) {
                            app.text_buffer = previous_state.text_buffer;
                            app.saved_version = previous_state.saved_version;
                            app.current_file_path = previous_path;
                        }
                        app.logger
                            .log(format!("Failed to read {}: {}", path.display(), error));
                        return;
                    }
                }
            } else {
                app.text_buffer = TextBuf::default();
                app.saved_version = app.text_buffer.version();
            }
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
        let previous_path = app.current_file_path.clone();
        match FileSystem::write_text_file(&path, &app.text_buffer.text()) {
            Ok(_) => {
                app.current_file_path = path;
                app.saved_version = app.text_buffer.version();
                app.open_documents.remove(&previous_path);
                app.open_documents.remove(&app.current_file_path);
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
        Self::new_document_with_options(app, true);
    }

    fn new_document_with_options(app: &mut App, avoid_current_path: bool) {
        Self::close_search_if_needed(app);
        app.text_buffer = TextBuf::default();
        app.current_file_path = Self::next_available_new_document_path(app, avoid_current_path);
        app.saved_version = app.text_buffer.version();
        app.config
            .ensure_last_file(app.current_file_path.to_string_lossy().as_ref(), &mut app.logger);
        app.input.change_mode(EInputMode::TextEditor);
    }

    fn next_available_new_document_path(app: &App, avoid_current_path: bool) -> PathBuf {
        let mut seed: Option<PathBuf> = avoid_current_path.then(|| app.current_file_path.clone());
        loop {
            let candidate = FileSystem::next_new_document_path(seed.as_ref());
            let candidate_str = candidate.to_string_lossy().to_string();
            let used_in_tabs = app
                .config
                .get_last_files()
                .iter()
                .any(|value| value == &candidate_str);
            if !used_in_tabs {
                return candidate;
            }

            seed = Some(candidate);
        }
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

    fn remove_recent_path(app: &mut App, path: PathBuf) {
        let path_str = path.to_string_lossy().to_string();
        let recent_before = app.config.get_last_files().clone();
        let removed_index = recent_before
            .iter()
            .position(|value| value == &path_str);
        let is_current = path == app.current_file_path;

        if is_current && !AppDialogs::confirm_document_switch(app) {
            return;
        }

        Self::cleanup_empty_managed_new_document(app, &path);
        app.config.remove_last_file(&path_str, &mut app.logger);
        app.open_documents.remove(&path);

        if !is_current {
            return;
        }

        let fallback_path = removed_index.and_then(|index| {
            if index + 1 < recent_before.len() {
                return Some(PathBuf::from(&recent_before[index + 1]));
            }
            if index > 0 {
                return Some(PathBuf::from(&recent_before[index - 1]));
            }
            None
        });

        if let Some(next_path) = fallback_path {
            Self::open_document_internal(app, next_path, false);
        } else {
            Self::new_document_with_options(app, false);
        }
    }

    fn cleanup_empty_managed_new_document(app: &mut App, path: &PathBuf) {
        if !Self::is_managed_new_document_path(path) {
            return;
        }

        let mut should_remove_from_recent = false;
        let mut should_remove_file = false;

        if path == &app.current_file_path {
            if app.text_buffer.text().is_empty() {
                should_remove_from_recent = true;
                should_remove_file = path.exists();
            }
        } else if let Some(state) = app.open_documents.get(path) {
            if state.text_buffer.text().is_empty() {
                should_remove_from_recent = true;
                should_remove_file = path.exists();
            }
        } else if path.exists() {
            match FileSystem::read_text_file(path) {
                Ok(text) => {
                    if text.is_empty() {
                        should_remove_from_recent = true;
                        should_remove_file = true;
                    }
                }
                Err(error) => {
                    app.logger.log(format!(
                        "Failed to read candidate new document {}: {}",
                        path.display(),
                        error
                    ));
                }
            }
        }

        if !should_remove_from_recent {
            return;
        }

        if should_remove_file {
            if let Err(error) = std::fs::remove_file(path) {
                app.logger.log(format!(
                    "Failed to remove empty managed new document {}: {}",
                    path.display(),
                    error
                ));
            }
        }

        let path_str = path.to_string_lossy().to_string();
        app.config.remove_last_file(&path_str, &mut app.logger);
        app.open_documents.remove(path);
    }

    fn stash_current_document(app: &mut App) {
        let current_path = app.current_file_path.clone();
        Self::cleanup_empty_managed_new_document(app, &current_path);

        let current_path_str = current_path.to_string_lossy().to_string();
        if !app
            .config
            .get_last_files()
            .iter()
            .any(|value| value == &current_path_str)
        {
            return;
        }

        let state = OpenDocumentState {
            text_buffer: std::mem::take(&mut app.text_buffer),
            saved_version: app.saved_version,
        };
        app.open_documents.insert(current_path, state);
        app.saved_version = 0;
    }

    fn is_document_dirty(app: &App, path: &PathBuf) -> bool {
        if path == &app.current_file_path {
            return Self::is_current_document_dirty(app);
        }

        app.open_documents
            .get(path)
            .is_some_and(|state| state.text_buffer.version() != state.saved_version)
    }

    fn confirm_all_dirty_documents(app: &mut App) -> bool {
        let mut ordered_paths: Vec<PathBuf> = Vec::new();

        for value in app.config.get_last_files() {
            let path = PathBuf::from(value);
            if Self::is_document_dirty(app, &path) {
                ordered_paths.push(path);
            }
        }

        for path in app.open_documents.keys() {
            if Self::is_document_dirty(app, path) && !ordered_paths.iter().any(|value| value == path) {
                ordered_paths.push(path.clone());
            }
        }

        for path in ordered_paths {
            if path != app.current_file_path {
                Self::open_document_internal(app, path.clone(), true);
            }

            if !AppDialogs::confirm_document_switch(app) {
                return false;
            }
        }

        true
    }

    pub(crate) fn is_managed_new_document_path(path: &PathBuf) -> bool {
        let documents_dir = FileSystem::get_documents_dir();
        if !path.starts_with(&documents_dir) {
            return false;
        }

        let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
            return false;
        };

        file_name.starts_with("New Document") && file_name.ends_with(".txt")
    }
}
