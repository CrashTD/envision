use crate::{
    constants::{
        get_artists, APP_ID, APP_NAME, BUILD_DATETIME, ISSUES_URL, REPO_URL, SINGLE_DEVELOPER,
        VERSION,
    },
    device_prober::PhysicalXRDevice,
    linux_distro::LinuxDistro,
    vulkaninfo::VulkanInfo,
    xdg::XDG,
};
use relm4::prelude::*;
use std::{env, fs::read_to_string};

pub fn create_about_dialog() -> adw::AboutDialog {
    adw::AboutDialog::builder()
        .application_name(APP_NAME)
        .application_icon(APP_ID)
        .license_type(gtk::License::Agpl30)
        .version(VERSION)
        .website(REPO_URL)
        .issue_url(ISSUES_URL)
        .developer_name(SINGLE_DEVELOPER)
        .developers(
            env!("CARGO_PKG_AUTHORS")
                .split(':')
                .map(|s| s.to_string())
                .collect::<Vec<String>>(),
        )
        .artists(get_artists())
        .build()
}

const UNKNOWN: &str = "UNKNOWN";

pub fn populate_debug_info(dialog: &adw::AboutDialog, vkinfo: Option<&VulkanInfo>) {
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
                d = distro.unwrap_or(UNKNOWN.into()),
                f = distro_family
                    .map(|f| f.to_string())
                    .unwrap_or(UNKNOWN.into())
            ),
            format!(
                "Kernel: {}",
                read_to_string("/proc/version")
                    .unwrap_or("Unable to read /proc/version".into())
                    .trim()
            ),
            format!(
                "Session type: {}",
                env::var("XDG_SESSION_TYPE").unwrap_or(UNKNOWN.into())
            ),
            format!(
                "Desktop: {}",
                env::var("XDG_CURRENT_DESKTOP").unwrap_or(UNKNOWN.into())
            ),
            format!(
                "CPU: {}",
                read_to_string("/proc/cpuinfo")
                    .ok()
                    .and_then(|s| {
                        s.split("\n")
                            .find(|line| line.starts_with("model name"))
                            .map(|line| line.split(':').last().map(|s| s.trim().to_string()))
                    })
                    .flatten()
                    .unwrap_or(UNKNOWN.into())
            ),
            format!(
                "GPUs: {}",
                vkinfo
                    .map(|i| i.gpu_names.join(", "))
                    .unwrap_or(UNKNOWN.into())
            ),
            format!("Detected XR Devices: {}", {
                let devs = PhysicalXRDevice::from_usb();
                if devs.is_empty() {
                    "None".into()
                } else {
                    devs.iter()
                        .map(PhysicalXRDevice::to_string)
                        .collect::<Vec<String>>()
                        .join(", ")
                }
            }),
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
        .map(|s| format!("- {s}"))
        .join("\n"),
    );
}
