use crate::{paths::get_home_dir, xdg::XDG};
use anyhow::bail;
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs::read_to_string,
    path::{Path, PathBuf},
};

#[derive(Deserialize)]
pub struct SteamLibraryFolder {
    pub path: String,
    pub apps: HashMap<u32, usize>,
}

pub fn get_all_potential_steam_main_dir_paths() -> Vec<PathBuf> {
    vec![
        get_home_dir().join(".steam/root"),
        XDG.get_data_home().join("Steam"),
        XDG.get_data_home().join("steam"),
    ]
}

pub fn chaperone_info_exists() -> bool {
    get_all_potential_steam_main_dir_paths().iter().any(|p| {
        p.join("config/chaperone_info.vrchap")
            .canonicalize()
            .is_ok_and(|p| p.is_file())
    })
}

impl SteamLibraryFolder {
    pub fn get_folders() -> anyhow::Result<HashMap<u32, Self>> {
        if let Some(libraryfolders_path) = get_all_potential_steam_main_dir_paths()
            .into_iter()
            .filter_map(|p| p.join("steamapps/libraryfolders.vdf").canonicalize().ok())
            .find(|p| p.is_file())
        {
            Self::get_folders_from_path(&libraryfolders_path)
        } else {
            bail!("Steam libraryfolders.vdf does not exist in any of its canonical locations");
        }
    }

    /// Do not use this: use get_folders() instead as it always uses Steam's
    /// canonical root path. This is intended to be directly used only for
    /// unit tests
    pub fn get_folders_from_path(p: &Path) -> anyhow::Result<HashMap<u32, Self>> {
        Ok(keyvalues_serde::from_str(read_to_string(p)?.as_str())?)
    }
}

#[cfg(test)]
mod tests {
    use super::SteamLibraryFolder;
    use std::path::Path;

    #[test]
    fn deserialize_steam_libraryfolders_vdf() {
        let lf = SteamLibraryFolder::get_folders_from_path(Path::new(
            "./test/files/steam_libraryfolders.vdf",
        ))
        .unwrap();
        assert_eq!(lf.len(), 1);
        let first = lf.get(&0).unwrap();
        assert_eq!(first.path, "/home/gabmus/.local/share/Steam");
        assert_eq!(first.apps.len(), 10);
        assert_eq!(first.apps.get(&228980).unwrap(), &29212173);
        assert_eq!(first.apps.get(&632360).unwrap(), &0);
    }
}
