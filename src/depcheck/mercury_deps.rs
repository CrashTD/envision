use super::{common::dep_opencv, DepType, Dependency, DependencyCheckResult};
use crate::linux_distro::LinuxDistro;
use std::collections::HashMap;

fn mercury_deps() -> Vec<Dependency> {
    vec![
        dep_opencv(),
        Dependency {
            name: "jq".into(),
            dep_type: DepType::Executable,
            filename: "jq".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "jq".into()),
                (LinuxDistro::Debian, "jq".into()),
                (LinuxDistro::Fedora, "jq".into()),
                (LinuxDistro::Alpine, "jq".into()),
                (LinuxDistro::Gentoo, "app-misc/jq".into()),
                (LinuxDistro::Suse, "jq".into()),
            ]),
        },
        Dependency {
            name: "git-lfs".into(),
            dep_type: DepType::Executable,
            filename: "git-lfs".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "git-lfs".into()),
                (LinuxDistro::Debian, "git-lfs".into()),
                (LinuxDistro::Fedora, "git-lfs".into()),
                (LinuxDistro::Alpine, "git-lfs".into()),
                (LinuxDistro::Gentoo, "dev-vcs/git-lfs".into()),
                (LinuxDistro::Suse, "git-lfs".into()),
            ]),
        },
    ]
}

pub fn check_mercury_deps() -> Vec<DependencyCheckResult> {
    Dependency::check_many(mercury_deps())
}

pub fn get_missing_mercury_deps() -> Vec<Dependency> {
    check_mercury_deps()
        .iter()
        .filter(|res| !res.found)
        .map(|res| res.dependency.clone())
        .collect()
}
