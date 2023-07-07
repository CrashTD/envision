use crate::{constants::CMD_NAME, runner::Runner};
use expect_dialog::ExpectDialog;
use std::{
    env,
    fs::{self, create_dir_all, File, OpenOptions},
    io::{BufReader, BufWriter},
    path::Path,
};

pub fn get_writer(path_s: &String) -> BufWriter<std::fs::File> {
    let path = Path::new(path_s);
    match path.parent() {
        Some(parent) => {
            if !parent.is_dir() {
                create_dir_all(parent).expect_dialog("Could not create dir")
            }
        }
        None => {}
    };
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .expect_dialog("Could not open file");
    BufWriter::new(file)
}

pub fn get_reader(path_s: &String) -> Option<BufReader<File>> {
    let path = Path::new(&path_s);
    if !(path.is_file() || path.is_symlink()) {
        return None;
    }
    match File::open(path) {
        Err(e) => {
            println!("Error opening {}: {}", path_s, e);
            return None;
        }
        Ok(fd) => Some(BufReader::new(fd)),
    }
}

pub fn deserialize_file<T: serde::de::DeserializeOwned>(path_s: &String) -> Option<T> {
    match get_reader(&path_s) {
        None => None,
        Some(reader) => match serde_json::from_reader(reader) {
            Err(e) => {
                println!("Failed to deserialize {}: {}", path_s, e);
                None
            }
            Ok(res) => Some(res),
        },
    }
}

pub fn get_home_dir() -> String {
    env::var("HOME").expect_dialog("HOME env var not defined")
}

pub fn get_xdg_config_dir() -> String {
    match env::var("XDG_CONFIG_HOME") {
        Ok(conf_home) => conf_home,
        Err(_) => format!("{home}/.config", home = get_home_dir()),
    }
}

pub fn get_xdg_data_dir() -> String {
    match env::var("XDG_DATA_HOME") {
        Ok(data_home) => data_home,
        Err(_) => format!("{home}/.local/share", home = get_home_dir()),
    }
}

pub fn get_xdg_cache_dir() -> String {
    match env::var("XDG_CACHE_HOME") {
        Ok(cache_home) => cache_home,
        Err(_) => format!("{home}/.cache", home = get_home_dir()),
    }
}

pub fn get_config_dir() -> String {
    format!(
        "{config}/{name}",
        config = get_xdg_config_dir(),
        name = CMD_NAME
    )
}

pub fn get_data_dir() -> String {
    format!("{data}/{name}", data = get_xdg_data_dir(), name = CMD_NAME)
}

pub fn get_cache_dir() -> String {
    format!(
        "{cache}/{name}",
        cache = get_xdg_cache_dir(),
        name = CMD_NAME
    )
}

pub fn set_file_radonly(path_s: &String, readonly: bool) {
    let path = Path::new(&path_s);
    if !path.is_file() {
        println!("WARN: trying to set readonly on a file that does not exist");
        return;
    }
    let mut perms = fs::metadata(path)
        .expect_dialog("Could not get metadata for file")
        .permissions();
    perms.set_readonly(readonly);
    fs::set_permissions(path, perms).expect_dialog("Could not set permissions for file")
}

pub fn setcap_cap_sys_nice_eip(file: String) {
    let mut runner = Runner::new(
        None,
        "pkexec".into(),
        vec!["setcap".into(), "CAP_SYS_NICE=eip".into(), file],
    );
    runner.start();
    runner.join();
}

pub fn get_exec_prefix() -> String {
    let p = Path::new("/proc/self/exe");
    if !p.is_symlink() {
        panic!("/proc/self/exe is not a symlink!");
    }
    p.read_link()
        .unwrap()
        .as_path()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}
