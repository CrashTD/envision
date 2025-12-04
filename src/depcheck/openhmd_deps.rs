use super::{
    DepcheckResultGetMissing, Dependency, DependencyCheckResult,
    common::{dep_gcc, dep_git, dep_gpp, dep_ninja},
};
use crate::linux_distro::LinuxDistro;
use std::collections::HashMap;

fn openhmd_deps() -> Vec<Dependency> {
    vec![
        dep_gcc(),
        dep_gpp(),
        dep_ninja(),
        dep_git(),
        Dependency {
            name: "meson".into(),
            filename: "meson".into(),
            dep_type: crate::depcheck::DepType::Executable,
            packages: HashMap::from([
                (LinuxDistro::Arch, "meson".into()),
                (LinuxDistro::Debian, "meson".into()),
                (LinuxDistro::Fedora, "meson".into()),
                (LinuxDistro::Alpine, "meson".into()),
                (LinuxDistro::Suse, "meson".into()),
            ]),
        },
    ]
}

pub fn check_openhmd_deps() -> Vec<DependencyCheckResult> {
    Dependency::check_many(openhmd_deps())
}

pub fn get_missing_openhmd_deps() -> Vec<Dependency> {
    check_openhmd_deps().filter_missing_deps()
}
