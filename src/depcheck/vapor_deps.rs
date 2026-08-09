use super::{DepType, DepcheckResultGetMissing, Dependency, DependencyCheckResult};
use crate::{
    depcheck::common::{
        dep_cmake, dep_git, dep_libc, dep_libgl, dep_libx11, dep_openxr, dep_vulkan_headers,
        dep_vulkan_icd_loader,
    },
    linux_distro::LinuxDistro,
};
use std::collections::HashMap;

fn vapor_deps() -> Vec<Dependency> {
    vec![
        // makedepends:
        dep_git(),
        // make?
        dep_cmake(),
        Dependency {
            name: "nlohmann-json".into(),
            dep_type: DepType::Include,
            filename: "nlohmann/json.hpp".into(),
            packages: HashMap::from([
                (LinuxDistro::Arch, "nlohmann-json".into()),
                (LinuxDistro::Debian, "nlohmann-json3-dev".into()),
                (LinuxDistro::Fedora, "json-devel".into()),
                (LinuxDistro::Gentoo, "dev-cpp/nlohmann_json".into()),
                (LinuxDistro::Suse, "nlohmann_json".into()),
            ]),
        },
        dep_vulkan_headers(),
        // runtime deps:
        dep_openxr(),
        dep_vulkan_icd_loader(),
        // libgcc?
        dep_libgl(),
        dep_libc(),
        dep_libx11(),
        // libstdc++?
    ]
}

pub fn check_vapor_deps() -> Vec<DependencyCheckResult> {
    Dependency::check_many(vapor_deps())
}

pub fn get_missing_vapor_deps() -> Vec<Dependency> {
    check_vapor_deps().filter_missing_deps()
}
