use std::env;
use std::fs::{File, OpenOptions};
use std::io;
use std::path::PathBuf;

#[derive(Default)]
pub struct FileSystem {}

impl FileSystem {
    pub fn new() -> Self {
        let _ = std::fs::create_dir(Self::get_notepad_dir());
        let _ = std::fs::create_dir_all(Self::get_documents_dir());
        let _ = std::fs::create_dir_all(Self::get_recovery_dir());
        Self::default()
    }

    fn base_app_dir() -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            if let Some(path) = env::var_os("APPDATA") {
                return PathBuf::from(path);
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Some(path) = env::var_os("XDG_CONFIG_HOME") {
                return PathBuf::from(path);
            }
            if let Some(home) = env::var_os("HOME") {
                return PathBuf::from(home).join(".config");
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Some(home) = env::var_os("HOME") {
                return PathBuf::from(home)
                    .join("Library")
                    .join("Application Support");
            }
        }

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

    pub fn get_documents_dir() -> PathBuf {
        Self::get_notepad_dir().join("documents")
    }

    pub fn get_recovery_dir() -> PathBuf {
        Self::get_notepad_dir().join("recovery")
    }

    pub fn get_recovery_file_path() -> PathBuf {
        Self::get_recovery_dir().join("session.json")
    }

    pub fn ensure_documents_dir() -> PathBuf {
        let dir = Self::get_documents_dir();
        std::fs::create_dir_all(&dir)
            .unwrap_or_else(|_| panic!("Failed to create {} directory!", dir.display()));
        dir
    }

    pub fn next_new_document_path(current_path: Option<&PathBuf>) -> PathBuf {
        let dir = Self::ensure_documents_dir();
        let base_name = "New Document";

        for index in 0..=9999 {
            let file_name = if index == 0 {
                format!("{}.txt", base_name)
            } else {
                format!("{} {}.txt", base_name, index)
            };

            let candidate = dir.join(file_name);
            if candidate.exists() {
                continue;
            }

            if current_path.is_some_and(|path| path == &candidate) {
                continue;
            }

            return candidate;
        }

        dir.join("New Document Overflow.txt")
    }

    pub fn create_file(path: PathBuf) -> File {
        match File::create(&path) {
            Ok(file) => file,
            Err(_) => panic!("Failed to create {} file!", path.display()),
        }
    }

    pub fn open_file_to_write(path: PathBuf, clear: bool) -> File {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).expect("Failed to create log dir");
        }

        let mut options = OpenOptions::new();
        options.create(true);

        if clear {
            options.write(true).truncate(true);
        } else {
            options.append(true);
        }

        match options.open(&path) {
            Ok(file) => file,
            Err(error) => panic!("Failed to open {} file: {}", path.display(), error),
        }
    }

    pub fn create_dir(path: PathBuf) {
        if path.exists() {
            return;
        }

        std::fs::create_dir(&path)
            .unwrap_or_else(|_| panic!("Failed to create {} directory!", path.display()));
    }

    pub fn read_text_file(path: &PathBuf) -> io::Result<String> {
        std::fs::read_to_string(path)
    }

    pub fn write_text_file(path: &PathBuf, text: &str) -> io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }

        std::fs::write(path, text)
    }
}
