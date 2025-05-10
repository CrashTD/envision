pub mod basalt_deps;
pub mod boost_deps;
pub mod common;
pub mod libsurvive_deps;
pub mod mercury_deps;
pub mod monado_deps;
pub mod openhmd_deps;
pub mod wivrn_deps;
pub mod xrizer_deps;

use crate::linux_distro::LinuxDistro;
use std::{collections::HashMap, env, fmt::Display, path::Path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DepType {
    SharedObject,
    Executable,
    Include,
    UdevRule,
    Share,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dependency {
    pub name: String,
    pub dep_type: DepType,
    pub filename: String,
    pub packages: HashMap<LinuxDistro, String>,
}

impl Ord for Dependency {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for Dependency {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Dependency {
    pub fn check(&self) -> bool {
        for dir in match self.dep_type {
            DepType::SharedObject => shared_obj_paths(),
            DepType::Executable => env::var("PATH")
                .expect("PATH environment variable not defined")
                .split(':')
                .map(str::to_string)
                .collect(),
            DepType::Include => include_paths(),
            DepType::UdevRule => udev_rules_paths(),
            DepType::Share => share_paths(),
        } {
            let path_s = &format!("{dir}/{fname}", dir = dir, fname = self.filename);
            let path = Path::new(&path_s);
            if path.is_file() {
                return true;
            }
        }
        false
    }

    pub fn check_many(deps: Vec<Self>) -> Vec<DependencyCheckResult> {
        deps.iter()
            .map(|dep| DependencyCheckResult {
                dependency: dep.clone(),
                found: dep.check(),
            })
            .collect()
    }

    pub fn package_name_for_distro(&self, distro: Option<&LinuxDistro>) -> String {
        if let Some(distro) = distro {
            if let Some(pname) = self
                .packages
                .iter()
                .find(|(k, _)| *k == distro)
                .map(|(_, v)| v)
            {
                return pname.clone();
            }
        }
        self.name.clone()
    }

    pub fn package_name(&self) -> String {
        self.package_name_for_distro(LinuxDistro::get().as_ref())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyCheckResult {
    pub dependency: Dependency,
    pub found: bool,
}

impl Display for DependencyCheckResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{name} {result}",
            name = self.dependency.name,
            result = match self.found {
                true => "Found",
                false => "Not found",
            }
        ))
    }
}

pub trait DepcheckResultGetMissing {
    fn filter_missing_deps(self) -> Vec<Dependency>;
}

impl DepcheckResultGetMissing for Vec<DependencyCheckResult> {
    fn filter_missing_deps(self) -> Vec<Dependency> {
        self.into_iter()
            .filter_map(|res| {
                if !res.found {
                    Some(res.dependency)
                } else {
                    None
                }
            })
            .collect()
    }
}

fn shared_obj_paths() -> Vec<String> {
    vec![
        "/lib".into(),
        "/usr/lib".into(),
        "/usr/lib64".into(),
        "/usr/local/lib".into(),
        "/usr/local/lib64".into(),
        "/usr/lib/x86_64-linux-gnu".into(),
        "/usr/lib/aarch64-linux-gnu".into(),
        "/lib/x86_64-linux-gnu".into(),
        "/lib/aarch64-linux-gnu".into(),
        "/app/lib".into(),
    ]
}

fn include_paths() -> Vec<String> {
    vec![
        "/usr/include".into(),
        "/usr/local/include".into(),
        // fedora puts avcodec here
        "/usr/include/ffmpeg".into(),
        "/usr/include/x86_64-linux-gnu".into(),
        "/usr/include/aarch64-linux-gnu".into(),
        "/usr/include/opencv4".into(),
        "/usr/local/include/opencv4".into(),
        "/usr/include/ffmpeg/libavcodec".into(),
        "/usr/include/ffmpeg/libavfilter".into(),
        "/usr/include/ffmpeg/libavformat".into(),
        "/usr/include/ffmpeg/libavutil".into(),
        "/usr/include/ffmpeg/libpostproc".into(),
        "/usr/include/ffmpeg/libswresample".into(),
        "/usr/include/ffmpeg/libswscale".into(),
    ]
}

fn udev_rules_paths() -> Vec<String> {
    vec!["/usr/lib/udev/rules.d".into()]
}

fn share_paths() -> Vec<String> {
    vec!["/usr/share".into()]
}

#[cfg(test)]
mod tests {
    use super::{DepType, Dependency};
    use std::collections::HashMap;

    #[test]
    fn can_find_libc() {
        let libc_dep = Dependency {
            name: "libc".into(),
            dep_type: DepType::SharedObject,
            filename: "libc.so".into(),
            packages: HashMap::new(),
        };

        assert!(libc_dep.check());
    }

    #[test]
    fn can_find_ls() {
        let ls_dep = Dependency {
            name: "ls".into(),
            dep_type: DepType::Executable,
            filename: ("ls".into()),
            packages: HashMap::new(),
        };

        assert!(ls_dep.check());
    }

    #[test]
    fn cannot_find_fake_lib() {
        let fake_dep = Dependency {
            name: "fakedep".into(),
            dep_type: DepType::SharedObject,
            filename: "fakedep.so".into(),
            packages: HashMap::new(),
        };

        assert!(!fake_dep.check());
    }
}
