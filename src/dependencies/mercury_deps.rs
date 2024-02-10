use crate::{
    depcheck::{DepType, Dependency, DependencyCheckResult},
    dependencies::common::dep_opencv,
    linux_distro::LinuxDistro,
};
use std::collections::HashMap;

fn mercury_deps() -> Vec<Dependency> {
    vec![
        dep_opencv(),
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
