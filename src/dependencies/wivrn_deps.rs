use crate::depcheck::{check_dependencies, DepType, Dependency, DependencyCheckResult};

fn wivrn_deps() -> Vec<Dependency> {
    // TODO: populate!
    vec![Dependency {
        name: "x264-dev".into(),
        dep_type: DepType::Include,
        filename: "x265.h".into(),
    }]
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
