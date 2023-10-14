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

pub fn dep_gcc() -> Dependency {
    Dependency {
        name: "gcc".into(),
        dep_type: DepType::Executable,
        filename: "gcc".into(),
        packages: HashMap::from([
            (LinuxDistro::Arch, "gcc".into()),
            (LinuxDistro::Debian, "gcc".into()),
            (LinuxDistro::Fedora, "gcc".into()),
        ]),
    }
}

pub fn dep_gpp() -> Dependency {
    Dependency {
        name: "g++".into(),
        dep_type: DepType::Executable,
        filename: "g++".into(),
        packages: HashMap::from([
            (LinuxDistro::Arch, "gcc".into()),
            (LinuxDistro::Debian, "g++".into()),
            (LinuxDistro::Fedora, "g++".into()),
        ]),
    }
}

pub fn dep_libdrm() -> Dependency {
    Dependency {
        name: "libdrm".into(),
        dep_type: DepType::SharedObject,
        filename: "libdrm.so".into(),
        packages: HashMap::from([
            (LinuxDistro::Arch, "libdrm".into()),
            (LinuxDistro::Debian, "libdrm-dev".into()),
            (LinuxDistro::Fedora, "libdrm".into()),
        ]),
    }
}

pub fn dep_openxr() -> Dependency {
    Dependency {
        name: "openxr".into(),
        dep_type: DepType::SharedObject,
        filename: "libopenxr_loader.so".into(),
        packages: HashMap::from([
            (LinuxDistro::Arch, "openxr".into()),
            (LinuxDistro::Debian, "libopenxr-dev".into()),
            (LinuxDistro::Fedora, "openxr-devel".into()),
        ]),
    }
}

pub fn dep_vulkan_icd_loader() -> Dependency {
    Dependency {
        name: "vulkan-icd-loader".into(),
        dep_type: DepType::SharedObject,
        filename: "libvulkan.so".into(),
        packages: HashMap::from([
            (LinuxDistro::Arch, "vulkan-icd-loader".into()),
            (LinuxDistro::Debian, "libvulkan-dev".into()),
            (LinuxDistro::Fedora, "vulkan-loader-devel".into()),
        ]),
    }
}

pub fn dep_vulkan_headers() -> Dependency {
    Dependency {
        name: "vulkan-headers".into(),
        dep_type: DepType::Include,
        filename: "vulkan/vulkan.h".into(),
        packages: HashMap::from([
            (LinuxDistro::Arch, "vulkan-headers".into()),
            (LinuxDistro::Debian, "libvulkan-dev".into()),
            (LinuxDistro::Fedora, "vulkan-devel".into()),
        ]),
    }
}
