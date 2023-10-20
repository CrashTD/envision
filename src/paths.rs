use crate::{constants::CMD_NAME, profile::XRServiceType};
use std::{env, fs::create_dir_all, path::Path};

pub fn data_opencomposite_path() -> String {
    format!("{data}/opencomposite", data = get_data_dir())
}

pub fn data_monado_path() -> String {
    format!("{data}/monado", data = get_data_dir())
}

pub fn data_wivrn_path() -> String {
    format!("{data}/wivrn", data = get_data_dir())
}

pub fn data_libsurvive_path() -> String {
    format!("{data}/libsurvive", data = get_data_dir())
}

pub fn data_openhmd_path() -> String {
    format!("{data}/openhmd", data = get_data_dir())
}

pub fn data_basalt_path() -> String {
    format!("{data}/basalt", data = get_data_dir())
}

pub fn wivrn_apk_download_path() -> String {
    format!("{cache}/wivrn.apk", cache = get_cache_dir())
}

pub const SYSTEM_PREFIX: &str = "/usr";

/** System prefix inside a bubblewrap environment (flatpak or pressure vessel) */
pub const BWRAP_SYSTEM_PREFIX: &str = "/run/host/usr";

pub fn get_home_dir() -> String {
    env::var("HOME").expect("HOME env var not defined")
}

pub fn get_xdg_config_dir() -> String {
    match env::var("XDG_CONFIG_HOME") {
        Ok(conf_home) => conf_home,
        Err(_) => format!("{home}/.config", home = get_home_dir()),
    }
}

pub fn get_xdg_data_dir() -> String {
    match env::var("XDG_DATA_HOME") {
        Ok(data_home) => data_home,
        Err(_) => format!("{home}/.local/share", home = get_home_dir()),
    }
}

pub fn get_xdg_cache_dir() -> String {
    match env::var("XDG_CACHE_HOME") {
        Ok(cache_home) => cache_home,
        Err(_) => format!("{home}/.cache", home = get_home_dir()),
    }
}

pub fn get_xdg_runtime_dir() -> String {
    env::var("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR is not set")
}

pub fn get_config_dir() -> String {
    format!(
        "{config}/{name}",
        config = get_xdg_config_dir(),
        name = CMD_NAME
    )
}

pub fn get_data_dir() -> String {
    format!("{data}/{name}", data = get_xdg_data_dir(), name = CMD_NAME)
}

pub fn get_cache_dir() -> String {
    format!(
        "{cache}/{name}",
        cache = get_xdg_cache_dir(),
        name = CMD_NAME
    )
}

pub fn get_backup_dir() -> String {
    let p_s = format!("{data}/backups", data = get_data_dir());
    let p = Path::new(&p_s);
    if !p.is_dir() {
        if p.is_file() {
            panic!(
                "{} is a file but it should be a directory!",
                p.to_str().unwrap()
            );
        }
        create_dir_all(p).expect("Failed to create backups dir");
    }
    p.to_str().unwrap().to_string()
}

pub fn get_exec_prefix() -> String {
    let p = Path::new("/proc/self/exe");
    if !p.is_symlink() {
        panic!("/proc/self/exe is not a symlink!");
    }
    p.read_link()
        .unwrap()
        .as_path()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

pub fn get_ipc_file_path(xrservice_type: &XRServiceType) -> String {
    format!(
        "{runtime}/{xrservice}_comp_ipc",
        runtime = get_xdg_runtime_dir(),
        xrservice = match xrservice_type {
            XRServiceType::Monado => "monado",
            XRServiceType::Wivrn => "wivrn",
        }
    )
}
