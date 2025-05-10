use super::{
    common::{dep_cmake, dep_eigen, dep_gcc, dep_git, dep_gpp, dep_ninja},
    DepcheckResultGetMissing, Dependency, DependencyCheckResult,
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
    Dependency::check_many(libsurvive_deps())
}

pub fn get_missing_libsurvive_deps() -> Vec<Dependency> {
    check_libsurvive_deps().filter_missing_deps()
}
