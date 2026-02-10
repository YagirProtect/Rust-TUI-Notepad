use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::fs::FileSystem;
use crate::logger::FileLogger;

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    last_files: Vec<PathBuf>,
    width: u16,
    height: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self{
            last_files: vec![],
            width: 400,
            height: 200,
        }
    }
}

impl Config {
    pub fn new(logger: &mut FileLogger) -> Self {
        if (FileSystem::get_config_file_path().exists()) {
            return Self::load_config();
        }
        let default = Config::default();
        let file = FileSystem::create_file(FileSystem::get_config_file_path());
        match serde_json::to_writer(BufWriter::new(file), &default){
            Ok(_) => {
                return default;
            }
            Err(e) => {
                logger.log(e.to_string());
            }
        }
        return default;
    }


    pub fn get_win_size(&self) -> (u16, u16) {
        (self.width, self.height)
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