use crate::fs::FileSystem;
use crate::logger::FileLogger;
use crate::text_buffer::TextBufRecoveryState;
use serde::{Deserialize, Serialize};
use std::fs::File;

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct RecoveryDocument {
    pub path: String,
    pub text: String,
    pub buffer_state: TextBufRecoveryState,
}

impl Default for RecoveryDocument {
    fn default() -> Self {
        Self {
            path: String::new(),
            text: String::new(),
            buffer_state: TextBufRecoveryState::default(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct RecoverySnapshot {
    pub current_path: String,
    pub documents: Vec<RecoveryDocument>,
}

pub struct RecoveryStore;

impl RecoveryStore {
    pub fn load(logger: &mut FileLogger) -> Option<RecoverySnapshot> {
        let path = FileSystem::get_recovery_file_path();
        if !path.exists() {
            return None;
        }

        let file = match File::open(&path) {
            Ok(file) => file,
            Err(error) => {
                logger.log(format!(
                    "Failed to open recovery snapshot {}: {}",
                    path.display(),
                    error
                ));
                return None;
            }
        };

        match serde_json::from_reader::<_, RecoverySnapshot>(file) {
            Ok(snapshot) => Some(snapshot),
            Err(error) => {
                logger.log(format!(
                    "Failed to parse recovery snapshot {}: {}",
                    path.display(),
                    error
                ));
                None
            }
        }
    }

    pub fn save(snapshot: &RecoverySnapshot, logger: &mut FileLogger) {
        let path = FileSystem::get_recovery_file_path();
        if let Some(parent) = path.parent() {
            if let Err(error) = std::fs::create_dir_all(parent) {
                logger.log(format!(
                    "Failed to create recovery dir {}: {}",
                    parent.display(),
                    error
                ));
                return;
            }
        }

        let file = match File::create(&path) {
            Ok(file) => file,
            Err(error) => {
                logger.log(format!(
                    "Failed to create recovery snapshot {}: {}",
                    path.display(),
                    error
                ));
                return;
            }
        };

        if let Err(error) = serde_json::to_writer(file, snapshot) {
            logger.log(format!(
                "Failed to write recovery snapshot {}: {}",
                path.display(),
                error
            ));
        }
    }

    pub fn clear(logger: &mut FileLogger) {
        let path = FileSystem::get_recovery_file_path();
        if !path.exists() {
            return;
        }

        if let Err(error) = std::fs::remove_file(&path) {
            logger.log(format!(
                "Failed to remove recovery snapshot {}: {}",
                path.display(),
                error
            ));
        }
    }
}
