use super::{
    common::{
        dep_cmake, dep_eigen, dep_gcc, dep_git, dep_glslang_validator, dep_gpp, dep_libdrm,
        dep_libglvnd, dep_libudev, dep_libx11, dep_libxcb, dep_ninja, dep_openxr,
        dep_vulkan_headers, dep_vulkan_icd_loader,
    },
    DepType, Dependency, DependencyCheckResult,
};
use crate::linux_distro::LinuxDistro;
use std::collections::HashMap;

fn monado_deps() -> Vec<Dependency> {
    vec![
        dep_libdrm(),
        dep_openxr(),
        dep_vulkan_icd_loader(),
        dep_vulkan_headers(),
        dep_libxcb(),
        dep_libx11(),
        Dependency {
            name: "wayland".into(),
            dep_type: DepType::SharedObject,
            filename: "libwayland-client.so".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "wayland".into()),
                (LinuxDistro::Debian, "libwayland-dev".into()),
                (LinuxDistro::Fedora, "wayland-devel".into()),
                (LinuxDistro::Gentoo, "dev-libs/wayland".into()),
                (LinuxDistro::Suse, "wayland-devel".into()),
            ]),
        },
        dep_cmake(),
        dep_eigen(),
        dep_git(),
        dep_ninja(),
        dep_gcc(),
        dep_gpp(),
        Dependency {
            name: "glslc".into(),
            dep_type: DepType::Executable,
            filename: "glslc".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "shaderc".into()),
                (LinuxDistro::Debian, "glslc".into()),
                (LinuxDistro::Fedora, "glslc".into()),
                (LinuxDistro::Alpine, "shaderc".into()),
                (LinuxDistro::Gentoo, "media-libs/shaderc".into()),
                (LinuxDistro::Suse, "shaderc".into()),
            ]),
        },
        dep_glslang_validator(),
        Dependency {
            name: "sdl2".into(),
            dep_type: DepType::SharedObject,
            filename: "libSDL2.so".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "sdl2".into()),
                (LinuxDistro::Debian, "libsdl2-dev".into()),
                (LinuxDistro::Fedora, "SDL2-devel".into()),
                (LinuxDistro::Gentoo, "media-libs/libsdl2".into()),
                (LinuxDistro::Suse, "SDL2-devel".into()),
            ]),
        },
        dep_libudev(),
        Dependency {
            name: "mesa-common-dev".into(),
            dep_type: DepType::Include,
            filename: "GL/internal/dri_interface.h".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "mesa".into()),
                (LinuxDistro::Debian, "mesa-common-dev".into()),
                (LinuxDistro::Fedora, "mesa-libGL-devel".into()),
                (LinuxDistro::Suse, "Mesa-dri-devel".into()),
            ]),
        },
        dep_libglvnd(),
    ]
}

pub fn check_monado_deps() -> Vec<DependencyCheckResult> {
    Dependency::check_many(monado_deps())
}

pub fn get_missing_monado_deps() -> Vec<Dependency> {
    check_monado_deps()
        .iter()
        .filter(|res| !res.found)
        .map(|res| res.dependency.clone())
        .collect()
}
