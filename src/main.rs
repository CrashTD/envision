use anyhow::Result;
use constants::{APP_ID, APP_NAME, GETTEXT_PACKAGE, LOCALE_DIR, RESOURCES};
use gettextrs::LocaleCategory;
use relm4::{
    adw,
    gtk::{self, gio, glib, gdk},
    RelmApp,
};
use ui::app::App;

pub mod builders;
pub mod config;
pub mod constants;
pub mod depcheck;
pub mod dependencies;
pub mod file_builders;
pub mod file_utils;
pub mod paths;
pub mod profile;
pub mod profiles;
pub mod runner;
pub mod runner_pipeline;
pub mod ui;

fn main() -> Result<()> {
    // Prepare i18n
    gettextrs::setlocale(LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain(GETTEXT_PACKAGE, LOCALE_DIR).expect("Unable to bind the text domain");
    gettextrs::textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    gtk::init()?;
    glib::set_application_name(APP_NAME);

    {
        let res = gio::Resource::load(RESOURCES).expect("Could not load gresource file");
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

    relm4_icons::initialize_icons();
    let main_app = adw::Application::builder()
        .application_id(APP_ID)
        .flags(gio::ApplicationFlags::empty())
        .resource_base_path("/org/gabmus/rex2")
        .build();
    let app = RelmApp::from_app(main_app);
    app.run::<App>(());
    Ok(())
}
