use crate::util::steam_library_folder::get_steam_main_dir_path;

pub fn chaperone_info_exists() -> bool {
    get_steam_main_dir_path().is_ok_and(|p| p.join("config/chaperone_info.vrchap").is_file())
}
