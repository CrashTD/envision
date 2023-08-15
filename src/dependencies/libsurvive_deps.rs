use crate::depcheck::{check_dependencies, DepType, Dependency, DependencyCheckResult};

fn libsurvive_deps() -> Vec<Dependency> {
    vec![
        Dependency {
            name: "eigen".into(),
            dep_type: DepType::Include,
            filename: "eigen3/Eigen/src/Core/EigenBase.h".into(),
        },
        Dependency {
            name: "cmake".into(),
            dep_type: DepType::Executable,
            filename: "cmake".into(),
        },
        Dependency {
            name: "git".into(),
            dep_type: DepType::Executable,
            filename: "git".into(),
        },
        Dependency {
            name: "ninja".into(),
            dep_type: DepType::Executable,
            filename: "ninja".into(),
        },
    ]
}

pub fn check_libsurvive_deps() -> Vec<DependencyCheckResult> {
    check_dependencies(libsurvive_deps())
}

pub fn get_missing_libsurvive_deps() -> Vec<Dependency> {
    check_libsurvive_deps()
        .iter()
        .filter(|res| !res.found)
        .map(|res| res.dependency.clone())
        .collect()
}
