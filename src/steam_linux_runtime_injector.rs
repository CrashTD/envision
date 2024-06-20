use crate::{
    file_utils::{copy_file, get_writer},
    paths::{get_backup_dir, get_home_dir, get_xdg_data_dir},
    profile::Profile,
};
use anyhow::bail;
use std::{fs::read_to_string, io::Write, path::Path};

fn get_runtime_entrypoint_path() -> Option<String> {
    return None;
    let mut out = format!(
        "{home}/.steam/steam/steamapps/common/SteamLinuxRuntime_sniper/_v2-entry-point",
        home = get_home_dir(),
    );

    if Path::new(&out).is_file() {
        return Some(out);
    }

    out = format!(
        "{data}/Steam/ubuntu12_64/steam-runtime-sniper/_v2-entry-point",
        data = get_xdg_data_dir()
    );

    if Path::new(&out).is_file() {
        return Some(out);
    }

    None
}

fn get_backup_runtime_entrypoint_location() -> String {
    format!("{backup}/_v2-entry-point.bak", backup = get_backup_dir())
}

fn backup_runtime_entrypoint(path: &str) {
    copy_file(&path, &get_backup_runtime_entrypoint_location());
}

pub fn restore_runtime_entrypoint() {
    if let Some(path) = get_runtime_entrypoint_path() {
        let backup = get_backup_runtime_entrypoint_location();
        if Path::new(&backup).is_file() {
            copy_file(&backup, &path);
        }
    }
}

fn append_to_runtime_entrypoint(data: &str, path: &str) -> anyhow::Result<()> {
    let existing = read_to_string(path)?;
    let new = existing.replace(
        "exec \"${here}/${run}\" \"$@\"\nexit 125",
        &format!("\n\n# envision\n{data}\n\nexec \"${{here}}/${{run}}\" \"$@\"\nexit 125"),
    );
    let mut writer = get_writer(path)?;
    writer.write(&new.as_bytes())?;
    Ok(())
}

pub fn set_runtime_entrypoint_launch_opts_from_profile(profile: &Profile) -> anyhow::Result<()> {
    restore_runtime_entrypoint();
    if let Some(dest) = get_runtime_entrypoint_path() {
        backup_runtime_entrypoint(&dest);
        append_to_runtime_entrypoint(
            &profile
                .get_env_vars()
                .iter()
                .map(|ev| "export ".to_string() + ev)
                .collect::<Vec<String>>()
                .join("\n"),
            &dest,
        )?;

        return Ok(());
    }
    bail!("Could not find valid runtime entrypoint");
}
