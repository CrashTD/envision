use crate::{
    depcheck::{
        basalt_deps::get_missing_basalt_deps, libsurvive_deps::get_missing_libsurvive_deps,
        mercury_deps::get_missing_mercury_deps, monado_deps::get_missing_monado_deps,
        openhmd_deps::get_missing_openhmd_deps, wivrn_deps::get_missing_wivrn_deps,
        xrizer_deps::get_missing_xrizer_deps, Dependency,
    },
    file_builders::active_runtime_json::ActiveRuntime,
    paths::{get_data_dir, BWRAP_SYSTEM_PREFIX, SYSTEM_PREFIX},
    util::file_utils::{deserialize_file, get_writer},
    xdg::XDG,
};
use nix::NixPath;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fmt::Display,
    fs::{remove_dir_all, File},
    io::BufReader,
    path::{Path, PathBuf},
    slice::Iter,
    str::FromStr,
};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum XRServiceType {
    Monado,
    Wivrn,
}

impl XRServiceType {
    pub fn iter() -> Iter<'static, Self> {
        [Self::Monado, Self::Wivrn].iter()
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
            Self::Wivrn => "wivrn/libmonado_wivrn.so",
        }
    }

    pub fn openxr_json_rel_path(&self) -> &'static str {
        match self {
            Self::Monado => "share/openxr/1/openxr_monado.json",
            Self::Wivrn => "share/openxr/1/openxr_wivrn.json",
        }
    }

    pub fn ipc_file_path(&self) -> PathBuf {
        XDG.get_runtime_directory()
            .expect("XDG runtime directory is not available")
            .join(match self {
                XRServiceType::Monado => "monado_comp_ipc",
                XRServiceType::Wivrn => "wivrn/comp_ipc",
            })
    }
}

impl From<&str> for XRServiceType {
    fn from(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "monado" => Self::Monado,
            "wivrn" => Self::Wivrn,
            _ => Self::Monado,
        }
    }
}

impl From<u32> for XRServiceType {
    fn from(i: u32) -> Self {
        match i {
            0 => Self::Monado,
            1 => Self::Wivrn,
            _ => panic!("XRServiceType index out of bounds"),
        }
    }
}

impl From<&XRServiceType> for u32 {
    fn from(value: &XRServiceType) -> Self {
        match value {
            XRServiceType::Monado => 0,
            XRServiceType::Wivrn => 1,
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
    pub fn iter() -> Iter<'static, ProfileFeatureType> {
        [Self::Libsurvive, Self::Basalt].iter()
    }
}

impl From<&str> for ProfileFeatureType {
    fn from(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "libsurvive" => Self::Libsurvive,
            "basalt" => Self::Basalt,
            "openhmd" => Self::OpenHmd,
            _ => panic!("Unknown profile feature type"),
        }
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

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LighthouseDriver {
    #[default]
    Vive,
    Survive,
    SteamVR,
}

impl From<&str> for LighthouseDriver {
    fn from(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "vive" => Self::Vive,
            "survive" => Self::Survive,
            "libsurvive" => Self::Survive,
            "steam" => Self::SteamVR,
            "steamvr" => Self::SteamVR,
            _ => Self::Vive,
        }
    }
}

impl From<u32> for LighthouseDriver {
    fn from(i: u32) -> Self {
        match i {
            0 => Self::Vive,
            1 => Self::Survive,
            2 => Self::SteamVR,
            _ => panic!("LighthouseDriver index out of bounds"),
        }
    }
}

impl From<&LighthouseDriver> for u32 {
    fn from(value: &LighthouseDriver) -> Self {
        match value {
            LighthouseDriver::Vive => 0,
            LighthouseDriver::Survive => 1,
            LighthouseDriver::SteamVR => 2,
        }
    }
}

impl LighthouseDriver {
    pub fn iter() -> Iter<'static, Self> {
        [Self::Vive, Self::Survive, Self::SteamVR].iter()
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum OvrCompatibilityModuleType {
    #[default]
    Opencomposite,
    Xrizer,
    Vapor,
}

impl Display for OvrCompatibilityModuleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Opencomposite => "OpenComposite",
            Self::Xrizer => "xrizer",
            Self::Vapor => "VapoR",
        })
    }
}

impl OvrCompatibilityModuleType {
    pub fn iter() -> Iter<'static, Self> {
        [Self::Opencomposite, Self::Xrizer, Self::Vapor].iter()
    }

    pub fn get_missing_deps(&self) -> Vec<Dependency> {
        match self {
            OvrCompatibilityModuleType::Xrizer => get_missing_xrizer_deps(),
            OvrCompatibilityModuleType::Opencomposite | OvrCompatibilityModuleType::Vapor => {
                Vec::default()
            }
        }
    }
}

impl FromStr for OvrCompatibilityModuleType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().trim() {
            "opencomposite" => Ok(Self::Opencomposite),
            "xrizer" => Ok(Self::Xrizer),
            "vapor" => Ok(Self::Vapor),
            _ => Err(format!("no match for ovr compatibility module `{s}`")),
        }
    }
}

