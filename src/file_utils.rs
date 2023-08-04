use crate::{constants::CMD_NAME, runner::Runner};
use std::{
    env,
    fs::{self, copy, create_dir_all, remove_dir_all, File, OpenOptions},
    io::{BufReader, BufWriter},
    path::Path,
};

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
        .truncate(true)
        .open(path)
        .expect("Could not open file");
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

pub fn set_file_readonly(path_s: &String, readonly: bool) -> Result<(), std::io::Error> {
    let path = Path::new(&path_s);
    if !path.is_file() {
        println!("WARN: trying to set readonly on a file that does not exist");
        return Ok(());
    }
    let mut perms = fs::metadata(path)
        .expect("Could not get metadata for file")
        .permissions();
    perms.set_readonly(readonly);
    fs::set_permissions(path, perms)
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

pub fn rm_rf(path_s: &String) {
    match remove_dir_all(path_s) {
        Err(_) => println!("Failed to remove path {}", path_s),
        Ok(_) => {}
    }
}

pub fn copy_file(source_s: &String, dest_s: &String) {
    let source = Path::new(source_s);
    let dest = Path::new(dest_s);
    let parent = dest.parent();
    if parent.is_some() {
        if !parent.unwrap().is_dir() {
            create_dir_all(parent.unwrap()).expect(
                format!("Failed to create dir {}", parent.unwrap().to_str().unwrap()).as_str(),
            );
        }
    }
    set_file_readonly(dest_s, false);
    copy(source, dest).expect(format!("Failed to copy {} to {}", source_s, dest_s).as_str());
}
