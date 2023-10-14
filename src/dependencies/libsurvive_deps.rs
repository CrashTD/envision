use crate::{
    depcheck::{check_dependencies, Dependency, DependencyCheckResult},
    dependencies::common::{dep_cmake, dep_eigen, dep_gcc, dep_git, dep_gpp, dep_ninja},
};

fn libsurvive_deps() -> Vec<Dependency> {
    vec![
        dep_eigen(),
        dep_cmake(),
        dep_git(),
        dep_ninja(),
        dep_gcc(),
        dep_gpp(),
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
