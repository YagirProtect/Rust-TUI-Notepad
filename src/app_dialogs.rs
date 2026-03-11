use crate::app::App;
use crate::app_actions::AppActions;
use rfd::{FileDialog, MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};

pub struct AppDialogs;

impl AppDialogs {
    pub fn open_file_dialog(app: &mut App) {
        let mut dialog = FileDialog::new();
        if let Some(parent) = app.current_file_path.parent() {
            dialog = dialog.set_directory(parent);
        }

        if let Some(path) = dialog.pick_file() {
            AppActions::open_document(app, path);
        }
    }

    pub fn save_current_document_as(app: &mut App) -> bool {
        let mut dialog = FileDialog::new();
        if let Some(parent) = app.current_file_path.parent() {
            dialog = dialog.set_directory(parent);
        }
        if let Some(file_name) = app.current_file_path.file_name().and_then(|value| value.to_str()) {
            dialog = dialog.set_file_name(file_name);
        }

        if let Some(path) = dialog.save_file() {
            return AppActions::save_document_to(app, path);
        }

        false
    }

    pub fn confirm_document_switch(app: &mut App) -> bool {
        if !AppActions::is_current_document_dirty(app) {
            return true;
        }

        let result = MessageDialog::new()
            .set_level(MessageLevel::Warning)
            .set_title("Unsaved changes")
            .set_description("Current document has unsaved changes.\nSave before closing it?")
            .set_buttons(MessageButtons::YesNoCancel)
            .show();

        match result {
            MessageDialogResult::Yes => {
                if AppActions::is_current_document_virtual(app) {
                    Self::save_current_document_as(app)
                } else {
                    AppActions::save_current_document(app)
                }
            }
            MessageDialogResult::No => true,
            MessageDialogResult::Cancel => false,
            MessageDialogResult::Ok => true,
            MessageDialogResult::Custom(_) => false,
        }
    }
}
