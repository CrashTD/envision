use std::{fs::File, io::BufReader, path::Path};

use serde::{Deserialize, Serialize};

use crate::file_utils::{get_config_dir, get_writer};

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
        config = get_config_dir()
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

fn get_openvrpaths_from_path(path_s: String) -> Option<OpenVrPaths> {
    let path = Path::new(&path_s);
    if !(path.is_file() || path.is_symlink()) {
        return None;
    }
    let fd = File::open(path).expect("Unable to open openvrpaths.vrpath");
    let reader = BufReader::new(fd);
    Some(serde_json::from_reader(reader).expect("Failed to deserialize openvrpaths.vrpath"))
}

pub fn get_current_openvrpaths() -> Option<OpenVrPaths> {
    get_openvrpaths_from_path(get_openvrpaths_vrpath_path())
}

fn dump_openvrpaths_to_path(ovr_paths: OpenVrPaths, path_s: String) {
    let writer = get_writer(&path_s);
    serde_json::to_writer_pretty(writer, &ovr_paths).expect("Unable to save openvrpaths");
}

pub fn dump_current_openvrpaths(ovr_paths: OpenVrPaths) {
    dump_openvrpaths_to_path(ovr_paths, get_openvrpaths_vrpath_path())
}

#[cfg(test)]
mod tests {
    use super::{dump_openvrpaths_to_path, get_openvrpaths_from_path, OpenVrPaths};

    #[test]
    fn can_read_openvrpaths_vrpath_steamvr() {
        let ovrp = get_openvrpaths_from_path("./test/files/openvrpaths.vrpath".into()).unwrap();
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
