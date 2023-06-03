use iced::{Application, Settings};
use ui::main_win::MainWin;

pub mod profile;
pub mod runner;
pub mod ui;
pub mod config;
pub mod constants;
pub mod file_utils;
pub mod profiles;
pub mod depcheck;
pub mod dependencies;

fn main() {
    MainWin::run(Settings::default());
}
