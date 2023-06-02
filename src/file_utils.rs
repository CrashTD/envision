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

pub fn get_config_dir() -> String {
    match env::var("XDG_CONFIG_HOME") {
        Ok(conf_home) => format!(
            "{chome}/{name}",
            chome = conf_home,
            name = CMD_NAME
        ),
        Err(_) => format!(
            "{home}/.config/{name}",
            home = env::var("HOME").expect("HOME env var not defined"),
            name = CMD_NAME
        ),
    }
}
