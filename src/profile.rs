use serde::{Deserialize, Serialize};
use std::{
    fs::{create_dir_all, File},
    io::{BufReader, BufWriter},
    path::Path,
};

#[derive(Serialize, Deserialize)]
pub struct Profile {
    monado_path: String,
    openovr_path: String,
    libsurvive_path: Option<String>,
    basalt_path: Option<String>,
    mercury_path: Option<String>,
    libsurvive_enabled: bool,
    basalt_enabled: bool,
    mercury_enabled: bool,
}

pub fn load_profile(path: String) -> Result<Profile, serde_json::Error> {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    serde_json::from_reader(reader)
}

pub fn dump_profile(profile: Profile, path_s: String) -> () {
    let path = Path::new(&path_s);
    match path.parent() {
        Some(parent) => {
            if !parent.is_dir() {
                create_dir_all(parent).expect("Could not create dir")
            }
        }
        None => {}
    };
    let file = match path.is_file() {
        true => File::open(path).expect("Could not open file"),
        false => File::create(path).expect("Could not create file")
    };
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &profile).expect("Could not write profile")
}

#[cfg(test)]
mod tests {
    use super::{dump_profile, Profile, load_profile};

    #[test]
    fn profile_can_be_loaded() {
        let profile = load_profile(String::from("./test/files/profile.json"));
        match profile {
            Ok(profile) => {
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
            }
            Err(_) => assert!(false),
        }
    }

    #[test]
    fn profile_can_be_dumped() {
        let p = Profile {
            monado_path: String::from("/home/user/monado"),
            openovr_path: String::from("/home/user/openovr"),
            libsurvive_path: Some(String::from("/home/user/libsurvive")),
            basalt_path: None,
            mercury_path: None,
            libsurvive_enabled: true,
            basalt_enabled: false,
            mercury_enabled: false,
        };
        dump_profile(p, String::from("./target/testout/testprofile.json"))
    }
}
