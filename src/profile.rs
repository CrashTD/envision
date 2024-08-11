use crate::{
    file_utils::get_writer,
    paths::{get_data_dir, get_ipc_file_path, BWRAP_SYSTEM_PREFIX, SYSTEM_PREFIX},
};
use nix::NixPath;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::Display,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    slice::Iter,
};
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

    /// relative path from the prefix lib dir of the libopenxr shared object
    pub fn libopenxr_path(&self) -> &'static str {
        match self {
            Self::Monado => "libopenxr_monado.so",
            Self::Wivrn => "wivrn/libopenxr_wivrn.so",
        }
    }

    /// relative path from the prefix lib dir of the libmonado shared object
    pub fn libmonado_path(&self) -> &'static str {
        match self {
            Self::Monado => "libmonado.so",
            Self::Wivrn => "wivrn/libmonado.so",
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
    pub path: Option<PathBuf>,
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
    pub xrservice_path: PathBuf,
    pub xrservice_repo: Option<String>,
    pub xrservice_branch: Option<String>,
    #[serde(default = "HashMap::<String, String>::default")]
    pub xrservice_cmake_flags: HashMap<String, String>,
    pub opencomposite_path: PathBuf,
    pub opencomposite_repo: Option<String>,
    pub opencomposite_branch: Option<String>,
    pub features: ProfileFeatures,
    pub environment: HashMap<String, String>,
    /// Install prefix
    pub prefix: PathBuf,
    pub can_be_built: bool,
    pub editable: bool,
    pub pull_on_build: bool,
    #[serde(default = "LighthouseDriver::default")]
    /// Only applicable for Monado
    pub lighthouse_driver: LighthouseDriver,
    #[serde(default = "String::default")]
    pub xrservice_launch_options: String,
    pub autostart_command: Option<String>,
    #[serde(default)]
    pub skip_dependency_check: bool,
}

impl Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name.to_string())
    }
}

impl Default for Profile {
    fn default() -> Self {
        let uuid = Self::new_uuid();
        let profile_dir = get_data_dir().join(&uuid);
        Self {
            name: "Default profile name".into(),
            xrservice_path: profile_dir.join("xrservice"),
            xrservice_type: XRServiceType::Monado,
            xrservice_repo: None,
            xrservice_branch: None,
            xrservice_cmake_flags: HashMap::<String, String>::default(),
            features: ProfileFeatures {
                libsurvive: ProfileFeature {
                    enabled: false,
                    path: Some(profile_dir.join("libsurvive")),
                    repo: None,
                    branch: None,
                    feature_type: ProfileFeatureType::Libsurvive,
                },
                basalt: ProfileFeature {
                    enabled: false,
                    path: Some(profile_dir.join("basalt")),
                    repo: None,
                    branch: None,
                    feature_type: ProfileFeatureType::Basalt,
                },
                openhmd: ProfileFeature {
                    enabled: false,
                    path: Some(profile_dir.join("openhmd")),
                    repo: None,
                    branch: None,
                    feature_type: ProfileFeatureType::OpenHmd,
                },
                mercury_enabled: false,
            },
            environment: HashMap::new(),
            prefix: get_data_dir().join("prefixes").join(&uuid),
            can_be_built: true,
            pull_on_build: true,
            opencomposite_path: profile_dir.join("opencomposite"),
            opencomposite_repo: None,
            opencomposite_branch: None,
            editable: true,
            lighthouse_driver: LighthouseDriver::default(),
            xrservice_launch_options: String::default(),
            uuid,
            autostart_command: None,
            skip_dependency_check: false,
        }
    }
}

impl Profile {
    pub fn xr_runtime_json_env_var(&self) -> String {
        format!(
            "XR_RUNTIME_JSON=\"{prefix}/share/openxr/1/openxr_{runtime}.json\"",
            prefix = match self.prefix.to_string_lossy().to_string().as_str() {
                SYSTEM_PREFIX => BWRAP_SYSTEM_PREFIX,
                other => other,
            },
            runtime = match self.xrservice_type {
                XRServiceType::Monado => "monado",
                XRServiceType::Wivrn => "wivrn",
            }
        )
    }

