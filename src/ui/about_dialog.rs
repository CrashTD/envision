use crate::constants::{get_developers, APP_ID, APP_NAME, REPO_URL, SINGLE_DEVELOPER, VERSION};
use relm4::prelude::*;

pub fn create_about_dialog() -> adw::AboutDialog {
    adw::AboutDialog::builder()
        .application_name(APP_NAME)
        .application_icon(APP_ID)
        .license_type(gtk::License::Agpl30)
        .version(VERSION)
        .website(REPO_URL)
        .developer_name(SINGLE_DEVELOPER)
        .developers(get_developers())
        .build()
}
