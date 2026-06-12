use lazy_static::lazy_static;
use std::path::PathBuf;
use xdg::BaseDirectories;

lazy_static! {
    pub static ref XDG: BaseDirectories = BaseDirectories::new();
    pub static ref XDG_DATA_HOME: PathBuf =
        XDG.get_data_home().expect("Failed to get XDG_DATA_HOME");
    pub static ref XDG_CONFIG_HOME: PathBuf = XDG
        .get_config_home()
        .expect("Failed to get XDG_CONFIG_HOME");
    pub static ref XDG_CACHE_HOME: PathBuf =
        XDG.get_cache_home().expect("Failed to get XDG_CACHE_HOME");
}
