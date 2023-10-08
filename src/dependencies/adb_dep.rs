use crate::{
    depcheck::{DepType, Dependency},
    linux_distro::LinuxDistro,
};
use std::collections::HashMap;

pub fn adb_dep() -> Dependency {
    Dependency {
        name: "adb".into(),
        dep_type: DepType::Executable,
        filename: "adb".into(),
        packages: HashMap::from([
            (LinuxDistro::Arch, "android-tools".into()),
            (LinuxDistro::Fedora, "android-tools".into()),
            (LinuxDistro::Debian, "adb".into()),
        ]),
    }
}
