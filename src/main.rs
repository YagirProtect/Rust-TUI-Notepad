use crate::app::App;
use std::path::PathBuf;

mod fs;
mod config;
mod logger;
mod app;
mod app_actions;
mod app_dialogs;
mod screen_buffer;
mod terminal;
mod input;
mod ui;
mod controls;
mod characters;
mod e_actions;
mod panels;
mod shortcuts;
mod syntax_highlight;
mod text_buffer;

fn main(){
    let start_path = std::env::args_os().nth(1).map(PathBuf::from);
    let mut app = App::new(start_path);
    let _ = app.run();
}
