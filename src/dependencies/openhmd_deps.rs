use crate::depcheck::{check_dependencies, Dependency, DependencyCheckResult};

fn openhmd_deps() -> Vec<Dependency> {
    vec![]
}

pub fn check_openhmd_deps() -> Vec<DependencyCheckResult> {
    check_dependencies(openhmd_deps())
}

pub fn get_missing_openhmd_deps() -> Vec<Dependency> {
    check_openhmd_deps()
        .iter()
        .filter(|res| !res.found)
        .map(|res| res.dependency.clone())
        .collect()
}
