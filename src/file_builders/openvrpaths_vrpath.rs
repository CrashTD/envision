use crate::{
    checkerr,
    file_utils::{copy_file, deserialize_file, get_writer, set_file_readonly},
    paths::{get_backup_dir, get_xdg_config_dir, get_xdg_data_dir},
    profile::Profile,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenVrPaths {
    config: Vec<String>,
    external_drivers: Option<Vec<String>>, // never seen it populated
    jsonid: String,
    log: Vec<String>,
    runtime: Vec<String>,
    version: u32,
}

pub fn get_openvr_conf_dir() -> String {
    format!("{config}/openvr", config = get_xdg_config_dir())
}

fn get_openvrpaths_vrpath_path() -> String {
    format!(
        "{config}/openvrpaths.vrpath",
        config = get_openvr_conf_dir()
    )
}

pub fn is_steam(ovr_paths: &OpenVrPaths) -> bool {
    ovr_paths
        .runtime
        .iter()
        .find(|rt| {
            rt.to_lowercase()
                .ends_with("/steam/steamapps/common/steamvr")
        })
        .is_some()
}

fn get_backup_steam_openvrpaths_path() -> String {
    format!(
        "{backup}/openvrpaths.vrpath.steam.bak",
        backup = get_backup_dir()
    )
}

fn get_backed_up_steam_openvrpaths() -> Option<OpenVrPaths> {
    get_openvrpaths_from_path(&get_backup_steam_openvrpaths_path())
}

fn backup_steam_openvrpaths() {
    let openvrpaths = get_current_openvrpaths();
    if openvrpaths.is_some() {
        if is_steam(&openvrpaths.unwrap()) {
            copy_file(
                &get_openvrpaths_vrpath_path(),
                &get_backup_steam_openvrpaths_path(),
            );
        }
    }
}

fn get_openvrpaths_from_path(path_s: &String) -> Option<OpenVrPaths> {
    deserialize_file(path_s)
}

pub fn get_current_openvrpaths() -> Option<OpenVrPaths> {
    get_openvrpaths_from_path(&get_openvrpaths_vrpath_path())
}

fn dump_openvrpaths_to_path(
    ovr_paths: &OpenVrPaths,
    path_s: &String,
) -> Result<(), serde_json::Error> {
    let writer = get_writer(path_s);
    serde_json::to_writer_pretty(writer, ovr_paths)
}

pub fn dump_current_openvrpaths(ovr_paths: &OpenVrPaths) -> Result<(), serde_json::Error> {
    dump_openvrpaths_to_path(ovr_paths, &get_openvrpaths_vrpath_path())
}

fn build_steam_openvrpaths() -> OpenVrPaths {
    let backup = get_backed_up_steam_openvrpaths();
    if backup.is_some() {
        return backup.unwrap();
    }
    let datadir = get_xdg_data_dir();
    OpenVrPaths {
        config: vec![format!("{data}/Steam/config", data = datadir)],
        external_drivers: None,
        jsonid: "vrpathreg".into(),
        log: vec![format!("{data}/Steam/logs", data = datadir)],
        runtime: vec![format!(
            "{data}/Steam/steamapps/common/SteamVR",
            data = datadir
        )],
        version: 1,
    }
}

pub fn set_current_openvrpaths_to_steam() -> Result<(), ()> {
    checkerr!(set_file_readonly(&get_openvrpaths_vrpath_path(), false));
    checkerr!(dump_current_openvrpaths(&build_steam_openvrpaths()));
    Ok(())
}

pub fn build_profile_openvrpaths(profile: &Profile) -> OpenVrPaths {
    let datadir = get_xdg_data_dir();
    OpenVrPaths {
        config: vec![format!("{data}/Steam/config", data = datadir)],
        external_drivers: None,
        jsonid: "vrpathreg".into(),
        log: vec![format!("{data}/Steam/logs", data = datadir)],
        runtime: vec![format!(
            "{opencomp_dir}/build",
            opencomp_dir = profile.opencomposite_path
        )],
        version: 1,
    }
}

pub fn set_current_openvrpaths_to_profile(profile: &Profile) -> Result<(), ()> {
    let dest = get_openvrpaths_vrpath_path();
    checkerr!(set_file_readonly(&dest, false));
    backup_steam_openvrpaths();
    checkerr!(dump_current_openvrpaths(&build_profile_openvrpaths(profile)));
    checkerr!(set_file_readonly(&dest, true));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{dump_openvrpaths_to_path, get_openvrpaths_from_path, OpenVrPaths};

    #[test]
    fn can_read_openvrpaths_vrpath_steamvr() {
        let ovrp = get_openvrpaths_from_path(&"./test/files/openvrpaths.vrpath".into()).unwrap();
        assert_eq!(ovrp.config.len(), 1);
        assert_eq!(
            ovrp.config.get(0).unwrap(),
            "/home/user/.local/share/Steam/config"
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
        dump_openvrpaths_to_path(&ovrp, &"./target/testout/openvrpaths.vrpath".into())
            .expect("could not dump openvrpaths to path");
    }
}
