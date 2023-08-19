use anyhow::Result;
use constants::{resources, APP_ID, APP_NAME, GETTEXT_PACKAGE, LOCALE_DIR};
use file_builders::{
    active_runtime_json::{get_current_active_runtime, set_current_active_runtime_to_steam},
    openvrpaths_vrpath::{get_current_openvrpaths, set_current_openvrpaths_to_steam},
};
use gettextrs::LocaleCategory;
use relm4::{
    adw,
    gtk::{self, gdk, gio, glib},
    RelmApp,
};
use ui::app::{App, AppInit};

pub mod adb;
pub mod build_tools;
pub mod builders;
pub mod checkerr;
pub mod cmd_runner;
pub mod config;
pub mod constants;
pub mod depcheck;
pub mod dependencies;
pub mod device_prober;
pub mod downloader;
pub mod env_var_descriptions;
pub mod file_builders;
pub mod file_utils;
pub mod func_runner;
pub mod log_level;
pub mod log_parser;
pub mod paths;
pub mod profile;
pub mod profiles;
pub mod runner;
pub mod runner_pipeline;
pub mod steamvr_utils;
pub mod ui;
pub mod xr_devices;

fn restore_steam_xr_files() {
    let active_runtime = get_current_active_runtime();
    let openvrpaths = get_current_openvrpaths();
    if let Some(ar) = active_runtime {
        if !file_builders::active_runtime_json::is_steam(&ar) {
            match set_current_active_runtime_to_steam() {
                Ok(_) => {}
                Err(_) => println!("Warning: failed to restore active runtime to steam!"),
            };
        }
    }
    if let Some(ovrp) = openvrpaths {
        if !file_builders::openvrpaths_vrpath::is_steam(&ovrp) {
            match set_current_openvrpaths_to_steam() {
                Ok(_) => {}
                Err(_) => println!("Warning: failed to restore openvrpaths to steam!"),
            };
        }
    }
}

fn main() -> Result<()> {
    restore_steam_xr_files();

    // Prepare i18n
    gettextrs::setlocale(LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain(GETTEXT_PACKAGE, LOCALE_DIR).expect("Unable to bind the text domain");
    gettextrs::textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    gtk::init()?;
    glib::set_application_name(APP_NAME);

    {
        let res = gio::Resource::load(resources()).expect("Could not load gresource file");
        gio::resources_register(&res);
    }

    let provider = gtk::CssProvider::new();
    provider.load_from_resource("/org/gabmus/rex2/style.css");
    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
    gtk::Window::set_default_icon_name(APP_ID);

    let main_app = adw::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::empty())
        .resource_base_path("/org/gabmus/rex2")
        .build();
    let app = RelmApp::from_app(main_app.clone());
    app.run::<App>(AppInit {
        application: main_app,
    });
    Ok(())
}
