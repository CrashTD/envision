use crate::{
    depcheck::{check_dependencies, DepType, Dependency, DependencyCheckResult},
    dependencies::common::{dep_cmake, dep_eigen, dep_git, dep_glslang_validator, dep_ninja},
    linux_distro::LinuxDistro,
};
use std::collections::HashMap;

fn monado_deps() -> Vec<Dependency> {
    vec![
        Dependency {
            name: "libdrm".into(),
            dep_type: DepType::SharedObject,
            filename: "libdrm.so".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "libdrm".into()),
                (LinuxDistro::Debian, "libdrm-dev".into()),
                (LinuxDistro::Fedora, "libdrm".into()),
            ]),
        },
        Dependency {
            name: "openxr".into(),
            dep_type: DepType::SharedObject,
            filename: "libopenxr_loader.so".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "openxr".into()),
                (LinuxDistro::Debian, "libopenxr-dev".into()),
                (LinuxDistro::Fedora, "openxr-devel".into()),
            ]),
        },
        Dependency {
            name: "vulkan-icd-loader".into(),
            dep_type: DepType::SharedObject,
            filename: "libvulkan.so".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "vulkan-icd-loader".into()),
                (LinuxDistro::Debian, "libvulkan-dev".into()),
                (LinuxDistro::Fedora, "vulkan-loader-devel".into()),
            ]),
        },
        Dependency {
            name: "vulkan-headers".into(),
            dep_type: DepType::Include,
            filename: "vulkan/vulkan.h".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "vulkan-headers".into()),
                (LinuxDistro::Debian, "libvulkan-dev".into()),
                (LinuxDistro::Fedora, "vulkan-devel".into()),
            ]),
        },
        Dependency {
            name: "wayland".into(),
            dep_type: DepType::SharedObject,
            filename: "libwayland-client.so".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "wayland".into()),
                (LinuxDistro::Debian, "libwayland-dev".into()),
            ]),
        },
        dep_cmake(),
        dep_eigen(),
        dep_git(),
        dep_ninja(),
        Dependency {
            name: "glslc".into(),
            dep_type: DepType::Executable,
            filename: "glslc".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "shaderc".into()),
                (LinuxDistro::Debian, "glslc".into()),
                (LinuxDistro::Fedora, "glslc".into()),
            ]),
        },
        dep_glslang_validator(),
    ]
}

pub fn check_monado_deps() -> Vec<DependencyCheckResult> {
    check_dependencies(monado_deps())
}

pub fn get_missing_monado_deps() -> Vec<Dependency> {
    check_monado_deps()
        .iter()
        .filter(|res| !res.found)
        .map(|res| res.dependency.clone())
        .collect()
}
