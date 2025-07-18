// gracefully stolen from https://github.com/galister/wlx-overlay-s/blob/main/src/backend/openxr/helpers.rs#L9
// original code licensed gpl 3
use openxr as xr;

use crate::constants::CMD_NAME;

pub fn is_openxr_ready() -> bool {
    let entry = xr::Entry::linked();

    if entry.enumerate_extensions().is_err() {
        return false;
    };

    let enabled_extensions = xr::ExtensionSet::default();

    let layers = [];

    let Ok(xr_instance) = entry.create_instance(
        &xr::ApplicationInfo {
            application_name: &format!("{CMD_NAME}-openxr-prober"),
            application_version: 0,
            engine_name: CMD_NAME,
            engine_version: 0,
            api_version: xr::Version::new(1, 0, 0),
        },
        &enabled_extensions,
        &layers,
    ) else {
        return false;
    };

    if xr_instance
        .system(xr::FormFactor::HEAD_MOUNTED_DISPLAY)
        .is_err()
    {
        return false;
    };

    true
}
