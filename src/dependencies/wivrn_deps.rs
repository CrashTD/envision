use crate::depcheck::{Dependency, check_dependencies, DependencyCheckResult};

fn wivrn_deps() -> Vec<Dependency> {
    // TODO: populate!
    vec![]
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
