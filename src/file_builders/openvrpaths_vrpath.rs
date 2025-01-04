use std::path::{Path, PathBuf};

use crate::{
    paths::get_backup_dir,
    profile::{OvrCompatibilityModuleType, Profile},
    util::file_utils::{copy_file, deserialize_file, get_writer, set_file_readonly},
    xdg::XDG,
};
use serde::{ser::Error, Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenVrPaths {
    config: Vec<PathBuf>,
    external_drivers: Option<Vec<String>>, // never seen it populated
    jsonid: String,
    log: Vec<PathBuf>,
    runtime: Vec<PathBuf>,
    version: u32,
}

pub fn get_openvr_conf_dir() -> PathBuf {
    XDG.get_config_home().join("openvr")
}

fn get_openvrpaths_vrpath_path() -> PathBuf {
    get_openvr_conf_dir().join("openvrpaths.vrpath")
}

pub fn is_steam(ovr_paths: &OpenVrPaths) -> bool {
    ovr_paths.runtime.iter().any(|rt| {
        rt.to_string_lossy()
            .to_lowercase()
            .ends_with("/steam/steamapps/common/steamvr")
    })
}

fn get_backup_steam_openvrpaths_path() -> PathBuf {
    get_backup_dir().join("openvrpaths.vrpath.steam.bak")
}

fn get_backed_up_steam_openvrpaths() -> Option<OpenVrPaths> {
    get_openvrpaths_from_path(&get_backup_steam_openvrpaths_path())
}

fn backup_steam_openvrpaths() {
    if let Some(openvrpaths) = get_current_openvrpaths() {
        if is_steam(&openvrpaths) {
            copy_file(
                &get_openvrpaths_vrpath_path(),
                &get_backup_steam_openvrpaths_path(),
            );
        }
    }
}

fn get_openvrpaths_from_path(path: &Path) -> Option<OpenVrPaths> {
    deserialize_file(path)
}

pub fn get_current_openvrpaths() -> Option<OpenVrPaths> {
    get_openvrpaths_from_path(&get_openvrpaths_vrpath_path())
}

fn dump_openvrpaths_to_path(ovr_paths: &OpenVrPaths, path: &Path) -> Result<(), serde_json::Error> {
    let writer = get_writer(path).map_err(serde_json::Error::custom)?;
    serde_json::to_writer_pretty(writer, ovr_paths)
}

pub fn dump_current_openvrpaths(ovr_paths: &OpenVrPaths) -> Result<(), serde_json::Error> {
    dump_openvrpaths_to_path(ovr_paths, &get_openvrpaths_vrpath_path())
}

fn build_steam_openvrpaths() -> OpenVrPaths {
    if let Some(backup) = get_backed_up_steam_openvrpaths() {
        return backup;
    }
    let datadir = XDG.get_data_home();
    OpenVrPaths {
        config: vec![datadir.join("Steam/config")],
        external_drivers: None,
        jsonid: "vrpathreg".into(),
        log: vec![datadir.join("Steam/logs")],
        runtime: vec![datadir.join("Steam/steamapps/common/SteamVR")],
        version: 1,
    }
}

pub fn set_current_openvrpaths_to_steam() -> anyhow::Result<()> {
    set_file_readonly(&get_openvrpaths_vrpath_path(), false)?;
    dump_current_openvrpaths(&build_steam_openvrpaths())?;
    Ok(())
}

pub fn build_profile_openvrpaths(profile: &Profile) -> OpenVrPaths {
    let datadir = XDG.get_data_home();
    OpenVrPaths {
        config: vec![datadir.join("Steam/config")],
        external_drivers: None,
        jsonid: "vrpathreg".into(),
        log: vec![datadir.join("Steam/logs")],
        runtime: match profile.ovr_comp.mod_type {
            OvrCompatibilityModuleType::Opencomposite => vec![profile.ovr_comp.path.join("build")],
            OvrCompatibilityModuleType::Xrizer => {
                vec![profile.ovr_comp.path.join("target/release")]
            }
        },
        version: 1,
    }
}

pub fn set_current_openvrpaths_to_profile(profile: &Profile) -> anyhow::Result<()> {
    let dest = get_openvrpaths_vrpath_path();
    set_file_readonly(&dest, false)?;
    backup_steam_openvrpaths();
    dump_current_openvrpaths(&build_profile_openvrpaths(profile))?;
    set_file_readonly(&dest, true)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{dump_openvrpaths_to_path, get_openvrpaths_from_path, OpenVrPaths};

    #[test]
    fn can_read_openvrpaths_vrpath_steamvr() {
        let ovrp = get_openvrpaths_from_path(Path::new("./test/files/openvrpaths.vrpath")).unwrap();
        assert_eq!(ovrp.config.len(), 1);
        assert_eq!(
            ovrp.config.first().unwrap(),
            &Path::new("/home/user/.local/share/Steam/config")
        );
        assert_eq!(ovrp.external_drivers, None);
        assert_eq!(ovrp.jsonid, "vrpathreg");
        assert_eq!(ovrp.version, 1);
    }

    #[test]
    fn can_dump_openvrpaths_vrpath() {
        let ovrp = OpenVrPaths {
            config: vec!["/home/user/.local/share/Steam/config".into()],
            external_drivers: None,
            jsonid: "vrpathreg".into(),
            log: vec!["/home/user/.local/share/Steam/logs".into()],
            runtime: vec!["/home/user/.local/share/Steam/steamapps/common/SteamVR".into()],
            version: 1,
        };
        dump_openvrpaths_to_path(&ovrp, Path::new("./target/testout/openvrpaths.vrpath"))
            .expect("could not dump openvrpaths to path");
    }
}
