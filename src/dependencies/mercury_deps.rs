use crate::depcheck::{check_dependencies, DepType, Dependency, DependencyCheckResult};

fn mercury_deps() -> Vec<Dependency> {
    vec![
        Dependency {
            name: "opencv".into(),
            dep_type: DepType::SharedObject,
            filename: "libopencv_core.so".into(),
        },
        Dependency {
            name: "opencv-dev".into(),
            dep_type: DepType::Include,
            filename: "opencv4/opencv2/core/base.hpp".into(),
        },
        Dependency {
            name: "opencv-dev".into(),
            dep_type: DepType::Include,
            filename: "opencv4/opencv2/core/base.hpp".into(),
        },
        Dependency {
            name: "git-lfs".into(),
            dep_type: DepType::Executable,
            filename: "git-lfs".into(),
        },
    ]
}

pub fn check_mercury_deps() -> Vec<DependencyCheckResult> {
    check_dependencies(mercury_deps())
}

pub fn get_missing_mercury_deps() -> Vec<Dependency> {
    check_mercury_deps()
        .iter()
        .filter(|res| !res.found)
        .map(|res| res.dependency.clone())
        .collect()
}