impl From<u32> for OvrCompatibilityModuleType {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Opencomposite,
            1 => Self::Xrizer,
            2 => Self::Vapor,
            _ => panic!("OvrCompatibilityModuleType index out of bounds"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileOvrCompatibilityModule {
    pub mod_type: OvrCompatibilityModuleType,
    pub repo: Option<String>,
    pub branch: Option<String>,
    pub path: PathBuf,
}

impl ProfileOvrCompatibilityModule {
    pub fn default_for_uuid(uuid: &str) -> Self {
        let mod_type = OvrCompatibilityModuleType::default();
        Self {
            mod_type,
            repo: None,
            branch: None,
            path: get_data_dir().join(uuid).join(mod_type.to_string()),
        }
    }

    /// get the directory corresponding to the openvr runtime.
    /// this should correspond to the build output directory
    pub fn runtime_dir(&self) -> PathBuf {
        match self.mod_type {
            OvrCompatibilityModuleType::Opencomposite | OvrCompatibilityModuleType::Vapor => {
                self.path.join("build")
            }
            OvrCompatibilityModuleType::Xrizer => self.path.join("target/release"),
        }
    }
}

impl Default for ProfileOvrCompatibilityModule {
    fn default() -> Self {
        let mod_type = OvrCompatibilityModuleType::default();
        Self {
            mod_type,
            repo: None,
            branch: None,
            path: get_data_dir().join("__envision__fallbackovrcomp"),
        }
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
    #[deprecated]
    #[serde(default)]
    pub opencomposite_path: PathBuf,
    #[deprecated]
    pub opencomposite_repo: Option<String>,
    #[deprecated]
    pub opencomposite_branch: Option<String>,
    #[serde(default)]
    pub ovr_comp: ProfileOvrCompatibilityModule,
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
    #[serde(default)]
    pub skip_dependency_check: bool,
}

impl Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name.to_string())
    }
}

impl Default for Profile {
    #[allow(deprecated)]
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
            prefix: Self::default_prefix_path(&uuid),
            can_be_built: true,
            pull_on_build: true,
            opencomposite_path: profile_dir.join("opencomposite"),
            opencomposite_repo: None,
            opencomposite_branch: None,
            ovr_comp: ProfileOvrCompatibilityModule::default_for_uuid(&uuid),
            editable: true,
            lighthouse_driver: LighthouseDriver::default(),
            xrservice_launch_options: String::default(),
            uuid,
            skip_dependency_check: false,
        }
    }
}

impl Profile {
    fn default_prefix_path(uuid: &str) -> PathBuf {
        get_data_dir().join("prefixes").join(uuid)
    }

