use crate::{
    constants::{
        get_developers, APP_ID, APP_NAME, BUILD_DATETIME, ISSUES_URL, REPO_URL, SINGLE_DEVELOPER,
        VERSION,
    },
    linux_distro::LinuxDistro,
    vulkaninfo::gpu_names,
    xdg::XDG,
};
use relm4::prelude::*;
use std::env;

pub fn create_about_dialog() -> adw::AboutDialog {
    adw::AboutDialog::builder()
        .application_name(APP_NAME)
        .application_icon(APP_ID)
        .license_type(gtk::License::Agpl30)
        .version(VERSION)
        .website(REPO_URL)
        .issue_url(ISSUES_URL)
        .developer_name(SINGLE_DEVELOPER)
        .developers(get_developers())
        .build()
}

pub fn populate_debug_info(dialog: &adw::AboutDialog) {
    if dialog.debug_info().len() > 0 {
        return;
    }
    let distro_family = LinuxDistro::get();
    let distro = LinuxDistro::get_specific_distro();
    dialog.set_debug_info(
        &[
            format!("Version: {VERSION}"),
            format!("Build time: {BUILD_DATETIME}"),
            format!(
                "Operating system: {d} ({f})",
                d = distro.unwrap_or("unknown".into()),
                f = distro_family
                    .map(|f| f.to_string())
                    .unwrap_or("unknown".into())
            ),
            format!(
                "Session type: {}",
                env::var("XDG_SESSION_TYPE").unwrap_or("unknown".into())
            ),
            format!(
                "Desktop: {}",
                env::var("XDG_CURRENT_DESKTOP").unwrap_or("unknown".into())
            ),
            format!(
                "GPUs: {}",
                unsafe { gpu_names() }
                    .ok()
                    .map(|names| names.join(", "))
                    .unwrap_or("unknown".into())
            ),
            format!(
                "Steam found: {}",
                if XDG.get_data_home().join("Steam").is_dir()
                    || XDG.get_data_home().join("steam").is_dir()
                {
                    "yes"
                } else {
                    "no"
                }
            ),
        ]
        .join("\n"),
    );
}
