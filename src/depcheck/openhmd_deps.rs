use super::{
    common::{dep_cmake, dep_gcc, dep_git, dep_gpp, dep_ninja},
    Dependency, DependencyCheckResult,
};

fn openhmd_deps() -> Vec<Dependency> {
    vec![dep_gcc(), dep_gpp(), dep_cmake(), dep_ninja(), dep_git()]
}

pub fn check_openhmd_deps() -> Vec<DependencyCheckResult> {
    Dependency::check_many(openhmd_deps())
}

pub fn get_missing_openhmd_deps() -> Vec<Dependency> {
    check_openhmd_deps()
        .iter()
        .filter(|res| !res.found)
        .map(|res| res.dependency.clone())
        .collect()
}
