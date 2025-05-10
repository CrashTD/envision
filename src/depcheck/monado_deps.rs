use super::{
    common::{
        dep_cmake, dep_eigen, dep_gcc, dep_git, dep_glslang_validator, dep_gpp, dep_libdrm,
        dep_libgl, dep_libudev, dep_libx11, dep_libxcb, dep_ninja, dep_openxr, dep_vulkan_headers,
        dep_vulkan_icd_loader,
    },
    DepType, DepcheckResultGetMissing, Dependency, DependencyCheckResult,
};
use crate::{depcheck::common::dep_libxrandr, linux_distro::LinuxDistro};
use std::collections::HashMap;

fn monado_deps() -> Vec<Dependency> {
    vec![
        dep_libdrm(),
        dep_openxr(),
        dep_vulkan_icd_loader(),
        dep_vulkan_headers(),
        dep_libxcb(),
        dep_libx11(),
        dep_libxrandr(),
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
        Dependency {
            name: "wayland-protocols".into(),
            dep_type: DepType::Share,
            filename: "wayland-protocols/staging/drm-lease/drm-lease-v1.xml".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "wayland-protocols".into()),
                (LinuxDistro::Debian, "wayland-protocols".into()),
                (LinuxDistro::Fedora, "wayland-protocols-devel".into()),
                (LinuxDistro::Gentoo, "dev-libs/wayland-protocols".into()),
                (LinuxDistro::Suse, "wayland-protocols-devel".into()),
            ]),
        },
        Dependency {
            name: "libbsd".into(),
            dep_type: DepType::SharedObject,
            filename: "libbsd.so".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "libbsd".into()),
                (LinuxDistro::Debian, "libbsd-dev".into()),
                (LinuxDistro::Fedora, "libbsd-devel".into()),
                (LinuxDistro::Gentoo, "dev-libs/libbsd".into()),
                (LinuxDistro::Suse, "libbsd-devel".into()),
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
                (LinuxDistro::Suse, "sdl2-compat-devel".into()),
            ]),
        },
        dep_libudev(),
        Dependency {
            name: "libusb".into(),
            dep_type: DepType::SharedObject,
            filename: "libusb-1.0.so".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "libusb".into()),
                (LinuxDistro::Debian, "libusb-1.0-0".into()),
                (LinuxDistro::Fedora, "libusb1".into()),
                (LinuxDistro::Gentoo, "dev-libs/libusb".into()),
                (LinuxDistro::Suse, "libusb-1_0-0".into()),
            ]),
        },
        Dependency {
            name: "libusb-dev".into(),
            dep_type: DepType::Include,
            filename: "libusb-1.0/libusb.h".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "libusb".into()),
                (LinuxDistro::Debian, "libusb-1.0-0-dev".into()),
                (LinuxDistro::Fedora, "libusb1-devel".into()),
                (LinuxDistro::Gentoo, "dev-libs/libusb".into()),
                (LinuxDistro::Suse, "libusb-1_0-devel".into()),
            ]),
        },
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
        dep_libgl(),
    ]
}

pub fn check_monado_deps() -> Vec<DependencyCheckResult> {
    Dependency::check_many(monado_deps())
}

pub fn get_missing_monado_deps() -> Vec<Dependency> {
    check_monado_deps().filter_missing_deps()
}
