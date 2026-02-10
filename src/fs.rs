use std::env;
use std::fs::{File, OpenOptions};
use std::path::PathBuf;

#[derive(Default)]
pub struct FileSystem{}

impl FileSystem {
    pub fn new() -> Self {
        match std::fs::create_dir(Self::get_notepad_dir()) {
            Ok(_) => {}
            Err(_) => {}
        };


        Self::default()
    }

    fn base_app_dir() -> PathBuf {
        // Windows: %APPDATA%
        #[cfg(target_os = "windows")]
        {
            if let Some(p) = env::var_os("APPDATA") {
                return PathBuf::from(p);
            }
        }

        // Linux: $XDG_CONFIG_HOME или ~/.config
        #[cfg(target_os = "linux")]
        {
            if let Some(p) = env::var_os("XDG_CONFIG_HOME") {
                return PathBuf::from(p);
            }
            if let Some(home) = env::var_os("HOME") {
                return PathBuf::from(home).join(".config");
            }
        }

        // macOS: ~/Library/Application Support
        #[cfg(target_os = "macos")]
        {
            if let Some(home) = env::var_os("HOME") {
                return PathBuf::from(home)
                    .join("Library")
                    .join("Application Support");
            }
        }

        // ???? wtf
        env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    }

    pub fn get_notepad_dir() -> PathBuf {
        Self::base_app_dir().join("notepad")
    }

    pub fn get_log_file_path() -> PathBuf {
        Self::get_notepad_dir().join("log.txt")
    }
    pub fn get_config_file_path() -> PathBuf {
        Self::get_notepad_dir().join(".config")
    }


    pub fn create_file(path: PathBuf) -> File {
        match File::create(&path) {
            Ok(file) => {
                return file;
            }
            Err(_) => {
                panic!("Failed to create {} file!", &path.display());
            }
        }
    }

    pub fn open_file_to_write(path: PathBuf, clear: bool) -> File {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).expect("Failed to create log dir");
        }

        let mut options = OpenOptions::new();
        options.create(true);

        if clear {
            options.write(true).truncate(true); // <-- обязательно write(true)
        } else {
            options.append(true);               // append сам включает запись
        }

        match options.open(&path) {
            Ok(file) => file,
            Err(e) => panic!("Failed to open {} file: {}", path.display(), e),
        }
    }

    pub fn create_dir(path: PathBuf) {
        if (path.exists()) {
            return;
        }

        std::fs::create_dir(&path).unwrap_or_else(|_| panic!("Failed to create {} directory!", &path.display()));
    }
}