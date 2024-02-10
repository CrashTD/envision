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
            (LinuxDistro::Fedora, "eigen3-devel".into()),
            (LinuxDistro::Alpine, "eigen-dev".into()),
            (LinuxDistro::Gentoo, "dev-cpp/eigen".into()),
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
            (LinuxDistro::Fedora, "cmake".into()),
            (LinuxDistro::Alpine, "cmake".into()),
            (LinuxDistro::Gentoo, "dev-build/cmake".into()),
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
            (LinuxDistro::Alpine, "git".into()),
            (LinuxDistro::Gentoo, "dev-vcs/git".into()),
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
            (LinuxDistro::Fedora, "ninja-build".into()),
            (LinuxDistro::Alpine, "ninja".into()),
            (LinuxDistro::Gentoo, "dev-build/ninja".into()),
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
            (LinuxDistro::Alpine, "glslang".into()),
            (LinuxDistro::Gentoo, "dev-util/glslang".into()),
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
            (LinuxDistro::Alpine, "gcc".into()),
            (LinuxDistro::Gentoo, "sys-devel/gcc".into()),
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
            (LinuxDistro::Alpine, "g++".into()),
            (LinuxDistro::Gentoo, "sys-devel/gcc".into()),
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
            (LinuxDistro::Alpine, "libdrm-dev".into()),
            (LinuxDistro::Gentoo, "x11-libs/libdrm".into()),
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
            (LinuxDistro::Alpine, "openxr-dev".into()),
            (LinuxDistro::Gentoo, "media-libs/openxr".into()),
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
            (LinuxDistro::Gentoo, "media-libs/vulkan-loader".into()),
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

pub fn dep_pkexec() -> Dependency {
    Dependency {
        name: "pkexec".into(),
        dep_type: DepType::Executable,
        filename: "pkexec".into(),
        packages: HashMap::from([
            (LinuxDistro::Arch, "polkit".into()),
            (LinuxDistro::Debian, "pkexec".into()),
            (LinuxDistro::Fedora, "polkit".into()),
            (LinuxDistro::Alpine, "polkit".into()),
            (LinuxDistro::Gentoo, "sys-auth/polkit".into()),
        ]),
    }
}

pub fn dep_opencv() -> Dependency {
    Dependency {
        name: "opencv".into(),
        dep_type: DepType::Include,
        filename: "opencv4/opencv2/core/hal/hal.hpp".into(),
        packages: HashMap::from([
            (LinuxDistro::Arch, "opencv".into()),
            (LinuxDistro::Debian, "libopencv-dev".into()),
            (LinuxDistro::Fedora, "opencv-devel".into()),
            (LinuxDistro::Gentoo, "media-libs/opencv".into()),
        ]),
    }
}
