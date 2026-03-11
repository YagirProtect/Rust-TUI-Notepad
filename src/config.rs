use std::fs::File;
use std::io::BufWriter;

use serde::{Deserialize, Serialize};

use crate::fs::FileSystem;
use crate::logger::FileLogger;

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(default)]
pub struct HotkeyBinding {
    pub action: String,
    pub shortcut: String,
}

impl HotkeyBinding {
    pub fn new(action: &str, shortcut: &str) -> Self {
        Self {
            action: action.to_string(),
            shortcut: shortcut.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    last_files: Vec<String>,
    last_text: Vec<Vec<char>>,
    width: u16,
    height: u16,
    #[serde(default = "default_true")]
    highlight_keywords: bool,
    #[serde(default = "default_hotkeys")]
    hotkeys: Vec<HotkeyBinding>,
}

impl Config {
    pub fn get_lines_clone(&self) -> Vec<Vec<char>> {
        self.last_text.clone()
    }

    pub fn shortcuts_for(&self, action: &str) -> Vec<String> {
        self.hotkeys
            .iter()
            .filter(|binding| binding.action.eq_ignore_ascii_case(action))
            .map(|binding| binding.shortcut.clone())
            .collect()
    }

    pub fn shortcuts_label_for(&self, action: &str) -> String {
        self.shortcuts_for(action).join(" / ")
    }

    pub fn hotkeys(&self) -> &[HotkeyBinding] {
        &self.hotkeys
    }

    pub fn highlight_keywords(&self) -> bool {
        self.highlight_keywords
    }

    pub fn toggle_highlight_keywords(&mut self, logger: &mut FileLogger) {
        self.highlight_keywords = !self.highlight_keywords;
        self.save(logger);
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            last_files: vec![],
            last_text: vec![],
            width: 400,
            height: 200,
            highlight_keywords: true,
            hotkeys: default_hotkeys(),
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_hotkeys() -> Vec<HotkeyBinding> {
    vec![
        HotkeyBinding::new("find", "Ctrl+F"),
        HotkeyBinding::new("replace", "Ctrl+H"),
        HotkeyBinding::new("new_file", "Ctrl+N"),
        HotkeyBinding::new("open_file", "Ctrl+O"),
        HotkeyBinding::new("save_file", "Ctrl+S"),
        HotkeyBinding::new("save_file_as", "Ctrl+Shift+S"),
        HotkeyBinding::new("open_in_explorer", "Ctrl+E"),
        HotkeyBinding::new("undo", "Ctrl+Z"),
        HotkeyBinding::new("redo", "Ctrl+Shift+Z"),
        HotkeyBinding::new("redo", "Ctrl+Y"),
        HotkeyBinding::new("select_all", "Ctrl+A"),
        HotkeyBinding::new("copy", "Ctrl+C"),
        HotkeyBinding::new("cut", "Ctrl+X"),
        HotkeyBinding::new("paste", "Ctrl+V"),
        HotkeyBinding::new("paste", "Alt+V"),
        HotkeyBinding::new("paste", "Shift+Insert"),
    ]
}

impl Config {
    pub fn new(logger: &mut FileLogger) -> Self {
        if FileSystem::get_config_file_path().exists() {
            return Self::load_config();
        }

        let default = Config::default();
        let file = FileSystem::create_file(FileSystem::get_config_file_path());
        match serde_json::to_writer(BufWriter::new(file), &default) {
            Ok(_) => default,
            Err(error) => {
                logger.log(error.to_string());
                default
            }
        }
    }

    pub fn get_win_size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    pub fn get_last_files(&self) -> &Vec<String> {
        &self.last_files
    }

    pub fn push_last_file(&mut self, path: &str, logger: &mut FileLogger) {
        self.last_files.retain(|value| value != path);
        self.last_files.insert(0, path.to_string());
        self.truncate_recent_files();
        self.save(logger);
    }

    pub fn ensure_last_file(&mut self, path: &str, logger: &mut FileLogger) {
        if self.last_files.iter().any(|value| value == path) {
            return;
        }

        self.last_files.push(path.to_string());
        self.truncate_recent_files();
        self.save(logger);
    }

    pub fn remove_last_file(&mut self, path: &str, logger: &mut FileLogger) {
        let original_len = self.last_files.len();
        self.last_files.retain(|value| value != path);
        if self.last_files.len() != original_len {
            self.save(logger);
        }
    }

    fn truncate_recent_files(&mut self) {
        const MAX_RECENT_FILES: usize = 24;
        if self.last_files.len() > MAX_RECENT_FILES {
            self.last_files.truncate(MAX_RECENT_FILES);
        }
    }

    pub fn save(&self, logger: &mut FileLogger) {
        let file = FileSystem::create_file(FileSystem::get_config_file_path());
        if let Err(error) = serde_json::to_writer_pretty(BufWriter::new(file), self) {
            logger.log(format!("Failed to save config: {}", error));
        }
    }

    pub fn load_config() -> Config {
        let path = FileSystem::get_config_file_path();

        if path.exists() {
            if let Ok(file) = File::open(&path) {
                if let Ok(cfg) = serde_json::from_reader::<_, Config>(file) {
                    return cfg;
                }
            }
        }

        Config::default()
    }
}
