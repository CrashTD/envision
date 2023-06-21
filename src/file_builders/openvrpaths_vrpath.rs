use crate::{
    file_utils::{
        deserialize_file, get_writer, get_xdg_config_dir, get_xdg_data_dir, set_file_radonly,
    },
    profile::Profile,
};
use expect_dialog::ExpectDialog;
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

fn get_openvrpaths_vrpath_path() -> String {
    format!(
        "{config}/openvr/openvrpaths.vrpath",
        config = get_xdg_config_dir()
    )
}

pub fn is_steam(ovr_paths: OpenVrPaths) -> bool {
    ovr_paths
        .runtime
        .iter()
        .find(|rt| {
            rt.to_lowercase()
                .ends_with("/steam/steamapps/common/steamvr")
        })
        .is_some()
}

fn get_openvrpaths_from_path(path_s: &String) -> Option<OpenVrPaths> {
    deserialize_file(path_s)
}

pub fn get_current_openvrpaths() -> Option<OpenVrPaths> {
    get_openvrpaths_from_path(&get_openvrpaths_vrpath_path())
}

fn dump_openvrpaths_to_path(ovr_paths: OpenVrPaths, path_s: String) {
    set_file_radonly(&path_s, false);
    let writer = get_writer(&path_s);
    serde_json::to_writer_pretty(writer, &ovr_paths).expect_dialog("Unable to save openvrpaths");
    set_file_radonly(&path_s, true);
}

pub fn dump_current_openvrpaths(ovr_paths: OpenVrPaths) {
    dump_openvrpaths_to_path(ovr_paths, get_openvrpaths_vrpath_path())
}

fn build_steam_openvrpaths() -> OpenVrPaths {
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

pub fn set_current_openvrpaths_to_steam() {
    dump_current_openvrpaths(build_steam_openvrpaths())
}

fn build_profile_openvrpaths(profile: Profile) -> OpenVrPaths {
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

pub fn set_current_openvrpaths_to_profile(profile: Profile) {
    dump_current_openvrpaths(build_profile_openvrpaths(profile))
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
        dump_openvrpaths_to_path(ovrp, "./target/testout/openvrpaths.vrpath".into())
    }
}