    /// always adds the xr runtime json var
    pub fn env_vars_full(&self) -> Vec<String> {
        vec![
            // format!(
            //     "VR_OVERRIDE={opencomp}/build",
            //     opencomp = self.opencomposite_path,
            // ),
            self.xr_runtime_json_env_var(),
            format!(
                "PRESSURE_VESSEL_FILESYSTEMS_RW=\"{path}\"",
                path = get_ipc_file_path(&self.xrservice_type).to_string_lossy(),
            ),
        ]
    }

    pub fn get_env_vars(&self) -> Vec<String> {
        if self.can_be_built {
            return vec![
                "PRESSURE_VESSEL_FILESYSTEMS_RW=\"$XDG_RUNTIME_DIR/wivrn_comp_ipc:$XDG_RUNTIME_DIR/monado_comp_ipc\"".into(),
            ];
        }
        self.env_vars_full()
    }

    pub fn get_steam_launch_options(&self) -> String {
        let mut vars = self.get_env_vars();
        vars.push("%command%".into());
        vars.join(" ")
    }

    pub fn get_survive_cli_path(&self) -> Option<PathBuf> {
        let path = self.prefix.join("bin/survive-cli");
        if path.is_file() {
            return Some(path);
        }
        None
    }

    pub fn load_profile(path: &Path) -> Self {
        let file = File::open(path).expect("Unable to open profile");
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).expect("Faiuled to deserialize profile")
    }

    pub fn dump_profile(&self, path: &Path) {
        let writer = get_writer(path).expect("Could not write profile");
        serde_json::to_writer_pretty(writer, self).expect("Could not write profile")
    }

    pub fn new_uuid() -> String {
        Uuid::new_v4().to_string()
    }

    pub fn create_duplicate(&self) -> Self {
        if !self.can_be_built {
            let mut dup = self.clone();
            dup.uuid = Self::new_uuid();
            dup.name = format!("Duplicate of {}", dup.name);
            dup.editable = true;
            return dup;
        }
        let mut dup = Self {
            name: format!("Duplicate of {}", self.name),
            xrservice_type: self.xrservice_type.clone(),
            xrservice_repo: self.xrservice_repo.clone(),
            xrservice_branch: self.xrservice_branch.clone(),
            xrservice_cmake_flags: self.xrservice_cmake_flags.clone(),
            features: ProfileFeatures {
                libsurvive: ProfileFeature {
                    enabled: self.features.libsurvive.enabled,
                    repo: self.features.libsurvive.repo.clone(),
                    branch: self.features.libsurvive.branch.clone(),
                    ..Default::default()
                },
                basalt: ProfileFeature {
                    enabled: self.features.basalt.enabled,
                    repo: self.features.basalt.repo.clone(),
                    branch: self.features.basalt.branch.clone(),
                    ..Default::default()
                },

                openhmd: ProfileFeature {
                    enabled: self.features.openhmd.enabled,
                    repo: self.features.openhmd.repo.clone(),
                    branch: self.features.openhmd.branch.clone(),
                    ..Default::default()
                },
                mercury_enabled: self.features.mercury_enabled,
            },
            environment: self.environment.clone(),
            autostart_command: self.autostart_command.clone(),
            pull_on_build: self.pull_on_build,
            lighthouse_driver: self.lighthouse_driver,
            opencomposite_repo: self.opencomposite_repo.clone(),
            opencomposite_branch: self.opencomposite_branch.clone(),
            skip_dependency_check: self.skip_dependency_check,
            xrservice_launch_options: self.xrservice_launch_options.clone(),
            ..Default::default()
        };
        if dup.environment.contains_key("LD_LIBRARY_PATH") {
            dup.environment.insert(
                "LD_LIBRARY_PATH".into(),
                prepare_ld_library_path(&dup.prefix),
            );
        }
        dup
    }

    pub fn validate(&self) -> Result<(), Vec<&str>> {
        let mut errors = vec![];
        // impossible
        if !self.editable {
            errors.push("You somehow managed to edit a non-editable profile. Congratulations, you found a bug! Please report it.");
        }
        // impossible
        if self.uuid.is_empty() {
            errors.push("For some reason this profile's UUID is empty. Congratulations, you found a bug! Please report it.");
        }
        if self.name.is_empty() {
            errors.push("The profile name cannot be empty.");
        }
        if self.xrservice_path.is_empty() {
            errors.push("The XR service path cannot be empty.")
        }
        if self.prefix.is_empty() {
            errors.push("The prefix path cannot be empty.")
        }
        if self.features.libsurvive.enabled
            && (self.features.libsurvive.path.is_none()
                || self
                    .features
                    .libsurvive
                    .path
                    .as_ref()
                    .is_some_and(|p| p.is_empty()))
        {
            errors.push("You enabled Libsurvive, but its path is empty. Disable Libsurvive or set a path for it.")
        }
        if self.features.basalt.enabled
            && (self.features.basalt.path.is_none()
                || self
                    .features
                    .basalt
                    .path
                    .as_ref()
                    .is_some_and(|p| p.is_empty()))
        {
            errors.push(
                "You enabled Basalt, but its path is empty. Disable Basalt or set a path for it.",
            )
        }
        if self.features.openhmd.enabled
            && (self.features.openhmd.path.is_none()
                || self
                    .features
                    .openhmd
                    .path
                    .as_ref()
                    .is_some_and(|p| p.is_empty()))
        {
            errors.push(
                "You enabled OpenHMD, but its path is empty. Disable OpenHMD or set a path for it.",
            )
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn xrservice_binary(&self) -> PathBuf {
        self.prefix.join("bin").join(match self.xrservice_type {
            XRServiceType::Monado => "monado-service",
            XRServiceType::Wivrn => "wivrn-server",
        })
    }

    pub fn can_start(&self) -> bool {
        self.xrservice_binary().is_file()
    }

    /// absolute path to a given shared object in the profile prefix
    pub fn find_so(&self, rel_path: &str) -> Option<PathBuf> {
        ["lib", "lib64"]
            .into_iter()
            .map(|lib| self.prefix.join(lib).join(rel_path))
            .find(|path| path.is_file())
    }

    /// absolute path to the libmonado shared object
    pub fn libmonado_so(&self) -> Option<PathBuf> {
        self.find_so(self.xrservice_type.libmonado_path())
    }

    /// absolute path to the libopenxr shared object
    pub fn libopenxr_so(&self) -> Option<PathBuf> {
        self.find_so(self.xrservice_type.libopenxr_path())
    }
}

pub fn prepare_ld_library_path(prefix: &Path) -> String {
    format!("{pfx}/lib:{pfx}/lib64", pfx = prefix.to_string_lossy())
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        path::{Path, PathBuf},
    };

    use crate::profile::{ProfileFeature, ProfileFeatureType, ProfileFeatures, XRServiceType};

    use super::Profile;

    #[test]
    fn profile_can_be_loaded() {
        let profile = Profile::load_profile(Path::new("./test/files/profile.json"));
        assert_eq!(profile.name, "Demo profile");
        assert_eq!(profile.xrservice_path, PathBuf::from("/home/user/monado"));
        assert_eq!(
            profile.opencomposite_path,
            PathBuf::from("/home/user/opencomposite")
        );
        assert_eq!(profile.prefix, PathBuf::from("/home/user/envisionprefix"));
        assert_eq!(
            profile.features.libsurvive.path,
            Some(PathBuf::from("/home/user/libsurvive"))
        );
        assert_eq!(profile.features.basalt.path, None);
        assert!(profile.features.libsurvive.enabled);
        assert!(!profile.features.basalt.enabled);
        assert!(!profile.features.mercury_enabled);
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
            xrservice_path: PathBuf::from("/home/user/monado"),
            xrservice_type: XRServiceType::Monado,
            opencomposite_path: PathBuf::from("/home/user/opencomposite"),
            features: ProfileFeatures {
                libsurvive: ProfileFeature {
                    feature_type: ProfileFeatureType::Libsurvive,
                    enabled: true,
                    path: Some(PathBuf::from("/home/user/libsurvive")),
                    repo: None,
                    branch: None,
                },
                basalt: ProfileFeature::default_basalt(),
                openhmd: ProfileFeature::default_openhmd(),
                mercury_enabled: false,
            },
            environment: env,
            prefix: PathBuf::from("/home/user/envisionprefix"),
            editable: true,
            ..Default::default()
        };
        let fpath = PathBuf::from("./target/testout/testprofile.json");
        p.dump_profile(&fpath);
        let loaded = Profile::load_profile(&fpath);
        assert_eq!(loaded.name, "Demo profile");
        assert_eq!(
            loaded.features.libsurvive.path,
            Some(PathBuf::from("/home/user/libsurvive"))
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
