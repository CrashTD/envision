use crate::{
    file_utils::{get_data_dir, get_writer},
    paths::{data_monado_path, data_opencomposite_path, BWRAP_SYSTEM_PREFIX, SYSTEM_PREFIX},
};
use expect_dialog::ExpectDialog;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display, fs::File, io::BufReader, slice::Iter};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum XRServiceType {
    Monado,
    Wivrn,
}

impl XRServiceType {
    pub fn from_string(s: String) -> Self {
        match s.trim().to_lowercase().as_str() {
            "monado" => Self::Monado,
            "wivrn" => Self::Wivrn,
            _ => Self::Monado,
        }
    }

    pub fn iter() -> Iter<'static, XRServiceType> {
        [Self::Monado, Self::Wivrn].iter()
    }

    pub fn as_number(&self) -> u32 {
        match self {
            Self::Monado => 0, Self::Wivrn => 1,
        }
    }
}

impl Display for XRServiceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Monado => "Monado",
            Self::Wivrn => "WiVRn",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub xrservice_path: String,
    pub xrservice_type: XRServiceType,
    pub opencomposite_path: String,
    pub libsurvive_path: Option<String>,
    pub basalt_path: Option<String>,
    pub mercury_path: Option<String>,
    pub libsurvive_enabled: bool,
    pub basalt_enabled: bool,
    pub mercury_enabled: bool,
    pub environment: HashMap<String, String>,
    /** Install prefix */
    pub prefix: String,
    pub can_be_built: bool,
    pub editable: bool,
}

impl Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name.to_string())
    }
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            name: "Default profile name".into(),
            xrservice_path: data_monado_path(),
            xrservice_type: XRServiceType::Monado,
            opencomposite_path: data_opencomposite_path(),
            basalt_path: None,
            mercury_path: None,
            libsurvive_path: None,
            basalt_enabled: false,
            mercury_enabled: false,
            libsurvive_enabled: false,
            environment: HashMap::new(),
            prefix: format!(
                "{data}/prefixes/default_profile_prefix",
                data = get_data_dir()
            ),
            can_be_built: true,
            editable: true,
        }
    }
}

impl Profile {
    pub fn get_steam_launch_options(&self) -> String {
        let mut opts = vec![];
        opts.push(format!(
            "XR_RUNTIME_JSON={prefix}/share/openxr/1/openxr_{runtime}.json",
            prefix = match self.prefix.as_str() {
                SYSTEM_PREFIX => BWRAP_SYSTEM_PREFIX,
                other => other,
            },
            runtime = match self.xrservice_type {
                XRServiceType::Monado => "monado",
                XRServiceType::Wivrn => "wivrn",
            }
        ));
        opts.push(format!(
            "PRESSURE_VESSEL_FILESYSTEMS_RW=$XDG_RUNTIME_DIR/{xrservice}_comp_ipc",
            xrservice = match self.xrservice_type {
                XRServiceType::Monado => "monado",
                XRServiceType::Wivrn => "wivrn",
            }
        ));
        opts.push("%command%".into());
        opts.join(" ")
    }

    pub fn load_profile(path: &String) -> Self {
        let file = File::open(path).expect_dialog("Unable to open profile");
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).expect_dialog("Faiuled to deserialize profile")
    }

    pub fn dump_profile(&self, path_s: &String) -> () {
        let writer = get_writer(path_s);
        serde_json::to_writer_pretty(writer, self).expect_dialog("Could not write profile")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::profile::XRServiceType;

    use super::Profile;

    #[test]
    fn profile_can_be_loaded() {
        let profile = Profile::load_profile(&"./test/files/profile.json".to_string());
        assert_eq!(profile.name, "Demo profile");
        assert_eq!(profile.xrservice_path, "/home/user/monado");
        assert_eq!(profile.opencomposite_path, "/home/user/opencomposite");
        assert_eq!(profile.prefix, "/home/user/rex2prefix");
        assert_eq!(
            profile.libsurvive_path.as_deref(),
            Some("/home/user/libsurvive")
        );
        assert_eq!(profile.mercury_path, None);
        assert_eq!(profile.basalt_path, None);
        assert_eq!(profile.libsurvive_enabled, true);
        assert_eq!(profile.basalt_enabled, false);
        assert_eq!(profile.mercury_enabled, false);
        assert!(profile
            .environment
            .contains_key("XRT_COMPOSITOR_SCALE_PERCENTAGE"));
        assert!(profile.environment.contains_key("XRT_COMPOSITOR_COMPUTE"));
        assert!(profile
            .environment
            .contains_key("SURVIVE_GLOBALSCENESOLVER"));
    }

    #[test]
    fn profile_can_be_dumped() {
        let mut env = HashMap::new();
        env.insert("XRT_COMPOSITOR_SCALE_PERCENTAGE".into(), "140".into());
        env.insert("XRT_COMPOSITOR_COMPUTE".into(), "1".into());
        let p = Profile {
            name: "Demo profile".into(),
            xrservice_path: String::from("/home/user/monado"),
            xrservice_type: XRServiceType::Monado,
            opencomposite_path: String::from("/home/user/opencomposite"),
            libsurvive_path: Some(String::from("/home/user/libsurvive")),
            basalt_path: None,
            mercury_path: None,
            libsurvive_enabled: true,
            basalt_enabled: false,
            mercury_enabled: false,
            environment: env,
            prefix: String::from("/home/user/rex2prefix"),
            can_be_built: true,
            editable: true,
        };
        let fpath = String::from("./target/testout/testprofile.json");
        p.dump_profile(&fpath);
        let loaded = Profile::load_profile(&fpath);
        assert_eq!(loaded.name, "Demo profile");
        assert_eq!(
            loaded.libsurvive_path,
            Some(String::from("/home/user/libsurvive"))
        );
        assert_eq!(
            loaded
                .environment
                .get("XRT_COMPOSITOR_COMPUTE")
                .expect("Key XRT_COMPOSITOR_COMPUTE not found"),
            "1"
        );
    }
}
