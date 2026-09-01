use crate::{
    constants::CMD_NAME,
    device_prober::PhysicalXRDevice,
    paths::get_config_dir,
    plugins::XRPlugin,
    profile::{OvrCompatibilityModuleType, Profile, XRServiceType},
    profiles::{
        lighthouse::lighthouse_profile, openhmd::openhmd_profile, simulated::simulated_profile,
        survive::survive_profile, wmr::wmr_profile,
    },
    util::file_utils::get_writer,
};
use serde::{Deserialize, Deserializer, Serialize, de::Error};
use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

const DEFAULT_WIN_SIZE: [i32; 2] = [360, 400];

const fn default_win_size() -> [i32; 2] {
    DEFAULT_WIN_SIZE
}

fn default_theme_name() -> String {
    "Follow system".into()
}

fn xrservicetype_deserializer_with_fallback<'de, D>(
    deserializer: D,
) -> Result<XRServiceType, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer).unwrap_or_default();
    Ok(serde_json::from_str(s).unwrap_or_default())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalPreferences {
    #[serde(default)]
    #[serde(deserialize_with = "xrservicetype_deserializer_with_fallback")]
    pub xrservice_type: XRServiceType,
    pub ovr_comp: OvrCompatibilityModuleType,

    /// XRT_COMPOSITOR_SCALE_PERCENTAGE
    pub scale_percentage: u16,
    /// XRT_COMPOSITOR_COMPUTE (1, 0)
    pub use_compositor_gpu_compute: bool,
    /// U_PACING_APP_USE_MIN_FRAME_PERIOD (1, 0)
    pub use_min_frame_period: bool,
    /// XRT_DEBUG_GUI (1, 0) + XRT_CURATED_GUI (1, 0)
    pub monado_ui: bool,
    /// U_PACING_COMP_TIME_FRACTION_PERCENT=95 + XRT_COMPOSITOR_USE_PRESENT_WAIT=1
    #[serde(default)]
    pub nvidia_mitigation: bool,
    #[serde(default)]
    pub env_vars: HashMap<String, String>,
}

impl Default for GlobalPreferences {
    fn default() -> Self {
        Self {
            xrservice_type: XRServiceType::default(),
            ovr_comp: OvrCompatibilityModuleType::default(),

            scale_percentage: 140,
            use_compositor_gpu_compute: true,
            use_min_frame_period: false,
            monado_ui: false,
            env_vars: HashMap::default(),
            nvidia_mitigation: false,
        }
    }
}

impl GlobalPreferences {
    pub fn get_env_vars(&self) -> HashMap<String, String> {
        let mut res = HashMap::from([
            (
                "XRT_COMPOSITOR_SCALE_PERCENTAGE".into(),
                self.scale_percentage.to_string(),
            ),
            (
                "XRT_COMPOSITOR_COMPUTE".into(),
                (if self.use_compositor_gpu_compute {
                    1
                } else {
                    0
                })
                .to_string(),
            ),
            (
                "U_PACING_APP_USE_MIN_FRAME_PERIOD".into(),
                (if self.use_min_frame_period { 1 } else { 0 }).to_string(),
            ),
            (
                "XRT_DEBUG_GUI".into(),
                (if self.monado_ui { 1 } else { 0 }).to_string(),
            ),
            (
                "XRT_CURATED_GUI".into(),
                (if self.monado_ui { 1 } else { 0 }).to_string(),
            ),
            ("ORT_DISABLE_TELEMETRY".into(), 1.to_string()),
        ]);
        if self.nvidia_mitigation {
            res.insert("U_PACING_COMP_TIME_FRACTION_PERCENT".into(), "95".into());
            res.insert("XRT_COMPOSITOR_USE_PRESENT_WAIT".into(), "1".into());
        }
        res.extend(self.env_vars.clone());
        res
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub selected_profile_uuid: String,
    pub debug_view_enabled: bool,
    pub user_profiles: Vec<Profile>,
    #[serde(default = "default_win_size")]
    pub win_size: [i32; 2],
    #[serde(default = "default_theme_name")]
    pub theme_name: String,
    #[serde(default)]
    pub global_preferences: GlobalPreferences,
    #[serde(default)]
    pub profiles_enabled: bool,

    /// each element is a path to a .desktop file with `X-XR-Plugin=true`
    #[serde(default)]
    pub xr_plugins: HashSet<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // TODO: using an empty string here is ugly
            selected_profile_uuid: "".to_string(),
            debug_view_enabled: false,
            user_profiles: Vec::default(),
            win_size: DEFAULT_WIN_SIZE,
            theme_name: default_theme_name(),
            global_preferences: GlobalPreferences::default(),
            profiles_enabled: false,
            xr_plugins: HashSet::default(),
        }
    }
}

