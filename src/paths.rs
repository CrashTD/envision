use crate::file_utils::get_data_dir;

pub fn data_opencomposite_path() -> String {
    format!("{data}/opencomposite", data = get_data_dir())
}

pub fn data_monado_path() -> String {
    format!("{data}/monado", data = get_data_dir())
}

pub fn data_libsurvive_path() -> String {
    format!("{data}/libsurvive", data = get_data_dir())
}

pub const SYSTEM_PREFIX: &str = "/usr";

/** System prefix inside a bubblewrap environment (flatpak or pressure vessel) */
pub const BWRAP_SYSTEM_PREFIX: &str = "/run/host/usr";
