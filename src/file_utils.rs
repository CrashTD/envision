use std::{
    fs::{create_dir_all, OpenOptions},
    io::BufWriter,
    path::Path, env,
};

use crate::constants::CMD_NAME;

pub fn get_writer(path_s: &String) -> BufWriter<std::fs::File> {
    let path = Path::new(path_s);
    match path.parent() {
        Some(parent) => {
            if !parent.is_dir() {
                create_dir_all(parent).expect("Could not create dir")
            }
        }
        None => {}
    };
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(path)
        .expect("Could not open file");
    BufWriter::new(file)
}

pub fn get_home_dir() -> String {
    env::var("HOME").expect("HOME env var not defined")
}

pub fn get_config_dir() -> String {
    match env::var("XDG_CONFIG_HOME") {
        Ok(conf_home) => format!(
            "{conf_home}/{name}",
            conf_home = conf_home,
            name = CMD_NAME
        ),
        Err(_) => format!(
            "{home}/.config/{name}",
            home = get_home_dir(),
            name = CMD_NAME
        ),
    }
}

pub fn get_data_dir() -> String {
    match env::var("XDG_DATA_HOME") {
        Ok(data_home) => format!(
            "{data_home}/{name}",
            data_home = data_home,
            name = CMD_NAME
        ),
        Err(_) => format!(
            "{home}/.local/share/{name}",
            home = get_home_dir(),
            name = CMD_NAME
        ),
    }
}
