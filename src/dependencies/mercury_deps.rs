use crate::{
    depcheck::{DepType, Dependency, DependencyCheckResult},
    linux_distro::LinuxDistro,
};
use std::collections::HashMap;

fn mercury_deps() -> Vec<Dependency> {
    vec![
        Dependency {
            name: "opencv".into(),
            dep_type: DepType::SharedObject,
            filename: "libopencv_core.so".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "opencv".into()),
                (LinuxDistro::Debian, "libopencv-dev".into()),
            ]),
        },
        Dependency {
            name: "opencv-dev".into(),
            dep_type: DepType::Include,
            filename: "opencv4/opencv2/core/base.hpp".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "opencv".into()),
                (LinuxDistro::Debian, "libopencv-dev".into()),
            ]),
        },
        Dependency {
            name: "git-lfs".into(),
            dep_type: DepType::Executable,
            filename: "git-lfs".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "git-lfs".into()),
                (LinuxDistro::Debian, "git-lfs".into()),
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
