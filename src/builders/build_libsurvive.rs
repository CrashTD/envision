use crate::{
    build_tools::{cmake::Cmake, git::Git},
    file_utils::rm_rf,
    profile::Profile,
    runner::Runner,
};
use std::{collections::HashMap, path::Path};

pub fn get_build_libsurvive_runners(profile: &Profile, clean_build: bool) -> Vec<Runner> {
    let mut runners: Vec<Runner> = vec![];
    let git = Git {
        repo: match profile.features.libsurvive.repo.as_ref() {
            Some(r) => r.clone(),
            None => "https://github.com/cntools/libsurvive".into(),
        },
        dir: profile.features.libsurvive.path.as_ref().unwrap().clone(),
    };
    if let Some(r) = git.clone_or_pull(profile) {
        runners.push(r);
    };
    if let Some(r) = git.get_checkout_ref_runner() {
        runners.push(r);
        if profile.pull_on_build {
            runners.push(git.get_pull_runner());
        }
    }

    let build_dir = format!(
        "{}/build",
        profile.features.libsurvive.path.as_ref().unwrap()
    );
    let mut cmake_vars: HashMap<String, String> = HashMap::new();
    cmake_vars.insert("CMAKE_BUILD_TYPE".into(), "Release".into());
    cmake_vars.insert("ENABLE_api_example".into(), "OFF".into());
    cmake_vars.insert("CMAKE_SKIP_INSTALL_RPATH".into(), "YES".into());
    cmake_vars.insert("CMAKE_INSTALL_PREFIX".into(), profile.prefix.clone());
    cmake_vars.insert(
        "CMAKE_INSTALL_LIBDIR".into(),
        format!("{}/lib", profile.prefix),
    );

    let cmake = Cmake {
        env: None,
        vars: Some(cmake_vars),
        source_dir: profile.features.libsurvive.path.as_ref().unwrap().clone(),
        build_dir: build_dir.clone(),
    };
    if !Path::new(&build_dir).is_dir() || clean_build {
        rm_rf(&build_dir);
        runners.push(cmake.get_prepare_runner());
    }
    runners.push(cmake.get_build_runner());
    runners.push(cmake.get_install_runner());

    runners
}
