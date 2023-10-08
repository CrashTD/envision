use crate::{
    depcheck::{check_dependencies, DepType, Dependency, DependencyCheckResult},
    dependencies::common::{dep_eigen, dep_glslang_validator},
    linux_distro::LinuxDistro,
};
use std::collections::HashMap;

fn wivrn_deps() -> Vec<Dependency> {
    // TODO: populate!
    vec![
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
            name: "avahi-client-dev".into(),
            dep_type: DepType::Include,
            filename: "avahi-client/client.h".into(),
            packages: HashMap::from([]), // TODO
        },
        Dependency {
            name: "libpulse-dev".into(),
            dep_type: DepType::Include,
            filename: "pulse/context.h".into(),
            packages: HashMap::from([]), // TODO
        },
        dep_eigen(),
        Dependency {
            name: "nlohmann-json".into(),
            dep_type: DepType::Include,
            filename: "nlohmann/json.hpp".into(),
            packages: HashMap::from([]), // TODO
        },
        // Dependency {
        //     name: "libavcodec-dev".into(),
        //     dep_type: DepType::Include,
        //     filename: "libavcodec/avcodec.h".into(),
        // },
        // Dependency {
        //     name: "libavfilter-dev".into(),
        //     dep_type: DepType::Include,
        //     filename: "libavfilter/avfilter.h".into(),
        // },
        // Dependency {
        //     name: "libswscale-dev".into(),
        //     dep_type: DepType::Include,
        //     filename: "libswscale/swscale.h".into(),
        // },
        dep_glslang_validator(),
        Dependency {
            name: "libudev".into(),
            dep_type: DepType::Include,
            filename: "libudev.h".into(),
            packages: HashMap::from([]), // TODO
        },
        Dependency {
            name: "gstreamer".into(),
            dep_type: DepType::SharedObject,
            filename: "libgstreamer-1.0.so".into(),
            packages: HashMap::from([]), // TODO
        },
        Dependency {
            name: "gst-plugins-base-libs".into(),
            dep_type: DepType::SharedObject,
            filename: "pkgconfig/gstreamer-app-1.0.pc".into(),
            packages: HashMap::from([]), // TODO
        },
        Dependency {
            name: "systemd-dev".into(),
            dep_type: DepType::Include,
            filename: "systemd/sd-daemon.h".into(),
            packages: HashMap::from([]), // TODO
        },
        Dependency {
            name: "libva-dev".into(),
            dep_type: DepType::Include,
            filename: "va/va.h".into(),
            packages: HashMap::from([]), // TODO
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
