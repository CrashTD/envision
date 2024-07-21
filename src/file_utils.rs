use crate::{cmd_runner::CmdRunner, runner::Runner};
use nix::{
    errno::Errno,
    sys::statvfs::{statvfs, FsFlags},
};
use std::{
    fs::{self, copy, create_dir_all, remove_dir_all, File, OpenOptions},
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

pub fn get_writer(path: &PathBuf) -> anyhow::Result<BufWriter<std::fs::File>> {
    if let Some(parent) = path.parent() {
        if !parent.is_dir() {
            create_dir_all(parent)?;
        }
    };
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    Ok(BufWriter::new(file))
}

pub fn get_reader(path: &PathBuf) -> Option<BufReader<File>> {
    if !(path.is_file() || path.is_symlink()) {
        return None;
    }
    match File::open(path) {
        Err(e) => {
            println!("Error opening {}: {}", path.to_string_lossy(), e);
            None
        }
        Ok(fd) => Some(BufReader::new(fd)),
    }
}

pub fn deserialize_file<T: serde::de::DeserializeOwned>(path: &PathBuf) -> Option<T> {
    match get_reader(path) {
        None => None,
        Some(reader) => match serde_json::from_reader(reader) {
            Err(e) => {
                println!("Failed to deserialize {}: {}", path.to_string_lossy(), e);
                None
            }
            Ok(res) => Some(res),
        },
    }
}

pub fn set_file_readonly(path: &PathBuf, readonly: bool) -> Result<(), std::io::Error> {
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

pub fn setcap_cap_sys_nice_eip(file: &PathBuf) {
    let mut runner = CmdRunner::new(
        None,
        "pkexec".into(),
        vec![
            "setcap".into(),
            "CAP_SYS_NICE=eip".into(),
            file.to_string_lossy().to_string(),
        ],
    );
    runner.start();
    runner.join();
}

pub fn rm_rf(path: &PathBuf) {
    if remove_dir_all(path).is_err() {
        println!("Failed to remove path {}", path.to_string_lossy());
    }
}

pub fn copy_file(source: &PathBuf, dest: &PathBuf) {
    if let Some(parent) = dest.parent() {
        if !parent.is_dir() {
            create_dir_all(parent)
                .unwrap_or_else(|_| panic!("Failed to create dir {}", parent.to_str().unwrap()));
        }
    }
    set_file_readonly(dest, false)
        .unwrap_or_else(|_| panic!("Failed to set file {} as rw", dest.to_string_lossy()));
    copy(source, dest).unwrap_or_else(|_| {
        panic!(
            "Failed to copy {} to {}",
            source.to_string_lossy(),
            dest.to_string_lossy()
        )
    });
}

pub fn mount_has_nosuid(path: &PathBuf) -> Result<bool, Errno> {
    match statvfs(path) {
        Ok(fstats) => Ok(fstats.flags().contains(FsFlags::ST_NOSUID)),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::file_utils::mount_has_nosuid;

    #[test]
    fn can_get_nosuid() {
        mount_has_nosuid(&PathBuf::from("/tmp")).expect("Error running statvfs");
    }
}
