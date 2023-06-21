use crate::depcheck::{check_dependencies, DepType, Dependency, DependencyCheckResult};

fn wivrn_deps() -> Vec<Dependency> {
    // TODO: populate!
    vec![
        Dependency {
            name: "x264-dev".into(),
            dep_type: DepType::Include,
            filename: "x264.h".into(),
        },
        Dependency {
            name: "avahi-client-dev".into(),
            dep_type: DepType::Include,
            filename: "avahi-client/client.h".into(),
        },
        Dependency {
            name: "libpulse-dev".into(),
            dep_type: DepType::Include,
            filename: "pulse/context.h".into(),
        },
        Dependency {
            name: "eigen".into(),
            dep_type: DepType::Include,
            filename: "eigen3/Eigen/src/Core/EigenBase.h".into(),
        },
        Dependency {
            name: "nlohmann-json".into(),
            dep_type: DepType::Include,
            filename: "nlohmann/json.hpp".into(),
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
        Dependency {
            name: "glslang".into(),
            dep_type: DepType::Executable,
            filename: "glslangValidator".into(),
        },
        Dependency {
            name: "libudev".into(),
            dep_type: DepType::Include,
            filename: "libudev.h".into(),
        },
        Dependency {
            name: "gstreamer".into(),
            dep_type: DepType::SharedObject,
            filename: "libgstreamer-1.0.so".into(),
        },
        Dependency {
            name: "gst-plugins-base-libs".into(),
            dep_type: DepType::SharedObject,
            filename: "pkgconfig/gstreamer-app-1.0.pc".into(),
        },
        Dependency {
            name: "systemd-dev".into(),
            dep_type: DepType::Include,
            filename: "systemd/sd-daemon.h".into(),
        },
        Dependency {
            name: "libva-dev".into(),
            dep_type: DepType::Include,
            filename: "va/va.h".into(),
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
