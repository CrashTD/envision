use super::{DepType, DepcheckResultGetMissing, Dependency, DependencyCheckResult};
use crate::{depcheck::common::dep_glslc, linux_distro::LinuxDistro};
use std::collections::HashMap;

fn xrizer_deps() -> Vec<Dependency> {
    vec![
        dep_glslc(),
        Dependency {
            name: "libxcb-glx".into(),
            dep_type: DepType::Include,
            filename: "xcb/glx.h".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "libxcb".into()),
                (LinuxDistro::Debian, "libxcb-glx0-dev".into()),
                (LinuxDistro::Fedora, "libxcb-devel".into()),
                (LinuxDistro::Gentoo, "x11-libs/libxcb".into()),
                (LinuxDistro::Suse, "libxcb-devel".into()),
            ]),
        },
        Dependency {
            name: "libclang".into(),
            dep_type: DepType::SharedObject,
            filename: "libclang.so".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "clang".into()),
                (LinuxDistro::Debian, "libclang-19-dev".into()),
                (LinuxDistro::Fedora, "clang19-devel".into()),
                (LinuxDistro::Gentoo, "llvm-core/clang-runtime".into()),
                (LinuxDistro::Suse, "clang19-devel".into()),
            ]),
        },
    ]
}

pub fn check_xrizer_deps() -> Vec<DependencyCheckResult> {
    Dependency::check_many(xrizer_deps())
}

pub fn get_missing_xrizer_deps() -> Vec<Dependency> {
    check_xrizer_deps().filter_missing_deps()
}
