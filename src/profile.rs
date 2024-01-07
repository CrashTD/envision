use crate::{
    file_utils::get_writer,
    paths::{
        data_monado_path, data_opencomposite_path, get_data_dir, get_ipc_file_path,
        BWRAP_SYSTEM_PREFIX, SYSTEM_PREFIX,
    },
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display, fs::File, io::BufReader, path::Path, slice::Iter};
use uuid::Uuid;

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

    pub fn iter() -> Iter<'static, Self> {
        [Self::Monado, Self::Wivrn].iter()
    }

    pub fn as_number(&self) -> u32 {
        match self {
            Self::Monado => 0,
            Self::Wivrn => 1,
        }
    }

    pub fn from_number(i: u32) -> Self {
        match i {
            0 => Self::Monado,
            1 => Self::Wivrn,
            _ => panic!("XRServiceType index out of bounds"),
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
pub enum ProfileFeatureType {
    Libsurvive,
    Basalt,
    OpenHmd,
}

impl ProfileFeatureType {
    pub fn from_string(s: String) -> Self {
        match s.trim().to_lowercase().as_str() {
            "libsurvive" => Self::Libsurvive,
            "basalt" => Self::Basalt,
            _ => panic!("Unknown profile feature type"),
        }
    }

    pub fn iter() -> Iter<'static, ProfileFeatureType> {
        [Self::Libsurvive, Self::Basalt].iter()
    }
}

impl Display for ProfileFeatureType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Libsurvive => "Libsurvive",
            Self::Basalt => "Basalt",
            Self::OpenHmd => "OpenHMD",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileFeature {
    pub feature_type: ProfileFeatureType,
    pub enabled: bool,
    pub path: Option<String>,
    pub repo: Option<String>,
    pub branch: Option<String>,
}

impl Default for ProfileFeature {
    fn default() -> Self {
        Self {
            feature_type: ProfileFeatureType::Libsurvive,
            enabled: false,
            path: None,
            repo: None,
            branch: None,
        }
    }
}

impl ProfileFeature {
    pub fn default_libsurvive() -> Self {
        Self {
            feature_type: ProfileFeatureType::Libsurvive,
            ..Default::default()
        }
    }

    pub fn default_basalt() -> Self {
        Self {
            feature_type: ProfileFeatureType::Basalt,
            ..Default::default()
        }
    }

    pub fn default_openhmd() -> Self {
        Self {
            feature_type: ProfileFeatureType::OpenHmd,
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileFeatures {
    pub libsurvive: ProfileFeature,
    pub basalt: ProfileFeature,
    #[serde(default = "ProfileFeature::default_openhmd")]
    pub openhmd: ProfileFeature,
    pub mercury_enabled: bool,
}

impl Default for ProfileFeatures {
    fn default() -> Self {
        Self {
            libsurvive: ProfileFeature::default_libsurvive(),
            basalt: ProfileFeature::default_basalt(),
            openhmd: ProfileFeature::default_openhmd(),
            mercury_enabled: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LighthouseDriver {
    Vive,
    Survive,
    SteamVR,
}

impl Default for LighthouseDriver {
    fn default() -> Self {
        Self::Vive
    }
}

impl LighthouseDriver {
    pub fn from_string(s: String) -> Self {
        match s.trim().to_lowercase().as_str() {
            "vive" => Self::Vive,
            "survive" => Self::Survive,
            "libsurvive" => Self::Survive,
            "steam" => Self::SteamVR,
            "steamvr" => Self::SteamVR,
            _ => Self::Vive,
        }
    }

    pub fn iter() -> Iter<'static, Self> {
        [Self::Vive, Self::Survive, Self::SteamVR].iter()
    }

    pub fn as_number(&self) -> u32 {
        match self {
            Self::Vive => 0,
            Self::Survive => 1,
            Self::SteamVR => 2,
        }
    }

    pub fn from_number(i: u32) -> Self {
        match i {
            0 => Self::Vive,
            1 => Self::Survive,
            2 => Self::SteamVR,
            _ => panic!("LighthouseDriver index out of bounds"),
        }
    }
}

impl Display for LighthouseDriver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Vive => "Vive",
            Self::Survive => "Survive",
            Self::SteamVR => "SteamVR",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub uuid: String,
    pub name: String,
    pub xrservice_type: XRServiceType,
    pub xrservice_path: String,
    pub xrservice_repo: Option<String>,
    pub xrservice_branch: Option<String>,
    #[serde(default = "HashMap::<String, String>::default")]
    pub xrservice_cmake_flags: HashMap<String, String>,
    pub opencomposite_path: String,
    pub opencomposite_repo: Option<String>,
    pub opencomposite_branch: Option<String>,
    pub features: ProfileFeatures,
    pub environment: HashMap<String, String>,
    /** Install prefix */
    pub prefix: String,
    pub can_be_built: bool,
    pub editable: bool,
    pub pull_on_build: bool,
    #[serde(default = "LighthouseDriver::default")]
    /** Only applicable for Monado */
    pub lighthouse_driver: LighthouseDriver,
    #[serde(default = "String::default")]
    pub xrservice_launch_options: String,
}

impl Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name.to_string())
    }
}

impl Default for Profile {
    fn default() -> Self {
        let uuid = Uuid::new_v4().to_string();
        let profile_dir = format!("{}/{}", get_data_dir(), uuid);
        Self {
            name: "Default profile name".into(),
            xrservice_path: format!("{}/xrservice", profile_dir),
            xrservice_type: XRServiceType::Monado,
            xrservice_repo: None,
            xrservice_branch: None,
            xrservice_cmake_flags: HashMap::<String, String>::default(),
            features: ProfileFeatures {
                libsurvive: ProfileFeature {
                    enabled: false,
                    path: Some(format!("{}/libsurvive", profile_dir)),
                    repo: None,
                    branch: None,
                    feature_type: ProfileFeatureType::Libsurvive,
                },
                basalt: ProfileFeature {
                    enabled: false,
                    path: Some(format!("{}/basalt", profile_dir)),
                    repo: None,
                    branch: None,
                    feature_type: ProfileFeatureType::Basalt,
                },
                openhmd: ProfileFeature {
                    enabled: false,
                    path: Some(format!("{}/openhmd", profile_dir)),
                    repo: None,
                    branch: None,
                    feature_type: ProfileFeatureType::OpenHmd,
                },
                mercury_enabled: false,
            },
            environment: HashMap::new(),
            prefix: format!("{}/prefixes/{}", get_data_dir(), uuid),
            can_be_built: true,
            pull_on_build: true,
            opencomposite_path: format!("{}/opencomposite", profile_dir),
            opencomposite_repo: None,
            opencomposite_branch: None,
            editable: true,
            lighthouse_driver: LighthouseDriver::default(),
            xrservice_launch_options: String::default(),
            uuid,
        }
    }
}

impl Profile {
    pub fn get_steam_launch_options(&self) -> String {
        vec![
            // format!(
            //     "VR_OVERRIDE={opencomp}/build",
            //     opencomp = self.opencomposite_path,
            // ),
            format!(
                "XR_RUNTIME_JSON={prefix}/share/openxr/1/openxr_{runtime}.json",
                prefix = match self.prefix.as_str() {
                    SYSTEM_PREFIX => BWRAP_SYSTEM_PREFIX,
                    other => other,
                },
                runtime = match self.xrservice_type {
                    XRServiceType::Monado => "monado",
                    XRServiceType::Wivrn => "wivrn",
                }
            ),
            format!(
                "PRESSURE_VESSEL_FILESYSTEMS_RW={path}",
                path = get_ipc_file_path(&self.xrservice_type),
            ),
            "%command%".into(),
        ]
        .join(" ")
    }

    pub fn get_survive_cli_path(&self) -> Option<String> {
        let path_s = format!("{pfx}/bin/survive-cli", pfx = self.prefix);
        if Path::new(&path_s).is_file() {
            return Some(path_s);
        }
        None
    }

    pub fn load_profile(path: &String) -> Self {
        let file = File::open(path).expect("Unable to open profile");
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).expect("Faiuled to deserialize profile")
    }

    pub fn dump_profile(&self, path_s: &String) {
        let writer = get_writer(path_s);
        serde_json::to_writer_pretty(writer, self).expect("Could not write profile")
    }

    pub fn create_duplicate(&self) -> Self {
        if !self.can_be_built {
            let mut dup = self.clone();
            dup.uuid = Uuid::new_v4().to_string();
            dup.name = format!("Duplicate of {}", dup.name);
            dup.editable = true;
            return dup;
        }
        let mut dup = Self::default();
        dup.name = format!("Duplicate of {}", self.name);
        dup.xrservice_type = self.xrservice_type.clone();
        dup.xrservice_repo = self.xrservice_repo.clone();
        dup.xrservice_branch = self.xrservice_branch.clone();
        dup.xrservice_cmake_flags = self.xrservice_cmake_flags.clone();
        dup.features.libsurvive.enabled = self.features.libsurvive.enabled;
        dup.features.libsurvive.repo = self.features.libsurvive.repo.clone();
        dup.features.libsurvive.branch = self.features.libsurvive.branch.clone();
        dup.features.basalt.enabled = self.features.basalt.enabled;
        dup.features.basalt.repo = self.features.basalt.repo.clone();
        dup.features.basalt.branch = self.features.basalt.branch.clone();
        dup.features.openhmd.enabled = self.features.openhmd.enabled;
        dup.features.openhmd.repo = self.features.openhmd.repo.clone();
        dup.features.openhmd.branch = self.features.openhmd.branch.clone();
        dup.features.mercury_enabled = self.features.mercury_enabled;
        dup.environment = self.environment.clone();
        dup.pull_on_build = self.pull_on_build;
        dup.opencomposite_repo = self.opencomposite_repo.clone();
        dup.opencomposite_branch = self.opencomposite_branch.clone();
        dup.lighthouse_driver = self.lighthouse_driver;
        dup.xrservice_launch_options = self.xrservice_launch_options.clone();
        dup
    }

    pub fn validate(&self) -> bool {
        !self.name.is_empty()
            && self.editable
            && !self.uuid.is_empty()
            && !self.xrservice_path.is_empty()
            && !self.prefix.is_empty()
            && (!self.features.libsurvive.enabled
                || !self
                    .features
                    .libsurvive
                    .path
                    .as_ref()
                    .unwrap_or(&"".to_string())
                    .is_empty())
            && (!self.features.basalt.enabled
                || !self
                    .features
                    .basalt
                    .path
                    .as_ref()
                    .unwrap_or(&"".to_string())
                    .is_empty())
            && (!self.features.openhmd.enabled
                || !self
                    .features
                    .openhmd
                    .path
                    .as_ref()
                    .unwrap_or(&"".to_string())
                    .is_empty())
    }

    pub fn xrservice_binary(&self) -> String {
        format!(
            "{}/bin/{}",
            self.prefix,
            match self.xrservice_type {
                XRServiceType::Monado => "monado-service",
                XRServiceType::Wivrn => "wivrn-server",
            }
        )
    }

    pub fn can_start(&self) -> bool {
        Path::new(&self.xrservice_binary()).is_file()
    }

    pub fn libmonado_so(&self) -> Option<String> {
        let mut res = format!("{}/lib/libmonado.so", self.prefix);
        if Path::new(&res).is_file() {
            return Some(res);
        }
        res = format!("{}/lib64/libmonado.so", self.prefix);
        if Path::new(&res).is_file() {
            return Some(res);
        }

        None
    }

    pub fn has_libmonado(&self) -> bool {
        self.libmonado_so().is_some()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::profile::{ProfileFeature, ProfileFeatureType, ProfileFeatures, XRServiceType};

    use super::Profile;

    #[test]
    fn profile_can_be_loaded() {
        let profile = Profile::load_profile(&"./test/files/profile.json".to_string());
        assert_eq!(profile.name, "Demo profile");
        assert_eq!(profile.xrservice_path, "/home/user/monado");
        assert_eq!(profile.opencomposite_path, "/home/user/opencomposite");
        assert_eq!(profile.prefix, "/home/user/envisionprefix");
        assert_eq!(
            profile.features.libsurvive.path.as_deref(),
            Some("/home/user/libsurvive")
        );
        assert_eq!(profile.features.basalt.path, None);
        assert_eq!(profile.features.libsurvive.enabled, true);
        assert_eq!(profile.features.basalt.enabled, false);
        assert_eq!(profile.features.mercury_enabled, false);
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
            uuid: "demo".into(),
            name: "Demo profile".into(),
            xrservice_path: String::from("/home/user/monado"),
            xrservice_type: XRServiceType::Monado,
            opencomposite_path: String::from("/home/user/opencomposite"),
            features: ProfileFeatures {
                libsurvive: ProfileFeature {
                    feature_type: ProfileFeatureType::Libsurvive,
                    enabled: true,
                    path: Some(String::from("/home/user/libsurvive")),
                    repo: None,
                    branch: None,
                },
                basalt: ProfileFeature::default_basalt(),
                openhmd: ProfileFeature::default_openhmd(),
                mercury_enabled: false,
            },
            environment: env,
            prefix: String::from("/home/user/envisionprefix"),
            editable: true,
            ..Default::default()
        };
        let fpath = String::from("./target/testout/testprofile.json");
        p.dump_profile(&fpath);
        let loaded = Profile::load_profile(&fpath);
        assert_eq!(loaded.name, "Demo profile");
        assert_eq!(
            loaded.features.libsurvive.path,
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
