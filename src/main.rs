use iced::{Application, Settings};
use ui::main_win::MainWin;

pub mod builders;
pub mod config;
pub mod constants;
pub mod depcheck;
pub mod dependencies;
pub mod file_utils;
pub mod profile;
pub mod profiles;
pub mod runner;
pub mod ui;
pub mod file_builders;

fn main() {
    MainWin::run(Settings::default()).expect("Application encountered an error");
}
