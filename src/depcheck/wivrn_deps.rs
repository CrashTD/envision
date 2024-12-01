use super::{
    common::{
        dep_cmake, dep_eigen, dep_gcc, dep_git, dep_glslang_validator, dep_gpp, dep_libdrm,
        dep_libudev, dep_libx11, dep_libxcb, dep_ninja, dep_openxr, dep_vulkan_headers,
        dep_vulkan_icd_loader,
    },
    DepType, Dependency, DependencyCheckResult,
};
use crate::{
    depcheck::common::{dep_libgl, dep_libxrandr},
    linux_distro::LinuxDistro,
};
use std::collections::HashMap;

fn wivrn_deps() -> Vec<Dependency> {
    vec![
        dep_cmake(),
        dep_ninja(),
        dep_git(),
        dep_gcc(),
        dep_gpp(),
        dep_libdrm(),
        dep_openxr(),
        dep_vulkan_icd_loader(),
        dep_vulkan_headers(),
        dep_libxcb(),
        dep_libx11(),
        dep_libxrandr(),
        dep_libgl(),
        Dependency {
            name: "patch".into(),
            dep_type: DepType::Executable,
            filename: "patch".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "patch".into()),
                (LinuxDistro::Debian, "patch".into()),
                (LinuxDistro::Fedora, "patch".into()),
                (LinuxDistro::Alpine, "patch".into()),
                (LinuxDistro::Gentoo, "sys-devel/patch".into()),
                (LinuxDistro::Suse, "patch".into()),
            ]),
        },
        Dependency {
            name: "x264-dev".into(),
            dep_type: DepType::Include,
            filename: "x264.h".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "x264".into()),
                (LinuxDistro::Debian, "libx264-dev".into()),
                (LinuxDistro::Fedora, "x264-devel".into()),
                (LinuxDistro::Gentoo, "media-libs/x264".into()),
                (LinuxDistro::Suse, "libx264-devel".into()),
            ]),
        },
        Dependency {
            name: "avahi".into(),
            dep_type: DepType::Include,
            filename: "avahi-client/client.h".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "avahi".into()),
                (LinuxDistro::Debian, "libavahi-client-dev".into()),
                (LinuxDistro::Fedora, "avahi-devel".into()),
                (LinuxDistro::Gentoo, "net-dns/avahi".into()),
                (LinuxDistro::Suse, "libavahi-devel".into()),
            ]),
        },
        Dependency {
            name: "avahi-glib-dev".into(),
            dep_type: DepType::Include,
            filename: "avahi-glib/glib-malloc.h".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "avahi".into()),
                (LinuxDistro::Debian, "libavahi-glib-dev".into()),
                (LinuxDistro::Fedora, "avahi-glib-devel".into()),
                // TODO
                // (LinuxDistro::Gentoo, "".into()),
                (LinuxDistro::Suse, "libavahi-glib-devel".into()),
            ]),
        },
        Dependency {
            name: "libpulse-dev".into(),
            dep_type: DepType::Include,
            filename: "pulse/context.h".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "libpulse".into()),
                (LinuxDistro::Debian, "libpulse-dev".into()),
                (LinuxDistro::Fedora, "pulseaudio-libs-devel".into()),
                (LinuxDistro::Gentoo, "media-libs/libpulse".into()),
                (LinuxDistro::Suse, "libpulse-devel".into()),
            ]),
        },
        dep_eigen(),
        Dependency {
            name: "nlohmann-json".into(),
            dep_type: DepType::Include,
            filename: "nlohmann/json.hpp".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "nlohmann-json".into()),
                (LinuxDistro::Debian, "nlohmann-json3-dev".into()),
                (LinuxDistro::Fedora, "json-devel".into()),
                (LinuxDistro::Gentoo, "dev-cpp/nlohmann_json".into()),
                (LinuxDistro::Suse, "nlohmann_json-devel".into()),
            ]),
        },
        Dependency {
            name: "libavcodec-dev".into(),
            dep_type: DepType::Include,
            filename: "libavcodec/avcodec.h".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "ffmpeg".into()),
                (LinuxDistro::Debian, "libavcodec-dev".into()),
                (LinuxDistro::Fedora, "ffmpeg-devel".into()),
                (LinuxDistro::Gentoo, "media-video/ffmpeg".into()),
                (LinuxDistro::Suse, "ffmpeg-6-libavcodec-devel".into()),
            ]),
        },
        Dependency {
            name: "libavfilter-dev".into(),
            dep_type: DepType::Include,
            filename: "libavfilter/avfilter.h".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "ffmpeg".into()),
                (LinuxDistro::Debian, "libavfilter-dev".into()),
                (LinuxDistro::Fedora, "ffmpeg-devel".into()),
                (LinuxDistro::Gentoo, "media-video/ffmpeg".into()),
                (LinuxDistro::Suse, "ffmpeg-6-libavfilter-devel".into()),
            ]),
        },
        Dependency {
            name: "libswscale-dev".into(),
            dep_type: DepType::Include,
            filename: "libswscale/swscale.h".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "ffmpeg".into()),
                (LinuxDistro::Debian, "libswscale-dev".into()),
                (LinuxDistro::Fedora, "ffmpeg-devel".into()),
                (LinuxDistro::Gentoo, "media-video/ffmpeg".into()),
                (LinuxDistro::Suse, "ffmpeg-6-libswscale-devel".into()),
            ]),
        },
        Dependency {
            name: "libavutil-dev".into(),
            dep_type: DepType::Include,
            filename: "libavutil/hwcontext_vaapi.h".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "ffmpeg".into()),
                (LinuxDistro::Debian, "libavutil-dev".into()),
                (LinuxDistro::Fedora, "ffmpeg-devel".into()),
                (LinuxDistro::Suse, "ffmpeg-6-libavutil-devel".into()),
            ]),
        },
        dep_glslang_validator(),
        dep_libudev(),
        Dependency {
            name: "gstreamer".into(),
            dep_type: DepType::SharedObject,
            filename: "libgstreamer-1.0.so".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "gstreamer".into()),
                (LinuxDistro::Debian, "libgstreamer1.0-dev".into()),
                (LinuxDistro::Fedora, "gstreamer1-devel".into()),
                (LinuxDistro::Gentoo, "media-libs/gstreamer".into()),
                (LinuxDistro::Suse, "gstreamer-devel".into()),
            ]),
        },
        Dependency {
            name: "gst-plugins-base-libs".into(),
            dep_type: DepType::SharedObject,
            filename: "pkgconfig/gstreamer-app-1.0.pc".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "gst-plugins-base-libs".into()),
                (LinuxDistro::Debian, "libgstreamer1.0-dev".into()),
                (LinuxDistro::Fedora, "gstreamer1-plugins-base-devel".into()),
                (LinuxDistro::Gentoo, "media-libs/gst-plugins-base".into()),
                (LinuxDistro::Suse, "gstreamer-plugins-base-devel".into()),
            ]),
        },
        Dependency {
            name: "systemd-dev".into(),
            dep_type: DepType::Include,
            filename: "systemd/sd-daemon.h".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "systemd-libs".into()),
                (LinuxDistro::Debian, "libsystemd-dev".into()),
                (LinuxDistro::Fedora, "systemd-devel".into()),
                (LinuxDistro::Gentoo, "sys-apps/systemd".into()),
                (LinuxDistro::Suse, "systemd-devel".into()),
            ]),
        },
        Dependency {
            name: "libva-dev".into(),
            dep_type: DepType::Include,
            filename: "va/va.h".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "libva".into()),
                (LinuxDistro::Debian, "libva-dev".into()),
                (LinuxDistro::Fedora, "libva-devel".into()),
                (LinuxDistro::Gentoo, "media-libs/libva".into()),
                (LinuxDistro::Suse, "libva-devel".into()),
            ]),
        },
        Dependency {
            name: "cli11".into(),
            dep_type: DepType::Include,
            filename: "CLI/CLI.hpp".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "cli11".into()),
                (LinuxDistro::Debian, "libcli11-dev".into()),
                (LinuxDistro::Fedora, "cli11-devel".into()),
                (LinuxDistro::Gentoo, "dev-cpp/cli11".into()),
                (LinuxDistro::Suse, "cli11-devel".into()),
            ]),
        },
        Dependency {
            name: "glib2-dev".into(),
            dep_type: DepType::Include,
            filename: "glib-2.0/glib.h".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "glib2".into()),
                (LinuxDistro::Alpine, "glib-dev".into()),
                (LinuxDistro::Debian, "libglib2.0-dev".into()),
                (LinuxDistro::Fedora, "glib2-devel".into()),
                (LinuxDistro::Suse, "glib2-devel".into()),
            ]),
        },
        Dependency {
            name: "gdbus-codegen".into(),
            dep_type: DepType::Executable,
            filename: "gdbus-codegen".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "glib2-devel".into()),
                (LinuxDistro::Alpine, "glib-dev".into()),
                (LinuxDistro::Debian, "libglib2.0-dev".into()),
                (LinuxDistro::Fedora, "glib2-devel".into()),
                (LinuxDistro::Suse, "glib2-devel".into()),
            ]),
        },
        Dependency {
            name: "openssl-dev".into(),
            dep_type: DepType::Include,
            filename: "openssl/ssl3.h".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "openssl".into()),
                (LinuxDistro::Alpine, "openssl-dev".into()),
                (LinuxDistro::Debian, "libssl-dev".into()),
                (LinuxDistro::Fedora, "openssl-devel".into()),
                (LinuxDistro::Suse, "openssl-devel".into()),
            ]),
        },
        Dependency {
            name: "libnotify-dev".into(),
            dep_type: DepType::Include,
            filename: "openssl/ssl3.h".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "libnotify".into()),
                (LinuxDistro::Alpine, "libnotify-dev".into()),
                (LinuxDistro::Debian, "libnotify-dev".into()),
                (LinuxDistro::Fedora, "libnotify-devel".into()),
                (LinuxDistro::Suse, "libnotify-devel".into()),
            ]),
        },
    ]
}

pub fn check_wivrn_deps() -> Vec<DependencyCheckResult> {
    Dependency::check_many(wivrn_deps())
}

pub fn get_missing_wivrn_deps() -> Vec<Dependency> {
    check_wivrn_deps()
        .iter()
        .filter(|res| !res.found)
        .map(|res| res.dependency.clone())
        .collect()
}
