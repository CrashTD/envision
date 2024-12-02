use crate::{
    paths::get_backup_dir,
    profile::Profile,
    util::{
        file_utils::{copy_file, get_writer},
        steam_library_folder::SteamLibraryFolder,
    },
};
use anyhow::bail;
use lazy_static::lazy_static;
use std::{
    fs::read_to_string,
    io::Write,
    path::{Path, PathBuf},
};
use tracing::error;

pub const PRESSURE_VESSEL_STEAM_APPID: u32 = 1628350;

fn get_runtime_entrypoint_path() -> Option<PathBuf> {
    match SteamLibraryFolder::get_folders() {
        Ok(libraryfolders) => libraryfolders
            .iter()
            .find(|(_, folder)| folder.apps.contains_key(&PRESSURE_VESSEL_STEAM_APPID))
            .map(|(_, folder)| {
                PathBuf::from(&folder.path)
                    .join("steamapps/common/SteamLinuxRuntime_sniper/_v2-entry-point")
            }),
        Err(e) => {
            error!("unable to get runtime entrypoint path: {e}");
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
