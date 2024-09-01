use crate::{constants::CMD_NAME, xdg::XDG};
use std::{
    env,
    fs::create_dir_all,
    path::{Path, PathBuf},
};

pub fn data_opencomposite_path() -> PathBuf {
    get_data_dir().join("opencomposite")
}

pub fn data_monado_path() -> PathBuf {
    get_data_dir().join("monado")
}

pub fn data_wivrn_path() -> PathBuf {
    get_data_dir().join("wivrn")
}

pub fn data_libsurvive_path() -> PathBuf {
    get_data_dir().join("libsurvive")
}

pub fn data_openhmd_path() -> PathBuf {
    get_data_dir().join("openhmd")
}

pub fn data_basalt_path() -> PathBuf {
    get_data_dir().join("basalt")
}

pub fn wivrn_apk_download_path() -> PathBuf {
    get_cache_dir().join("wivrn.apk")
}

pub const SYSTEM_PREFIX: &str = "/usr";

/// System prefix inside a bubblewrap environment (flatpak or pressure vessel)
pub const BWRAP_SYSTEM_PREFIX: &str = "/run/host/usr";

pub fn get_home_dir() -> PathBuf {
    env::var("HOME").expect("HOME env var not defined").into()
}

pub fn get_config_dir() -> PathBuf {
    XDG.get_config_home().join(CMD_NAME)
}

pub fn get_data_dir() -> PathBuf {
    XDG.get_data_home().join(CMD_NAME)
}

pub fn get_cache_dir() -> PathBuf {
    XDG.get_cache_home().join(CMD_NAME)
}

pub fn get_backup_dir() -> PathBuf {
    let p = get_data_dir().join("backups");
    if !p.is_dir() {
        if p.is_file() {
            panic!(
                "{} is a file but it should be a directory!",
                p.to_str().unwrap()
            );
        }
        create_dir_all(&p).expect("Failed to create backups dir");
    }
    p
}

pub fn get_exec_prefix() -> PathBuf {
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
        .into()
}

pub fn get_steamvr_bin_dir_path() -> PathBuf {
    XDG.get_data_home()
        .join("Steam/steamapps/common/SteamVR/bin/linux64")
}
