use std::{fs::File, io::BufReader, path::Path};

use expect_dialog::ExpectDialog;
use serde::{Deserialize, Serialize};

use crate::{file_utils::{get_config_dir, get_data_dir, get_writer, set_file_radonly}, profile::Profile};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveRuntimeInnerRuntime {
    #[serde(rename = "VALVE_runtime_is_steamvr")]
    pub valve_runtime_is_steamvr: Option<bool>,
    pub library_path: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveRuntime {
    pub file_format_version: String,
    pub runtime: ActiveRuntimeInnerRuntime,
}

fn get_active_runtime_json_path() -> String {
    format!(
        "{config}/openxr/1/active_runtime.json",
        config = get_config_dir()
    )
}

pub fn is_steam(active_runtime: ActiveRuntime) -> bool {
    match active_runtime.runtime.valve_runtime_is_steamvr {
        Some(true) => true,
        _ => false,
    }
}

fn get_active_runtime_from_path(path_s: String) -> Option<ActiveRuntime> {
    let path = Path::new(&path_s);
    if !(path.is_file() || path.is_symlink()) {
        return None;
    }
    let fd = File::open(path).expect_dialog("Unable to open active_runtime.json");
    let reader = BufReader::new(fd);
    Some(serde_json::from_reader(reader).expect_dialog("Failed to deserialize active_runtime.json"))
}

pub fn get_current_active_runtime() -> Option<ActiveRuntime> {
    get_active_runtime_from_path(get_active_runtime_json_path())
}

fn dump_active_runtime_to_path(active_runtime: ActiveRuntime, path_s: String) {
    set_file_radonly(&path_s, false);
    let writer = get_writer(&path_s);
    // TODO: deal with write protection
    serde_json::to_writer_pretty(writer, &active_runtime)
        .expect_dialog("Unable to save active runtime");
    set_file_radonly(&path_s, true);
}

pub fn dump_current_active_runtime(active_runtime: ActiveRuntime) {
    dump_active_runtime_to_path(active_runtime, get_active_runtime_json_path());
}

fn build_steam_active_runtime() -> ActiveRuntime {
    ActiveRuntime {
        file_format_version: "1.0.0".into(),
        runtime: ActiveRuntimeInnerRuntime {
            valve_runtime_is_steamvr: Some(true),
            library_path: format!(
                "{data}/Steam/steamapps/common/SteamVR/bin/linux64/vrclient.so",
                data = get_data_dir()
            ),
            name: Some("SteamVR".into()),
        },
    }
}

pub fn set_current_active_runtime_to_steam() {
    dump_current_active_runtime(build_steam_active_runtime())
}

fn build_profile_active_runtime(profile: Profile) -> ActiveRuntime {
    ActiveRuntime {
        file_format_version: "1.0.0".into(),
        runtime: ActiveRuntimeInnerRuntime {
            name: None,
            valve_runtime_is_steamvr: None,
            library_path: format!(
                "{prefix}/lib/libopenxr_monado.so",
                prefix = profile.prefix
            ),
        },
    }
}

pub fn set_current_active_runtime_to_profile(profile: Profile) {
    dump_current_active_runtime(build_profile_active_runtime(profile))
}

#[cfg(test)]
mod tests {
    use super::{
        dump_active_runtime_to_path, get_active_runtime_from_path, ActiveRuntime,
        ActiveRuntimeInnerRuntime,
    };

    #[test]
    fn can_read_active_runtime_json_steamvr() {
        let ar = get_active_runtime_from_path("./test/files/active_runtime.json.steamvr".into())
            .unwrap();
        assert_eq!(ar.file_format_version, "1.0.0");
        assert!(ar.runtime.valve_runtime_is_steamvr.unwrap());
        assert_eq!(
            ar.runtime.library_path,
            "/home/user/.local/share/Steam/steamapps/common/SteamVR/bin/linux64/vrclient.so"
        );
        assert_eq!(ar.runtime.name.unwrap(), "SteamVR");
    }

    #[test]
    fn can_dump_active_runtime_json() {
        let ar = ActiveRuntime {
            file_format_version: "1.0.0".into(),
            runtime: ActiveRuntimeInnerRuntime {
                valve_runtime_is_steamvr: Some(true),
                library_path:
                    "/home/user/.local/share/Steam/steamapps/common/SteamVR/bin/linux64/vrclient.so"
                        .into(),
                name: Some("SteamVR".into()),
            },
        };
        dump_active_runtime_to_path(ar, "./target/testout/active_runtime.json.steamvr".into());
    }
}
