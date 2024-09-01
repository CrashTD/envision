use crate::{async_process::async_process, profile::Profile};
use nix::{
    errno::Errno,
    sys::statvfs::{statvfs, FsFlags},
};
use std::{
    fs::{self, copy, create_dir_all, remove_dir_all, File, OpenOptions},
    io::{BufReader, BufWriter},
    path::Path,
};

pub fn get_writer(path: &Path) -> anyhow::Result<BufWriter<std::fs::File>> {
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

pub fn get_reader(path: &Path) -> Option<BufReader<File>> {
    if !(path.is_file() || path.is_symlink()) {
        return None;
    }
    match File::open(path) {
        Err(e) => {
            eprintln!("Error opening {}: {}", path.to_string_lossy(), e);
            None
        }
        Ok(fd) => Some(BufReader::new(fd)),
    }
}

pub fn deserialize_file<T: serde::de::DeserializeOwned>(path: &Path) -> Option<T> {
    match get_reader(path) {
        None => None,
        Some(reader) => match serde_json::from_reader(reader) {
            Err(e) => {
                eprintln!("Failed to deserialize {}: {}", path.to_string_lossy(), e);
                None
            }
            Ok(res) => Some(res),
        },
    }
}

pub fn set_file_readonly(path: &Path, readonly: bool) -> Result<(), std::io::Error> {
    if !path.is_file() {
        eprintln!("WARN: trying to set readonly on a file that does not exist");
        return Ok(());
    }
    let mut perms = fs::metadata(path)
        .expect("Could not get metadata for file")
        .permissions();
    perms.set_readonly(readonly);
    fs::set_permissions(path, perms)
}

pub fn setcap_cap_sys_nice_eip_cmd(profile: &Profile) -> Vec<String> {
    vec![
        "setcap".into(),
        "CAP_SYS_NICE=eip".into(),
        profile
            .prefix
            // not needed for wivrn, that's why monado is hardcoded
            .join("bin/monado-service")
            .to_string_lossy()
            .to_string(),
    ]
}

pub async fn setcap_cap_sys_nice_eip(profile: &Profile) {
    if let Err(e) = async_process("pkexec", Some(&setcap_cap_sys_nice_eip_cmd(profile)), None).await
    {
        eprintln!("Error: failed running setcap: {e}");
    }
}

pub fn rm_rf(path: &Path) {
    if remove_dir_all(path).is_err() {
        eprintln!("Failed to remove path {}", path.to_string_lossy());
    }
}

pub fn copy_file(source: &Path, dest: &Path) {
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

pub fn mount_has_nosuid(path: &Path) -> Result<bool, Errno> {
    match statvfs(path) {
        Ok(fstats) => Ok(fstats.flags().contains(FsFlags::ST_NOSUID)),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::mount_has_nosuid;
    use std::path::Path;

    #[test]
    fn can_get_nosuid() {
        mount_has_nosuid(Path::new("/tmp")).expect("Error running statvfs");
    }
}
