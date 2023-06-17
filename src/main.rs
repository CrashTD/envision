use constants::APP_ID;
use relm4::RelmApp;
use ui::app::App;

pub mod builders;
pub mod config;
pub mod constants;
pub mod depcheck;
pub mod dependencies;
pub mod file_builders;
pub mod file_utils;
pub mod profile;
pub mod profiles;
pub mod runner;
pub mod ui;
pub mod runner_pipeline;
pub mod paths;

fn main() {
    let app = RelmApp::new(APP_ID);
    relm4_icons::initialize_icons();
    app.run::<App>(())
}
