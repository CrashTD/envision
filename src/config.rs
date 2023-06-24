use crate::{
    constants::CMD_NAME,
    file_utils::{get_config_dir, get_writer}, profile::Profile,
};
use expect_dialog::ExpectDialog;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufReader};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub selected_profile_name: String,
    pub debug_view_enabled: bool,
    pub user_profiles: Vec<Profile>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            // TODO: handle first start with no profile selected
            selected_profile_name: "".to_string(),
            debug_view_enabled: false,
            user_profiles: vec![],
        }
    }
}

impl Config {
    pub fn get_selected_profile(&self, profiles: &Vec<Profile>) -> Profile {
        match profiles.iter().find(|p| {p.name == self.selected_profile_name}) {
            Some(p) => p.clone(),
            None => profiles.get(0).expect_dialog("No profiles found").clone(),
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
        let writer = get_writer(path_s);
        serde_json::to_writer_pretty(writer, self)
    }

    pub fn save(&self) {
        self.save_to_path(&Self::config_file_path()).expect_dialog("Failed to save config");
    }

    pub fn get_config() -> Self {
        Self::from_path(Self::config_file_path())
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
