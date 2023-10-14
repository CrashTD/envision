use crate::{
    depcheck::{check_dependencies, DepType, Dependency, DependencyCheckResult},
    dependencies::common::{dep_cmake, dep_eigen, dep_git, dep_glslang_validator, dep_ninja},
    linux_distro::LinuxDistro,
};
use std::collections::HashMap;

fn wivrn_deps() -> Vec<Dependency> {
    // TODO: populate!
    vec![
        dep_cmake(),
        dep_ninja(),
        dep_git(),
        Dependency {
            name: "x264-dev".into(),
            dep_type: DepType::Include,
            filename: "x264.h".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "x264".into()),
                (LinuxDistro::Debian, "libx264-dev".into()),
                (LinuxDistro::Fedora, "x264-devel".into()),
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
            ]),
        },
        Dependency {
            name: "libavcodec-dev".into(),
            dep_type: DepType::Include,
            filename: "libavcodec/avcodec.h".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "ffmpeg".into()),
                (LinuxDistro::Debian, "libavcodec-dev".into()),
                (LinuxDistro::Fedora, "libavcodec-devel".into()),
            ]),
        },
        Dependency {
            name: "libavfilter-dev".into(),
            dep_type: DepType::Include,
            filename: "libavfilter/avfilter.h".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "ffmpeg".into()),
                (LinuxDistro::Debian, "libavfilter-dev".into()),
                (LinuxDistro::Fedora, "libavfilter-devel".into()),
            ]),
        },
        Dependency {
            name: "libswscale-dev".into(),
            dep_type: DepType::Include,
            filename: "libswscale/swscale.h".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "ffmpeg".into()),
                (LinuxDistro::Debian, "libswscale-dev".into()),
                (LinuxDistro::Fedora, "libswscale-devel".into()),
            ]),
        },
        dep_glslang_validator(),
        Dependency {
            name: "libudev".into(),
            dep_type: DepType::Include,
            filename: "libudev.h".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "systemd-libs".into()),
                (LinuxDistro::Debian, "libudev-dev".into()),
                (LinuxDistro::Fedora, "systemd-devel".into()),
            ]),
        },
        Dependency {
            name: "gstreamer".into(),
            dep_type: DepType::SharedObject,
            filename: "libgstreamer-1.0.so".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "gstreamer".into()),
                (LinuxDistro::Debian, "libgstreamer1.0-dev".into()),
                (LinuxDistro::Fedora, "gstreamer1-devel".into()),
            ]),
        },
        Dependency {
            name: "gst-plugins-base-libs".into(),
            dep_type: DepType::SharedObject,
            filename: "pkgconfig/gstreamer-app-1.0.pc".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "gst-plugins-base-libs".into()),
                (LinuxDistro::Debian, "libgstreamer1.0-dev".into()),
                (LinuxDistro::Fedora, "gstreamer1-devel".into()),
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
            ]),
        },
    ]
}

pub fn check_wivrn_deps() -> Vec<DependencyCheckResult> {
    check_dependencies(wivrn_deps())
}

pub fn get_missing_wivrn_deps() -> Vec<Dependency> {
    check_wivrn_deps()
        .iter()
        .filter(|res| !res.found)
        .map(|res| res.dependency.clone())
        .collect()
}
