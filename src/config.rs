use crate::{
    constants::CMD_NAME,
    device_prober::get_xr_usb_devices,
    file_utils::get_writer,
    paths::get_config_dir,
    profile::Profile,
    profiles::{
        lighthouse::lighthouse_profile, openhmd::openhmd_profile, simulated::simulated_profile,
        survive::survive_profile, wivrn::wivrn_profile, wmr::wmr_profile,
    },
};
use serde::{ser::Error, Deserialize, Serialize};
use std::{fs::File, io::BufReader};

fn default_win_size() -> [i32; 2] {
    [360, 400]
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub selected_profile_uuid: String,
    pub debug_view_enabled: bool,
    pub user_profiles: Vec<Profile>,
    #[serde(default = "default_win_size")]
    pub win_size: [i32; 2],
}

impl Default for Config {
    fn default() -> Self {
        Config {
            // TODO: handle first start with no profile selected
            selected_profile_uuid: "".to_string(),
            debug_view_enabled: false,
            user_profiles: vec![],
            win_size: default_win_size(),
        }
    }
}

impl Config {
    pub fn get_selected_profile(&self, profiles: &[Profile]) -> Profile {
        let def = || profiles.get(0).expect("No profiles found").clone();

        match profiles
            .iter()
            .find(|p| p.uuid == self.selected_profile_uuid)
        {
            Some(p) => p.clone(),
            None => match get_xr_usb_devices().get(0) {
                Some(dev) => match dev.get_default_profile() {
                    Some(p) => p,
                    None => def(),
                },
                None => def(),
            },
        }
    }

    pub fn config_file_path() -> String {
        format!(
            "{config}/{name}.json",
            config = get_config_dir(),
            name = CMD_NAME
        )
    }

    fn from_path(path_s: String) -> Self {
        match File::open(path_s) {
            Ok(file) => {
                let reader = BufReader::new(file);
                match serde_json::from_reader(reader) {
                    Ok(config) => config,
                    Err(_) => Self::default(),
                }
            }
            Err(_) => Self::default(),
        }
    }

    fn save_to_path(&self, path_s: &String) -> Result<(), serde_json::Error> {
        let writer = get_writer(path_s).map_err(serde_json::Error::custom)?;
        serde_json::to_writer_pretty(writer, self)
    }

    pub fn save(&self) {
        self.save_to_path(&Self::config_file_path())
            .expect("Failed to save config");
    }

    pub fn get_config() -> Self {
        Self::from_path(Self::config_file_path())
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
    use crate::config::Config;

    #[test]
    fn will_load_default_if_config_does_not_exist() {
        assert_eq!(
            Config::from_path("/non/existing/file.json".into()).debug_view_enabled,
            false
        )
    }
}
