use crate::paths::get_home_dir;
use std::path::Path;

pub fn chaperone_info_exists() -> bool {
    let path_s = format!(
        "{}/.steam/steam/config/chaperone_info.vrchap",
        get_home_dir()
    );
    let path = Path::new(&path_s);
    path.is_file()
}
