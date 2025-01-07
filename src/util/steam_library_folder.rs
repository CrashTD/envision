use crate::paths::get_home_dir;
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

fn get_steam_main_dir_path() -> anyhow::Result<PathBuf> {
    let steam_root: PathBuf = get_home_dir().join(".steam/root").canonicalize()?;

    if steam_root.is_dir() {
        Ok(steam_root)
    } else {
        bail!(
            "Canonical steam root '{}' is not a dir nor a symlink!",
            steam_root.to_string_lossy()
        )
    }
}

impl SteamLibraryFolder {
    pub fn get_folders() -> anyhow::Result<HashMap<u32, Self>> {
        let libraryfolders_path = get_steam_main_dir_path()?
            .join("steamapps/libraryfolders.vdf")
            .canonicalize()?;
        if !libraryfolders_path.is_file() {
            bail!(
                "Steam libraryfolders.vdf does not exist in its canonical location {}",
                libraryfolders_path.to_string_lossy()
            );
        }
        Self::get_folders_from_path(&libraryfolders_path)
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
