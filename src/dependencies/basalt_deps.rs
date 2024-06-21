use crate::{
    depcheck::{DepType, Dependency, DependencyCheckResult},
    dependencies::common::{dep_cmake, dep_eigen, dep_gpp, dep_ninja, dep_opencv},
    linux_distro::LinuxDistro,
};
use std::collections::HashMap;

fn basalt_deps() -> Vec<Dependency> {
    vec![
        dep_gpp(),
        dep_cmake(),
        dep_ninja(),
        Dependency {
            name: "libepoxy".into(),
            dep_type: DepType::Include,
            filename: "epoxy/egl.h".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "libepoxy".into()),
                (LinuxDistro::Debian, "libepoxy-dev".into()),
                (LinuxDistro::Fedora, "libepoxy-devel".into()),
                (LinuxDistro::Alpine, "libepoxy-dev".into()),
            ]),
        },
        Dependency {
            name: "catch2".into(),
            dep_type: DepType::Include,
            filename: "catch2/catch_all.hpp".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "catch2".into()),
                (LinuxDistro::Debian, "catch2".into()),
                (LinuxDistro::Fedora, "catch2-devel".into()),
                (LinuxDistro::Alpine, "catch2-3".into()),
            ]),
        },
        Dependency {
            name: "boost".into(),
            dep_type: DepType::SharedObject,
            // just one of the many shared objects boost provides
            filename: "libboost_system.so".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "boost".into()),
                (LinuxDistro::Debian, "libboost-all-dev".into()),
                (LinuxDistro::Fedora, "boost".into()),
                (LinuxDistro::Alpine, "boost".into()),
                (LinuxDistro::Gentoo, "dev-libs/boost".into()),
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
                (LinuxDistro::Fedora, "boost-devel".into()),
                (LinuxDistro::Alpine, "boost-dev".into()),
            ]),
        },
        Dependency {
            name: "bzip2".into(),
            dep_type: DepType::SharedObject,
            filename: "libbz2.so".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "bzip2".into()),
                (LinuxDistro::Debian, "libbz2-dev".into()),
                (LinuxDistro::Fedora, "bzip2-devel".into()),
                (LinuxDistro::Alpine, "bzip2".into()),
                (LinuxDistro::Gentoo, "app-arch/bzip2".into()),
            ]),
        },
        Dependency {
            name: "bzip2-dev".into(),
            dep_type: DepType::Include,
            filename: "bzlib.h".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "bzip2".into()),
                (LinuxDistro::Debian, "libbz2-dev".into()),
                (LinuxDistro::Fedora, "bzip2-devel".into()),
                (LinuxDistro::Alpine, "bzip2-dev".into()),
                (LinuxDistro::Gentoo, "app-arch/bzip2".into()),
            ]),
        },
        // Dependency {
        //     name: "cli11-dev".into(),
        //     dep_type: DepType::Include,
        //     filename: "CLI/App.hpp".into(),
        //     packages: HashMap::from([
        //         (LinuxDistro::Arch, "cli11".into()),
        //         (LinuxDistro::Debian, "libcli11-dev")
        //     ])
        // },
        dep_eigen(),
        Dependency {
            name: "fmt".into(),
            dep_type: DepType::SharedObject,
            filename: "libfmt.so".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "fmt".into()),
                (LinuxDistro::Debian, "libfmt-dev".into()),
                (LinuxDistro::Fedora, "fmt-devel".into()),
                (LinuxDistro::Alpine, "fmt".into()),
                (LinuxDistro::Gentoo, "dev-libs/libfmt".into()),
            ]),
        },
        Dependency {
            name: "fmt-dev".into(),
            dep_type: DepType::Include,
            filename: "fmt/core.h".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "fmt".into()),
                (LinuxDistro::Debian, "libfmt-dev".into()),
                (LinuxDistro::Fedora, "fmt-devel".into()),
                (LinuxDistro::Alpine, "fmt-dev".into()),
            ]),
        },
        Dependency {
            name: "glew".into(),
            dep_type: DepType::SharedObject,
            filename: "libGLEW.so".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "glew".into()),
                (LinuxDistro::Debian, "libglew-dev".into()),
                (LinuxDistro::Fedora, "glew-devel".into()),
                (LinuxDistro::Gentoo, "media-libs/glew".into()),
            ]),
        },
        Dependency {
            name: "glew-dev".into(),
            dep_type: DepType::Include,
            filename: "GL/glew.h".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "glew".into()),
                (LinuxDistro::Debian, "libglew-dev".into()),
                (LinuxDistro::Fedora, "glew-devel".into()),
            ]),
        },
        Dependency {
            name: "gtest".into(),
            dep_type: DepType::Include,
            filename: "gtest/gtest.h".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "gtest".into()),
                (LinuxDistro::Debian, "libgtest-dev".into()),
                (LinuxDistro::Fedora, "gtest-devel".into()),
                (LinuxDistro::Gentoo, "dev-cpp/gtest".into()),
            ]),
        },
        Dependency {
            name: "tbb".into(),
            dep_type: DepType::Include,
            filename: "tbb/tbb.h".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "onetbb".into()),
                (LinuxDistro::Debian, "libtbb-dev".into()),
                (LinuxDistro::Fedora, "tbb-devel".into()),
                (LinuxDistro::Gentoo, "dev-cpp/tbb".into()),
            ]),
        },
        dep_opencv(),
        Dependency {
            name: "python3".into(),
            dep_type: DepType::Executable,
            filename: "python3".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "python".into()),
                (LinuxDistro::Debian, "python3".into()),
                (LinuxDistro::Fedora, "python3".into()),
                (LinuxDistro::Alpine, "python3".into()),
                (LinuxDistro::Gentoo, "dev-lang/python".into()),
            ]),
        },
        Dependency {
            name: "bc".into(),
            dep_type: DepType::Executable,
            filename: "bc".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "bc".into()),
                (LinuxDistro::Debian, "bc".into()),
                (LinuxDistro::Fedora, "bc".into()),
                (LinuxDistro::Alpine, "bc".into()),
                (LinuxDistro::Gentoo, "sys-devel/bc".into()),
            ]),
        },
    ]
}

pub fn check_basalt_deps() -> Vec<DependencyCheckResult> {
    Dependency::check_many(basalt_deps())
}

pub fn get_missing_basalt_deps() -> Vec<Dependency> {
    check_basalt_deps()
        .iter()
        .filter(|res| !res.found)
        .map(|res| res.dependency.clone())
        .collect()
}
