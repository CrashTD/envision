use crate::{
    build_tools::{cmake::Cmake, git::Git},
    file_utils::rm_rf,
    profile::Profile,
    runner::Runner,
};
use std::collections::HashMap;

pub fn get_build_monado_runners(profile: &Profile) -> Vec<Runner> {
    let mut runners: Vec<Runner> = vec![];
    let git = Git {
        repo: match profile.xrservice_repo.as_ref() {
            Some(r) => r.clone(),
            None => "https://gitlab.freedesktop.org/monado/monado".into(),
        },
        dir: profile.xrservice_path.clone(),
    };
    match git.clone_or_pull(profile) {
        Some(r) => runners.push(r),
        None => {}
    };

    let build_dir = format!("{}/build", profile.xrservice_path);
    rm_rf(&build_dir);
    let mut env: HashMap<String, String> = HashMap::new();
    env.insert(
        "PKG_CONFIG_PATH".into(),
        format!("{}/lib/pkgconfig", profile.prefix),
    );
    let mut cmake_vars: HashMap<String, String> = HashMap::new();
    cmake_vars.insert("CMAKE_BUILD_TYPE".into(), "Release".into());
    cmake_vars.insert("XRT_HAVE_SYSTEM_CJSON".into(), "NO".into());
    cmake_vars.insert("CMAKE_LIBDIR".into(), format!("{}/lib", profile.prefix));
    cmake_vars.insert("CMAKE_INSTALL_PREFIX".into(), profile.prefix.clone());
    cmake_vars.insert(
        "CMAKE_C_FLAGS".into(),
        format!("-Wl,-rpath='{}/lib'", profile.prefix),
    );
    cmake_vars.insert(
        "CMAKE_CXX_FLAGS".into(),
        format!("-Wl,-rpath='{}/lib'", profile.prefix),
    );

    let cmake = Cmake {
        env: Some(env),
        vars: Some(cmake_vars),
        source_dir: profile.xrservice_path.clone(),
        build_dir,
    };
    runners.push(cmake.get_prepare_runner());
    runners.push(cmake.get_build_runner());
    runners.push(cmake.get_install_runner());

    runners
}
