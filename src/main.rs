use crate::app::App;

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
    let mut app = App::new();
    app.run();
}
