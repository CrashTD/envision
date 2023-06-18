use crate::depcheck::{check_dependencies, Dependency, DependencyCheckResult, DepType};

fn basalt_deps() -> Vec<Dependency> {
    vec![
        Dependency {
            name: "boost".into(),
            dep_type: DepType::SharedObject,
            // just one of the many shared objects boost provides
            filename: "libboost_system.so".into(),
        },
        Dependency {
            name: "boost-dev".into(),
            dep_type: DepType::Include,
            // just one of the many headers boost provides
            filename: "boost/filesystem.hpp".into(),
        },
        Dependency {
            name: "bzip2".into(),
            dep_type: DepType::SharedObject,
            filename: "libbz2.so".into(),
        },
        Dependency {
            name: "bzip2-dev".into(),
            dep_type: DepType::Include,
            filename: "bzlib.h".into(),
        },
        Dependency {
            name: "eigen".into(),
            dep_type: DepType::Include,
            filename: "eigen3/Eigen/src/Core/EigenBase.h".into(),
        },
        Dependency {
            name: "fmt".into(),
            dep_type: DepType::SharedObject,
            filename: "libfmt.so".into(),
        },
        Dependency {
            name: "fmt-dev".into(),
            dep_type: DepType::Include,
            filename: "fmt/core.h".into(),
        },
        Dependency {
            name: "glew".into(),
            dep_type: DepType::SharedObject,
            filename: "libGLEW.so".into(),
        },
        Dependency {
            name: "glew-dev".into(),
            dep_type: DepType::Include,
            filename: "GL/glew.h".into(),
        },
        Dependency {
            name: "gtest".into(),
            dep_type: DepType::Include,
            filename: "gtest/gtest.h".into(),
        },
        Dependency {
            name: "opencv".into(),
            dep_type: DepType::Include,
            filename: "opencv4/opencv2/core/hal/hal.hpp".into(),
        },
        Dependency {
            name: "python3".into(),
            dep_type: DepType::Executable,
            filename: "python3".into(),
        },
        Dependency {
            name: "bc".into(),
            dep_type: DepType::Executable,
            filename: "bc".into(),
        },
    ]
}

pub fn check_basalt_deps() -> Vec<DependencyCheckResult> {
    check_dependencies(basalt_deps())
}

pub fn get_missing_basalt_deps() -> Vec<Dependency> {
    check_basalt_deps()
        .iter()
        .filter(|res| !res.found)
        .map(|res| res.dependency.clone())
        .collect()
}
