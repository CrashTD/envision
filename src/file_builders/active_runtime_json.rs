use crate::{
    checkerr,
    file_utils::{copy_file, deserialize_file, get_writer, set_file_readonly},
    paths::{get_backup_dir, get_xdg_config_dir, get_xdg_data_dir, SYSTEM_PREFIX},
    profile::{Profile, XRServiceType},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

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

pub fn get_openxr_conf_dir() -> String {
    format!("{config}/openxr", config = get_xdg_config_dir())
}

fn get_active_runtime_json_path() -> String {
    format!(
        "{config}/1/active_runtime.json",
        config = get_openxr_conf_dir()
    )
}

pub fn is_steam(active_runtime: &ActiveRuntime) -> bool {
    match active_runtime.runtime.valve_runtime_is_steamvr {
        Some(true) => true,
        _ => false,
    }
}

fn get_backup_steam_active_runtime_path() -> String {
    format!(
        "{backup}/active_runtime.json.steam.bak",
        backup = get_backup_dir()
    )
}

fn get_backed_up_steam_active_runtime() -> Option<ActiveRuntime> {
    get_active_runtime_from_path(&get_backup_steam_active_runtime_path())
}

fn backup_steam_active_runtime() {
    let ar = get_current_active_runtime();
    if ar.is_some() {
        if is_steam(&ar.unwrap()) {
            copy_file(
                &get_active_runtime_json_path(),
                &get_backup_steam_active_runtime_path(),
            );
        }
    }
}

fn get_active_runtime_from_path(path_s: &String) -> Option<ActiveRuntime> {
    deserialize_file(path_s)
}

pub fn get_current_active_runtime() -> Option<ActiveRuntime> {
    get_active_runtime_from_path(&get_active_runtime_json_path())
}

fn dump_active_runtime_to_path(
    active_runtime: &ActiveRuntime,
    path_s: &String,
) -> Result<(), serde_json::Error> {
    let writer = get_writer(path_s);
    serde_json::to_writer_pretty(writer, active_runtime)
}

pub fn dump_current_active_runtime(
    active_runtime: &ActiveRuntime,
) -> Result<(), serde_json::Error> {
    dump_active_runtime_to_path(active_runtime, &get_active_runtime_json_path())
}

fn build_steam_active_runtime() -> ActiveRuntime {
    let backup = get_backed_up_steam_active_runtime();
    if backup.is_some() {
        return backup.unwrap();
    }
    ActiveRuntime {
        file_format_version: "1.0.0".into(),
        runtime: ActiveRuntimeInnerRuntime {
            valve_runtime_is_steamvr: Some(true),
            library_path: format!(
                "{data}/Steam/steamapps/common/SteamVR/bin/linux64/vrclient.so",
                data = get_xdg_data_dir()
            ),
            name: Some("SteamVR".into()),
        },
    }
}

pub fn set_current_active_runtime_to_steam() -> Result<(), ()> {
    checkerr!(set_file_readonly(&get_active_runtime_json_path(), false));
    checkerr!(dump_current_active_runtime(&build_steam_active_runtime()));
    Ok(())
}

pub fn build_profile_active_runtime(profile: &Profile) -> ActiveRuntime {
    ActiveRuntime {
        file_format_version: "1.0.0".into(),
        runtime: ActiveRuntimeInnerRuntime {
            name: None,
            valve_runtime_is_steamvr: None,
            library_path: format!(
                "{prefix}/lib/libopenxr_{xrservice}.so",
                prefix = profile.prefix,
                xrservice = match profile.xrservice_type {
                    XRServiceType::Monado => "monado",
                    XRServiceType::Wivrn => "wivrn",
                }
            ),
        },
    }
}

pub fn set_current_active_runtime_to_profile(profile: &Profile) -> Result<(), ()> {
    let dest = get_active_runtime_json_path();
    checkerr!(set_file_readonly(&dest, false));
    backup_steam_active_runtime();
    let pfx = profile.clone().prefix;
    let mut ar = build_profile_active_runtime(profile);
    // hack: relativize libopenxr_monado.so path for system installs
    if pfx == SYSTEM_PREFIX {
        let path = Path::new(&dest);
        let mut rel_chain = path
            .components()
            .map(|_| String::from(".."))
            .collect::<Vec<String>>();
        rel_chain.pop();
        rel_chain.pop();
        ar.runtime.library_path = format!(
            "{rels}/usr/lib/libopenxr_monado.so",
            rels = rel_chain.join("/")
        );
    }
    checkerr!(dump_current_active_runtime(&ar));
    checkerr!(set_file_readonly(&dest, true));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        dump_active_runtime_to_path, get_active_runtime_from_path, ActiveRuntime,
        ActiveRuntimeInnerRuntime,
    };

    #[test]
    fn can_read_active_runtime_json_steamvr() {
        let ar = get_active_runtime_from_path(&"./test/files/active_runtime.json.steamvr".into())
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
        dump_active_runtime_to_path(&ar, &"./target/testout/active_runtime.json.steamvr".into())
            .expect("could not dump active runtime to path");
    }
}
