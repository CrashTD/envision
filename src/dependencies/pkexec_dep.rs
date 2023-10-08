use crate::{
    depcheck::{DepType, Dependency},
    linux_distro::LinuxDistro,
};
use std::collections::HashMap;

pub fn pkexec_dep() -> Dependency {
    Dependency {
        name: "pkexec".into(),
        dep_type: DepType::Executable,
        filename: "pkexec".into(),
        packages: HashMap::from([
            (LinuxDistro::Arch, "polkit".into()),
            (LinuxDistro::Debian, "pkexec".into()),
            (LinuxDistro::Fedora, "polkit".into()),
        ]),
    }
}
