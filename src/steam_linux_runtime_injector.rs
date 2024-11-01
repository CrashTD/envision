use crate::{
    paths::{get_backup_dir, get_home_dir},
    profile::Profile,
    util::file_utils::{copy_file, get_writer},
};
use anyhow::bail;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs::read_to_string,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Deserialize)]
struct LibraryFolder {
    pub path: String,
    pub apps: HashMap<u32, usize>,
}

pub const PRESSURE_VESSEL_STEAM_APPID: u32 = 1628350;

fn get_steam_main_dir_path() -> anyhow::Result<PathBuf> {
    let steam_root: PathBuf = get_home_dir().join(".steam/root");

    if steam_root.is_symlink() {
        Ok(steam_root.read_link()?)
    } else if steam_root.is_dir() {
        Ok(steam_root)
    } else {
        bail!(
            "Canonical steam root '{}' is not a dir nor a symlink!",
            steam_root.to_string_lossy()
        )
    }
}

fn parse_steam_libraryfolders_vdf(path: &Path) -> anyhow::Result<HashMap<u32, LibraryFolder>> {
    Ok(keyvalues_serde::from_str(read_to_string(path)?.as_str())?)
}

fn get_runtime_entrypoint_path() -> Option<PathBuf> {
    match get_steam_main_dir_path() {
        Ok(steam_root) => {
            let steam_libraryfolders_path = steam_root.join("steamapps/libraryfolders.vdf");

            if !steam_libraryfolders_path.is_file() {
                eprintln!(
                    "Steam libraryfolders.vdf does not exist in its canonical location {}",
                    steam_libraryfolders_path.to_string_lossy()
                );
                return None;
            }

            let libraryfolders: HashMap<u32, LibraryFolder> =
                parse_steam_libraryfolders_vdf(&steam_libraryfolders_path).ok()?;

            libraryfolders
                .iter()
                .find(|(_, libraryfolder)| {
                    libraryfolder
                        .apps
                        .contains_key(&PRESSURE_VESSEL_STEAM_APPID)
                })
                .map(|(_, libraryfolder)| PathBuf::from(&libraryfolder.path))
        }
        Err(e) => {
            eprintln!("Error getting steam root path: {e}");
            None
        }
    }
}

lazy_static! {
    static ref STEAM_RUNTIME_ENTRYPOINT_PATH: Option<PathBuf> = get_runtime_entrypoint_path();
}

fn get_backup_runtime_entrypoint_location() -> PathBuf {
    get_backup_dir().join("_v2-entry-point.bak")
}

fn backup_runtime_entrypoint(path: &Path) {
    copy_file(path, &get_backup_runtime_entrypoint_location());
}

pub fn restore_runtime_entrypoint() {
    if let Some(path) = STEAM_RUNTIME_ENTRYPOINT_PATH.as_ref() {
        let backup = get_backup_runtime_entrypoint_location();
        if Path::new(&backup).is_file() {
            copy_file(&backup, path);
        }
    }
}

fn append_to_runtime_entrypoint(data: &str, path: &Path) -> anyhow::Result<()> {
    let existing = read_to_string(path)?;
    let new = existing.replace(
        "exec \"${here}/${run}\" \"$@\"\nexit 125",
        &format!("\n\n# envision\n{data}\n\nexec \"${{here}}/${{run}}\" \"$@\"\nexit 125"),
    );
    let mut writer = get_writer(path)?;
    writer.write_all(new.as_bytes())?;
    Ok(())
}

pub fn set_runtime_entrypoint_launch_opts_from_profile(profile: &Profile) -> anyhow::Result<()> {
    restore_runtime_entrypoint();
    if let Some(dest) = STEAM_RUNTIME_ENTRYPOINT_PATH.as_ref() {
        backup_runtime_entrypoint(dest);
        append_to_runtime_entrypoint(
            &profile
                .get_env_vars()
                .iter()
                .map(|ev| "export ".to_string() + ev)
                .collect::<Vec<String>>()
                .join("\n"),
            dest,
        )?;

        return Ok(());
    }
    bail!("Could not find valid runtime entrypoint");
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::parse_steam_libraryfolders_vdf;

    #[test]
    fn deserialize_steam_libraryfolders_vdf() {
        let lf = parse_steam_libraryfolders_vdf(Path::new("./test/files/steam_libraryfolders.vdf"))
            .unwrap();
        assert_eq!(lf.len(), 1);
        let first = lf.get(&0).unwrap();
        assert_eq!(first.path, "/home/gabmus/.local/share/Steam");
        assert_eq!(first.apps.len(), 10);
        assert_eq!(first.apps.get(&228980).unwrap(), &29212173);
        assert_eq!(first.apps.get(&632360).unwrap(), &0);
    }
}
