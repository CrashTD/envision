use crate::paths::get_home_dir;

pub fn chaperone_info_exists() -> bool {
    get_home_dir()
        .join(".steam/steam/config/chaperone_info.vrchap")
        .is_file()
}
