use crate::{
    depcheck::{check_dependencies, DepType, Dependency, DependencyCheckResult},
    dependencies::common::{dep_eigen, dep_gpp},
    linux_distro::LinuxDistro,
};
use std::collections::HashMap;

fn basalt_deps() -> Vec<Dependency> {
    vec![
        dep_gpp(),
        Dependency {
            name: "boost".into(),
            dep_type: DepType::SharedObject,
            // just one of the many shared objects boost provides
            filename: "libboost_system.so".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "boost".into()),
                (LinuxDistro::Debian, "libboost-all-dev".into()),
            ]),
        },
        Dependency {
            name: "boost-dev".into(),
            dep_type: DepType::Include,
            // just one of the many headers boost provides
            filename: "boost/filesystem.hpp".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "boost".into()),
                (LinuxDistro::Debian, "libboost-all-dev".into()),
            ]),
        },
        Dependency {
            name: "bzip2".into(),
            dep_type: DepType::SharedObject,
            filename: "libbz2.so".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "bzip2".into()),
                (LinuxDistro::Debian, "libbz2-dev".into()),
            ]),
        },
        Dependency {
            name: "bzip2-dev".into(),
            dep_type: DepType::Include,
            filename: "bzlib.h".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "bzip2".into()),
                (LinuxDistro::Debian, "libbz2-dev".into()),
            ]),
        },
        dep_eigen(),
        Dependency {
            name: "fmt".into(),
            dep_type: DepType::SharedObject,
            filename: "libfmt.so".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "fmt".into()),
                (LinuxDistro::Debian, "libfmt-dev".into()),
            ]),
        },
        Dependency {
            name: "fmt-dev".into(),
            dep_type: DepType::Include,
            filename: "fmt/core.h".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "fmt".into()),
                (LinuxDistro::Debian, "libfmt-dev".into()),
            ]),
        },
        Dependency {
            name: "glew".into(),
            dep_type: DepType::SharedObject,
            filename: "libGLEW.so".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "glew".into()),
                (LinuxDistro::Debian, "libglew-dev".into()),
            ]),
        },
        Dependency {
            name: "glew-dev".into(),
            dep_type: DepType::Include,
            filename: "GL/glew.h".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "glew".into()),
                (LinuxDistro::Debian, "libglew-dev".into()),
            ]),
        },
        Dependency {
            name: "gtest".into(),
            dep_type: DepType::Include,
            filename: "gtest/gtest.h".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "gtest".into()),
                (LinuxDistro::Debian, "libgtest-dev".into()),
            ]),
        },
        Dependency {
            name: "opencv".into(),
            dep_type: DepType::Include,
            filename: "opencv4/opencv2/core/hal/hal.hpp".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "opencv".into()),
                (LinuxDistro::Debian, "libopencv-dev".into()),
            ]),
        },
        Dependency {
            name: "python3".into(),
            dep_type: DepType::Executable,
            filename: "python3".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "python".into()),
                (LinuxDistro::Debian, "python3".into()),
            ]),
        },
        Dependency {
            name: "bc".into(),
            dep_type: DepType::Executable,
            filename: "bc".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "bc".into()),
                (LinuxDistro::Debian, "bc".into()),
            ]),
        },
    ]
}

pub fn check_basalt_deps() -> Vec<DependencyCheckResult> {
    check_dependencies(basalt_deps())
}

pub fn get_missing_basalt_deps() -> Vec<Dependency> {
    check_basalt_deps()
        .iter()
        .filter(|res| !res.found)
        .map(|res| res.dependency.clone())
        .collect()
}
