use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{create_dir_all, File, OpenOptions},
    io::{BufReader, BufWriter},
    path::Path, fmt::Display,
};

use crate::file_utils::get_writer;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub monado_path: String,
    pub openovr_path: String,
    pub libsurvive_path: Option<String>,
    pub basalt_path: Option<String>,
    pub mercury_path: Option<String>,
    pub libsurvive_enabled: bool,
    pub basalt_enabled: bool,
    pub mercury_enabled: bool,
    pub environment: HashMap<String, String>,
}

impl Display for Profile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name.to_string())
    }
}

pub fn load_profile(path: &String) -> Result<Profile, serde_json::Error> {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    serde_json::from_reader(reader)
}

pub fn dump_profile(profile: Profile, path_s: &String) -> () {
    let writer = get_writer(path_s);
    serde_json::to_writer_pretty(writer, &profile).expect("Could not write profile")
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{dump_profile, load_profile, Profile};

    #[test]
    fn profile_can_be_loaded() {
        let profile = load_profile(&"./test/files/profile.json".to_string());
        match profile {
            Ok(profile) => {
                assert_eq!(profile.name, "Demo profile");
                assert_eq!(profile.monado_path, "/home/user/monado");
                assert_eq!(profile.openovr_path, "/home/user/openovr");
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
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn profile_can_be_dumped() {
        let mut env = HashMap::new();
        env.insert(
            "XRT_COMPOSITOR_SCALE_PERCENTAGE".into(),
            "140".into(),
        );
        env.insert("XRT_COMPOSITOR_COMPUTE".into(), "1".into());
        let p = Profile {
            name: "Demo profile".into(),
            monado_path: String::from("/home/user/monado"),
            openovr_path: String::from("/home/user/openovr"),
            libsurvive_path: Some(String::from("/home/user/libsurvive")),
            basalt_path: None,
            mercury_path: None,
            libsurvive_enabled: true,
            basalt_enabled: false,
            mercury_enabled: false,
            environment: env,
        };
        let fpath = String::from("./target/testout/testprofile.json");
        dump_profile(p, &fpath);
        let loaded = load_profile(&fpath).unwrap();
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
