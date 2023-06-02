use crate::{
    constants::CMD_NAME,
    file_utils::{get_config_dir, get_writer},
};
use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufReader};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub selected_profile_name: String,
    pub debug_view_enabled: bool,
}

fn config_file_path() -> String {
    format!(
        "{config}/{name}.json",
        config = get_config_dir(),
        name = CMD_NAME
    )
}

fn default_config() -> Config {
    Config {
        // TODO: handle first start with no profile selected
        selected_profile_name: "Demo profile".to_string(),
        debug_view_enabled: false,
    }
}

fn get_config_from_path(path_s: String) -> Config {
    match File::open(path_s) {
        Ok(file) => {
            let reader = BufReader::new(file);
            match serde_json::from_reader(reader) {
                Ok(config) => config,
                Err(_) => default_config(),
            }
        }
        Err(_) => default_config(),
    }
}

pub fn get_config() -> Config {
    get_config_from_path(config_file_path())
}

fn save_config_to_path(config: &Config, path_s: &String) -> Result<(), serde_json::Error> {
    let writer = get_writer(path_s);
    serde_json::to_writer_pretty(writer, config)
}

pub fn save_config(config: &Config) -> Result<(), serde_json::Error> {
    save_config_to_path(config, &config_file_path())
}

#[cfg(test)]
mod tests {
    use super::get_config_from_path;

    #[test]
    fn will_load_default_if_config_does_not_exist() {
        assert_eq!(
            get_config_from_path("/non/existing/file.json".into()).debug_view_enabled,
            false
        )
    }
}
