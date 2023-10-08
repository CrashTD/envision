use crate::{
    depcheck::{DepType, Dependency},
    linux_distro::LinuxDistro,
};
use std::collections::HashMap;

pub fn dep_eigen() -> Dependency {
    Dependency {
        name: "eigen".into(),
        dep_type: DepType::Include,
        filename: "eigen3/Eigen/src/Core/EigenBase.h".into(),
        packages: HashMap::from([
            (LinuxDistro::Arch, "eigen".into()),
            (LinuxDistro::Debian, "libeigen3-dev".into()),
        ]),
    }
}

pub fn dep_cmake() -> Dependency {
    Dependency {
        name: "cmake".into(),
        dep_type: DepType::Executable,
        filename: "cmake".into(),
        packages: HashMap::from([
            (LinuxDistro::Arch, "cmake".into()),
            (LinuxDistro::Debian, "cmake".into()),
        ]),
    }
}

pub fn dep_git() -> Dependency {
    Dependency {
        name: "git".into(),
        dep_type: DepType::Executable,
        filename: "git".into(),
        packages: HashMap::from([
            (LinuxDistro::Arch, "git".into()),
            (LinuxDistro::Debian, "git".into()),
            (LinuxDistro::Fedora, "git".into()),
        ]),
    }
}

pub fn dep_ninja() -> Dependency {
    Dependency {
        name: "ninja".into(),
        dep_type: DepType::Executable,
        filename: "ninja".into(),
        packages: HashMap::from([
            (LinuxDistro::Arch, "ninja".into()),
            (LinuxDistro::Debian, "ninja-build".into()),
        ]),
    }
}

pub fn dep_glslang_validator() -> Dependency {
    Dependency {
        name: "glslang".into(),
        dep_type: DepType::Executable,
        filename: "glslangValidator".into(),
        packages: HashMap::from([
            (LinuxDistro::Arch, "glslang".into()),
            (LinuxDistro::Debian, "glslang-tools".into()),
            (LinuxDistro::Fedora, "glslang-devel".into()),
        ]),
    }
}