impl Config {
    pub fn get_selected_profile(&self, profiles: &[Profile]) -> Profile {
        let def = || profiles.first().expect("No profiles found").clone();

        if let Some(p) = profiles
            .iter()
            .find(|p| p.uuid == self.selected_profile_uuid)
        {
            p.clone()
        } else {
            PhysicalXRDevice::from_usb()
                .first()
                .and_then(|xrd| xrd.get_default_profile())
                .unwrap_or_else(def)
        }
    }

    pub fn config_file_path() -> PathBuf {
        get_config_dir().join(format!("{CMD_NAME}.json"))
    }

    fn from_path(path: &Path) -> Self {
        let mut this: Self = File::open(path)
            .ok()
            .and_then(|file| serde_json::from_reader(BufReader::new(file)).ok())
            .unwrap_or_default();

        let mut needs_save = false;

        // remap legacy opencomposite data to new ovr_comp
        #[allow(deprecated)]
        for prof in this.user_profiles.iter_mut() {
            if prof
                .ovr_comp
                .path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                == "__envision__fallbackovrcomp"
            {
                prof.ovr_comp.path = prof.opencomposite_path.clone();
                needs_save = true;
            }
            if prof.opencomposite_repo.is_some() && prof.ovr_comp.repo.is_none() {
                prof.ovr_comp.repo = prof.opencomposite_repo.take();
                needs_save = true;
            }
            if prof.opencomposite_branch.is_some() && prof.ovr_comp.branch.is_none() {
                prof.ovr_comp.branch = prof.opencomposite_branch.take();
                needs_save = true;
            }
        }

        if needs_save {
            this.save_to_path(path).expect("Failed to save config");
        }

        this
    }

    fn save_to_path(&self, path: &Path) -> Result<(), serde_json::Error> {
        let writer = get_writer(path).map_err(serde_json::Error::custom)?;
        serde_json::to_writer_pretty(writer, self)
    }

    pub fn save(&self) {
        self.save_to_path(&Self::config_file_path())
            .expect("Failed to save config");
    }

    pub fn get_config() -> Self {
        Self::from_path(&Self::config_file_path())
    }

    pub fn set_profiles(&mut self, profiles: &[Profile]) {
        self.user_profiles = profiles.iter().filter(|p| p.editable).cloned().collect();
    }

    pub fn profiles(&self) -> Vec<Profile> {
        let mut profiles = vec![
            lighthouse_profile(),
            survive_profile(),
            wmr_profile(),
            openhmd_profile(),
            simulated_profile(),
        ];
        profiles.extend(self.user_profiles.clone());
        profiles.sort_unstable_by(|a, b| a.name.cmp(&b.name));
        profiles
    }

    /// returns enabled and valid XR plugins
    pub fn get_xr_plugins(&self) -> Vec<XRPlugin> {
        self.xr_plugins
            .iter()
            .filter_map(|path| XRPlugin::from_desktop_file(&PathBuf::from(path)))
            .collect()
    }

    /// gets rid of invalid plugins and saves to disk
    pub fn prune_xr_plugins(&mut self) {
        let n_plugins: HashSet<String> =
            self.get_xr_plugins().into_iter().map(|p| p.path).collect();
        if n_plugins != self.xr_plugins {
            self.xr_plugins = n_plugins;
            self.save();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{config::Config, profile::XRServiceType};
    use std::path::Path;

    #[test]
    fn will_load_default_if_config_does_not_exist() {
        assert!(!Config::from_path(Path::new("/non/existing/file.json")).debug_view_enabled,)
    }

    #[test]
    fn will_not_wipe_config_if_nvidia_mitigation_is_missing() {
        assert!(
            Config::from_path(Path::new(
                "test/files/config_without_nvidia_mitigation.json"
            ))
            .global_preferences
            .monado_ui,
        )
    }

    #[test]
    fn will_not_wipe_config_if_xr_service_type_is_set_to_wivrn() {
        // config should fall back to monado
        let config = Config::from_path(Path::new("test/files/config_with_wivrn.json"));
        assert!(config.global_preferences.monado_ui,);
        assert_eq!(
            config.global_preferences.xrservice_type,
            XRServiceType::Monado
        );
    }
}
