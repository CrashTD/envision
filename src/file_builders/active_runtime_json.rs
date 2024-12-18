use crate::{
    paths::SYSTEM_PREFIX,
    profile::Profile,
    util::file_utils::{deserialize_file, get_writer, set_file_readonly},
    xdg::XDG,
};
use anyhow::bail;
use serde::{Deserialize, Serialize};
use std::{
    fs::{create_dir_all, remove_file, rename},
    os::unix::fs::symlink,
    path::{Path, PathBuf},
};
use tracing::{debug, warn};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveRuntimeInnerRuntime {
    #[serde(rename = "VALVE_runtime_is_steamvr")]
    pub valve_runtime_is_steamvr: Option<bool>,
    #[serde(rename = "MND_libmonado_path")]
    pub libmonado_path: Option<PathBuf>,
    pub library_path: PathBuf,
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveRuntime {
    pub file_format_version: String,
    pub runtime: ActiveRuntimeInnerRuntime,
}

pub fn get_openxr_conf_dir() -> PathBuf {
    XDG.get_config_home().join("openxr")
}

fn get_active_runtime_json_path() -> PathBuf {
    get_openxr_conf_dir().join("1/active_runtime.json")
}

fn get_active_runtime_from_path(path: &Path) -> Option<ActiveRuntime> {
    deserialize_file(path)
}

pub fn get_current_active_runtime() -> Option<ActiveRuntime> {
    get_active_runtime_from_path(&get_active_runtime_json_path())
}

fn dump_active_runtime_to_path(active_runtime: &ActiveRuntime, path: &Path) -> anyhow::Result<()> {
    if path.is_symlink() {
        remove_file(path)?;
    }
    let writer = get_writer(path)?;
    serde_json::to_writer_pretty(writer, active_runtime)?;
    Ok(())
}

pub fn dump_current_active_runtime(active_runtime: &ActiveRuntime) -> anyhow::Result<()> {
    dump_active_runtime_to_path(active_runtime, &get_active_runtime_json_path())
}

pub fn build_profile_active_runtime(profile: &Profile) -> anyhow::Result<ActiveRuntime> {
    let Some(libopenxr_path) = profile.libopenxr_so() else {
        anyhow::bail!(
            "Could not find path to {}!",
            profile.xrservice_type.libopenxr_path()
        );
    };

    Ok(ActiveRuntime {
        file_format_version: "1.0.0".into(),
        runtime: ActiveRuntimeInnerRuntime {
            name: None,
            valve_runtime_is_steamvr: None,
            libmonado_path: profile.libmonado_so(),
            library_path: libopenxr_path,
        },
    })
}

// for system installs
fn relativize_active_runtime_lib_path(ar: &ActiveRuntime, path: &Path) -> ActiveRuntime {
    let mut res = ar.clone();
    let mut rel_chain = path
        .components()
        .map(|_| String::from(".."))
        .collect::<Vec<String>>();
    rel_chain.pop();
    rel_chain.pop();
    res.runtime.library_path = PathBuf::from(format!(
        "{rels}{fullpath}",
        rels = rel_chain.join("/"),
        fullpath = ar.runtime.library_path.to_string_lossy()
    ));
    res
}

const ACTIVE_RUNTIME_BAK: &str = "active_runtime.json.envision.bak";

pub fn set_current_active_runtime_to_profile(profile: &Profile) -> anyhow::Result<()> {
    let dest = get_active_runtime_json_path();
    if dest.is_dir() {
        bail!("{} is a directory", dest.to_string_lossy());
    }
    if !dest.is_symlink() {
        set_file_readonly(&dest, false)?;
    }
    if dest.is_file() || dest.is_symlink() {
        rename(&dest, dest.parent().unwrap().join(ACTIVE_RUNTIME_BAK))?;
    } else {
        debug!("no active_runtime.json file to backup")
    }

    let profile_openxr_json = profile.openxr_json_path();
    if profile_openxr_json.is_file() {
        create_dir_all(dest.parent().unwrap())?;
        symlink(profile_openxr_json, &dest)?;
    } else {
        warn!("profile openxr json file doesn't exist");
        // fallback: build the file from scratch
        let pfx = profile.clone().prefix;
        let mut ar = build_profile_active_runtime(profile)?;
        // hack: relativize libopenxr_monado.so path for system installs
        if pfx == PathBuf::from(SYSTEM_PREFIX) {
            ar = relativize_active_runtime_lib_path(&ar, &dest);
        }
        dump_current_active_runtime(&ar)?;
        set_file_readonly(&dest, true)?;
    }
    Ok(())
}

pub fn remove_current_active_runtime() -> anyhow::Result<()> {
    let dest = get_active_runtime_json_path();
    if dest.is_dir() {
        bail!("{} is a directory", dest.to_string_lossy());
    }
    if !dest.exists() {
        debug!("no current active_runtime.json to remove")
    }
    Ok(remove_file(dest)?)
}

pub fn restore_active_runtime_backup() -> anyhow::Result<()> {
    let dest = get_active_runtime_json_path();
    let bak = dest.parent().unwrap().join(ACTIVE_RUNTIME_BAK);
    if bak.is_file() || bak.is_symlink() {
        if dest.is_dir() {
            bail!("{} is a directory", dest.to_string_lossy());
        }
        if !dest.is_symlink() {
            set_file_readonly(&dest, false)?;
        }
        rename(&bak, &dest)?;
    } else {
        debug!("{ACTIVE_RUNTIME_BAK} does not exist, nothing to restore");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{
        dump_active_runtime_to_path, get_active_runtime_from_path,
        relativize_active_runtime_lib_path, ActiveRuntime, ActiveRuntimeInnerRuntime,
    };

    #[test]
    fn can_read_active_runtime_json_steamvr() {
        let ar =
            get_active_runtime_from_path(Path::new("./test/files/active_runtime.json.steamvr"))
                .unwrap();
        assert_eq!(ar.file_format_version, "1.0.0");
        assert!(ar.runtime.valve_runtime_is_steamvr.unwrap());
        assert_eq!(
            ar.runtime.library_path,
            PathBuf::from(
                "/home/user/.local/share/Steam/steamapps/common/SteamVR/bin/linux64/vrclient.so"
            )
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
                libmonado_path: None,
                name: Some("SteamVR".into()),
            },
        };
        dump_active_runtime_to_path(
            &ar,
            Path::new("./target/testout/active_runtime.json.steamvr"),
        )
        .expect("could not dump active runtime to path");
    }

    #[test]
    fn can_relativize_path() {
        let ar = ActiveRuntime {
            file_format_version: "1.0.0".into(),
            runtime: ActiveRuntimeInnerRuntime {
                valve_runtime_is_steamvr: None,
                library_path: "/usr/lib64/libopenxr_monado.so".into(),
                libmonado_path: None,
                name: None,
            },
        };
        let relativized = relativize_active_runtime_lib_path(
            &ar,
            Path::new("/home/user/.config/openxr/1/active_runtime.json"),
        );
        assert_eq!(
            relativized
                .runtime
                .library_path
                .to_string_lossy()
                .to_string(),
            "../../../../../usr/lib64/libopenxr_monado.so"
        );
    }
}
