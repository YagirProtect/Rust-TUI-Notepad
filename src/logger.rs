use crate::fs::FileSystem;
use std::fs::File;
use std::io::{BufWriter, Write};

pub struct FileLogger{
    file: BufWriter<File>,
}


impl FileLogger {
    pub fn new() -> Self {

        FileSystem::create_dir(FileSystem::get_notepad_dir());
        if (!FileSystem::get_log_file_path().exists()){
            FileSystem::create_file(FileSystem::get_log_file_path());
        }
        let file = FileSystem::open_file_to_write(FileSystem::get_log_file_path(), true);

        Self{
            file: BufWriter::new(file)
        }
    }
    pub fn log<M: AsRef<str>>(&mut self, message: M) {
        let msg = message.as_ref();
        let s = format!("Log: {}", msg);
        self.file.write_all(s.as_bytes()).unwrap();
        self.file.write_all(b"\n").unwrap();
    }

    pub fn log_err<M: AsRef<str>>(&mut self, message: M) {
        let msg = message.as_ref();
        let s = format!("Error: {}", msg);
        self.file.write_all(s.as_bytes()).unwrap();
        self.file.write_all(b"\n").unwrap();
    }
}