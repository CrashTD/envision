use crate::{
    constants::CMD_NAME,
    device_prober::PhysicalXRDevice,
    paths::get_config_dir,
    profile::Profile,
    profiles::{
        lighthouse::lighthouse_profile, openhmd::openhmd_profile, simulated::simulated_profile,
        survive::survive_profile, wivrn::wivrn_profile, wmr::wmr_profile,
    },
    ui::plugins::Plugin,
    util::file_utils::get_writer,
};
use serde::{Deserialize, Serialize, de::Error};
use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginConfig {
    pub plugin: Plugin,
    pub enabled: bool,
}

impl From<&Plugin> for PluginConfig {
    fn from(p: &Plugin) -> Self {
        Self {
            plugin: p.clone(),
            enabled: true,
        }
    }
}

impl From<&PluginConfig> for Plugin {
    fn from(cp: &PluginConfig) -> Self {
        cp.plugin.clone()
    }
}

const DEFAULT_WIN_SIZE: [i32; 2] = [360, 400];

const fn default_win_size() -> [i32; 2] {
    DEFAULT_WIN_SIZE
}

fn default_theme_name() -> String {
    "Follow system".into()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub selected_profile_uuid: String,
    pub debug_view_enabled: bool,
    pub user_profiles: Vec<Profile>,
    #[serde(default = "default_win_size")]
    pub win_size: [i32; 2],
    #[serde(default)]
    pub plugins: HashMap<String, PluginConfig>,
    #[serde(default = "default_theme_name")]
    pub theme_name: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            // TODO: using an empty string here is ugly
            selected_profile_uuid: "".to_string(),
            debug_view_enabled: false,
            user_profiles: Vec::default(),
            win_size: DEFAULT_WIN_SIZE,
            plugins: HashMap::default(),
            theme_name: default_theme_name(),
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
            wivrn_profile(),
            wmr_profile(),
            openhmd_profile(),
            simulated_profile(),
        ];
        profiles.extend(self.user_profiles.clone());
        profiles.sort_unstable_by(|a, b| a.name.cmp(&b.name));
        profiles
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::config::Config;

    #[test]
    fn will_load_default_if_config_does_not_exist() {
        assert!(!Config::from_path(Path::new("/non/existing/file.json")).debug_view_enabled,)
    }
}
