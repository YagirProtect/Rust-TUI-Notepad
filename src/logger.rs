use std::io::{LineWriter, Write};
use crate::fs::FileSystem;

pub struct FileLogger {
    file: LineWriter<std::fs::File>,
}

impl FileLogger {
    pub fn new() -> Self {
        FileSystem::create_dir(FileSystem::get_notepad_dir());
        let file = FileSystem::open_file_to_write(FileSystem::get_log_file_path(), true);
        Self { file: LineWriter::new(file) }
    }

    pub fn log<M: AsRef<str>>(&mut self, message: M) {
        writeln!(self.file, "Log: {}", message.as_ref()).unwrap();
    }

    pub fn log_err<M: AsRef<str>>(&mut self, message: M) {
        writeln!(self.file, "Error: {}", message.as_ref()).unwrap();
    }
}
