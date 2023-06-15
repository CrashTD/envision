use std::{
    env,
    fs::{self, create_dir_all, OpenOptions},
    io::BufWriter,
    path::Path,
};

use expect_dialog::ExpectDialog;

use crate::constants::CMD_NAME;

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

pub fn get_home_dir() -> String {
    env::var("HOME").expect_dialog("HOME env var not defined")
}

pub fn get_config_dir() -> String {
    match env::var("XDG_CONFIG_HOME") {
        Ok(conf_home) => format!("{conf_home}/{name}", conf_home = conf_home, name = CMD_NAME),
        Err(_) => format!(
            "{home}/.config/{name}",
            home = get_home_dir(),
            name = CMD_NAME
        ),
    }
}

pub fn get_data_dir() -> String {
    match env::var("XDG_DATA_HOME") {
        Ok(data_home) => format!("{data_home}/{name}", data_home = data_home, name = CMD_NAME),
        Err(_) => format!(
            "{home}/.local/share/{name}",
            home = get_home_dir(),
            name = CMD_NAME
        ),
    }
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
