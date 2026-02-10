use crate::app::App;

mod fs;
mod config;
mod logger;
mod app;
mod screen_buf;
mod terminal;
mod input;
mod ui;
mod controls;

fn main(){
    let mut app = App::new();
    app.run();
}