    /// deletes files and folders associated to this profile (mostly repo clones)
    pub fn delete_files(&self) -> Vec<std::io::Result<()>> {
        [
            Some(&self.xrservice_path),
            Some(&self.ovr_comp.path),
            self.features.libsurvive.path.as_ref(),
            self.features.basalt.path.as_ref(),
            self.features.openhmd.path.as_ref(),
        ]
        .iter()
        .map(|dir| match dir {
            Some(dir) => {
                if dir.try_exists().unwrap_or_default() {
                    remove_dir_all(dir)
                } else {
                    Ok(())
                }
            }
            None => Ok(()),
        })
        .collect()
    }

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
            //     "VR_OVERRIDE={}",
            //     self.ovr_comp.runtime_dir(),
            // ),
            self.xr_runtime_json_env_var(),
            format!(
                "PRESSURE_VESSEL_FILESYSTEMS_RW=\"{path}\"",
                path = self.xrservice_type.ipc_file_path().to_string_lossy(),
            ),
        ]
    }

    pub fn get_env_vars(&self) -> Vec<String> {
        if self.can_be_built {
            return vec![
                "PRESSURE_VESSEL_FILESYSTEMS_RW=\"$XDG_RUNTIME_DIR/wivrn_comp_ipc:$XDG_RUNTIME_DIR/wivrn/comp_ipc:$XDG_RUNTIME_DIR/monado_comp_ipc\"".into(),
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
        let uuid = Self::new_uuid();
        let profile_dir = get_data_dir().join(&uuid);
        #[allow(deprecated)]
        let mut dup = Self {
            name: format!("Duplicate of {}", self.name),
            xrservice_type: self.xrservice_type.clone(),
            xrservice_repo: self.xrservice_repo.clone(),
            xrservice_branch: self.xrservice_branch.clone(),
            xrservice_cmake_flags: self.xrservice_cmake_flags.clone(),
            xrservice_path: profile_dir.join("xrservice"),
            features: ProfileFeatures {
                libsurvive: ProfileFeature {
                    feature_type: ProfileFeatureType::Libsurvive,
                    enabled: self.features.libsurvive.enabled,
                    repo: self.features.libsurvive.repo.clone(),
                    branch: self.features.libsurvive.branch.clone(),
                    path: Some(profile_dir.join("libsurvive")),
                },
                basalt: ProfileFeature {
                    feature_type: ProfileFeatureType::Basalt,
                    enabled: self.features.basalt.enabled,
                    repo: self.features.basalt.repo.clone(),
                    branch: self.features.basalt.branch.clone(),
                    path: Some(profile_dir.join("basalt")),
                },
                openhmd: ProfileFeature {
                    feature_type: ProfileFeatureType::OpenHmd,
                    enabled: self.features.openhmd.enabled,
                    repo: self.features.openhmd.repo.clone(),
                    branch: self.features.openhmd.branch.clone(),
                    path: Some(profile_dir.join("openhmd")),
                },
                mercury_enabled: self.features.mercury_enabled,
            },
            environment: self.environment.clone(),
            pull_on_build: self.pull_on_build,
            lighthouse_driver: self.lighthouse_driver,
            opencomposite_repo: self.opencomposite_repo.clone(),
            opencomposite_branch: self.opencomposite_branch.clone(),
            opencomposite_path: profile_dir.join("opencomposite"),
            skip_dependency_check: self.skip_dependency_check,
            xrservice_launch_options: self.xrservice_launch_options.clone(),
            prefix: Self::default_prefix_path(&uuid),
            ovr_comp: ProfileOvrCompatibilityModule {
                mod_type: self.ovr_comp.mod_type,
                repo: self.ovr_comp.repo.clone(),
                branch: self.ovr_comp.branch.clone(),
                path: profile_dir.join(self.ovr_comp.mod_type.to_string()),
            },
            can_be_built: self.can_be_built,
            editable: true,
            uuid,
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
    pub fn find_so<P: AsRef<Path>>(&self, rel_path: P) -> Option<PathBuf> {
        ["lib", "lib64"]
            .into_iter()
            .map(|lib| self.prefix.join(lib).join(rel_path.as_ref()))
            .find(|path| path.is_file())
    }

    /// absolute path to the libmonado shared object
    pub fn libmonado_so(&self) -> Option<PathBuf> {
        // try by reading the openxr json file
        self.openxr_config()
            .and_then(|conf| conf.runtime.libmonado_path)
            .and_then(|libmonado_path| self.find_so(&libmonado_path))
            .or_else(||
                // try with the hardcoded paths
                self.find_so(self.xrservice_type.libmonado_path()))
    }

    fn openxr_config(&self) -> Option<ActiveRuntime> {
        deserialize_file(&self.openxr_json_path())
    }

    /// absolute path to the libopenxr shared object
    pub fn libopenxr_so(&self) -> Option<PathBuf> {
        // try by reading the openxr json file
        self.openxr_config()
            .map(|conf| conf.runtime.library_path)
            .and_then(|libmonado_path| self.find_so(&libmonado_path))
            .or_else(||
                // try with the hardcoded paths
                self.find_so(self.xrservice_type.libopenxr_path()))
    }

    pub fn missing_dependencies(&self) -> Vec<Dependency> {
        let mut missing_deps = Vec::new();
        if self.can_be_built {
            missing_deps.extend(match self.xrservice_type {
                XRServiceType::Monado => get_missing_monado_deps(),
                XRServiceType::Wivrn => get_missing_wivrn_deps(),
            });
            if self.features.libsurvive.enabled {
                missing_deps.extend(get_missing_libsurvive_deps());
            }
            if self.features.openhmd.enabled {
                missing_deps.extend(get_missing_openhmd_deps());
            }
            if self.features.basalt.enabled {
                missing_deps.extend(get_missing_basalt_deps());
            }
            if self.features.mercury_enabled {
                missing_deps.extend(get_missing_mercury_deps());
            }
            missing_deps.extend(self.ovr_comp.mod_type.get_missing_deps());
        }
        missing_deps.sort_unstable();
        missing_deps.dedup(); // dedup only works if sorted, hence the above
        missing_deps
    }

    /// the file that will become active_runtime.json, as installed in the
    /// prefix
    pub fn openxr_json_path(&self) -> PathBuf {
        self.prefix.join(self.xrservice_type.openxr_json_rel_path())
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

    use crate::profile::{
        OvrCompatibilityModuleType, ProfileFeature, ProfileFeatureType, ProfileFeatures,
        ProfileOvrCompatibilityModule, XRServiceType,
    };

    use super::Profile;

    #[test]
    fn profile_can_be_loaded() {
        let profile = Profile::load_profile(Path::new("./test/files/profile.json"));
        assert_eq!(profile.name, "Demo profile");
        assert_eq!(profile.xrservice_path, PathBuf::from("/home/user/monado"));
        assert_eq!(
            profile.ovr_comp.path,
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
            ovr_comp: ProfileOvrCompatibilityModule {
                path: PathBuf::from("/home/user/opencomposite"),
                repo: None,
                branch: None,
                mod_type: OvrCompatibilityModuleType::default(),
            },
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
