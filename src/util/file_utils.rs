use crate::{async_process::async_process, depcheck::common::dep_getcap_setcap, profile::Profile};
use anyhow::bail;
use nix::{
    errno::Errno,
    sys::statvfs::{statvfs, FsFlags},
};
use std::{
    fs::{self, copy, create_dir_all, remove_dir_all, File, OpenOptions},
    io::{BufReader, BufWriter},
    os::unix::fs::PermissionsExt,
    path::Path,
};
use tracing::{debug, error};

pub fn get_writer(path: &Path) -> anyhow::Result<BufWriter<std::fs::File>> {
    if let Some(parent) = path.parent()
        && !parent.is_dir()
    {
        create_dir_all(parent)?;
    };
    if path.is_symlink() {
        bail!(
            "Provided path '{}' exists already, but it is a symlink and not a file",
            path.to_string_lossy()
        );
    }
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
            error!("Error opening {}: {}", path.to_string_lossy(), e);
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
                error!("Failed to deserialize {}: {}", path.to_string_lossy(), e);
                None
            }
            Ok(res) => Some(res),
        },
    }
}

pub fn set_file_readonly(path: &Path, readonly: bool) -> anyhow::Result<()> {
    if path.is_symlink() {
        bail!(
            "path {} is a symlink, trying to change its write permission will only change the original file",
            path.to_string_lossy()
        );
    }
    if !path.is_file() {
        debug!(
            "trying to set readonly on a file that does not exist: {}",
            path.to_string_lossy()
        );
        return Ok(());
    }
    let mut perms = fs::metadata(path)
        .expect("Could not get metadata for file")
        .permissions();
    perms.set_readonly(readonly);
    Ok(fs::set_permissions(path, perms)?)
}

pub fn setcap_executable() -> Option<String> {
    if dep_getcap_setcap().check() {
        Some("setcap".into())
    } else if Path::new("/sbin/setcap").try_exists().unwrap_or_default() {
        Some("/sbin/setcap".into())
    } else {
        None
    }
}

pub fn getcap_executable() -> Option<String> {
    if dep_getcap_setcap().check() {
        Some("getcap".into())
    } else if Path::new("/sbin/getcap").try_exists().unwrap_or_default() {
        Some("/sbin/getcap".into())
    } else {
        None
    }
}

pub fn setcap_cap_sys_nice_eip_cmd(profile: &Profile) -> Vec<String> {
    vec![
        setcap_executable().unwrap_or("setcap".into()),
        "CAP_SYS_NICE=eip".into(),
        profile
            .prefix
            .join(profile.xrservice_binary())
            .to_string_lossy()
            .to_string(),
    ]
}

pub async fn verify_cap_sys_nice_eip(profile: &Profile) -> bool {
    let xrservice_binary = profile.xrservice_binary().to_string_lossy().to_string();
    if let Some(getcap_exec) = getcap_executable() {
        match async_process(&getcap_exec, Some(&[&xrservice_binary]), None).await {
            Err(e) => {
                error!("failed to run `getcap {xrservice_binary}`: {e:?}");
                false
            }
            Ok(out) => {
                debug!("getcap {xrservice_binary} stdout: {}", out.stdout);
                debug!("getcap {xrservice_binary} stderr: {}", out.stderr);
                if out.exit_code != 0 {
                    error!(
                        "command `getcap {xrservice_binary}` failed with status code {}",
                        out.exit_code
                    );
                    false
                } else {
                    out.stdout.to_lowercase().contains("cap_sys_nice=eip")
                }
            }
        }
    } else {
        error!("getcap executable does not exist");
        false
    }
}

pub async fn setcap_cap_sys_nice_eip(profile: &Profile) -> anyhow::Result<()> {
    async_process("pkexec", Some(&setcap_cap_sys_nice_eip_cmd(profile)), None).await?;
    Ok(())
}

pub fn rm_rf(path: &Path) {
    if remove_dir_all(path).is_err() {
        error!("failed to remove path {}", path.to_string_lossy());
    }
}

pub fn copy_file(source: &Path, dest: &Path) {
    if let Some(parent) = dest.parent()
        && !parent.is_dir()
    {
        create_dir_all(parent)
            .unwrap_or_else(|_| panic!("Failed to create dir {}", parent.to_str().unwrap()));
    }
    if !dest.is_symlink() {
        set_file_readonly(dest, false)
            .unwrap_or_else(|_| panic!("Failed to set file {} as rw", dest.to_string_lossy()));
    }
    copy(source, dest).unwrap_or_else(|e| {
        panic!(
            "Failed to copy {} to {}: {e}",
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

pub fn mark_as_executable(path: &Path) -> anyhow::Result<()> {
    if !path.is_file() {
        bail!("Path '{}' is not a file", path.to_string_lossy())
    } else {
        let mut perms = fs::metadata(path)?.permissions();
        perms.set_mode(perms.mode() | 0o111);
        fs::set_permissions(path, perms)?;
        Ok(())
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
