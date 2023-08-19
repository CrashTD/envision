use crate::{
    build_tools::{cmake::Cmake, git::Git},
    file_utils::rm_rf,
    profile::Profile,
    runner::Runner,
};
use std::{collections::HashMap, path::Path};

pub fn get_build_wivrn_runners(profile: &Profile, clean_build: bool) -> Vec<Box<dyn Runner>> {
    let mut runners: Vec<Box<dyn Runner>> = vec![];
    let git = Git {
        repo: match profile.xrservice_repo.as_ref() {
            Some(r) => r.clone(),
            None => "https://github.com/Meumeu/WiVRn".into(),
        },
        dir: profile.xrservice_path.clone(),
    };
    runners.push(Box::new(git.get_override_remote_url_runner()));
    if let Some(r) = git.clone_or_pull(profile) {
        runners.push(Box::new(r));
    }
    if let Some(r) = git.get_checkout_ref_runner() {
        runners.push(Box::new(r));
        if profile.pull_on_build {
            runners.push(Box::new(git.get_pull_runner()));
        }
    }

    let build_dir = format!("{}/build", profile.xrservice_path);
    let mut cmake_vars: HashMap<String, String> = HashMap::new();
    cmake_vars.insert("CMAKE_BUILD_TYPE".into(), "Release".into());
    cmake_vars.insert("XRT_HAVE_SYSTEM_CJSON".into(), "NO".into());
    cmake_vars.insert("WIVRN_BUILD_CLIENT".into(), "OFF".into());
    cmake_vars.insert("CMAKE_INSTALL_PREFIX".into(), profile.prefix.clone());

    let cmake = Cmake {
        env: None,
        vars: Some(cmake_vars),
        source_dir: profile.xrservice_path.clone(),
        build_dir: build_dir.clone(),
    };
    if !Path::new(&build_dir).is_dir() || clean_build {
        rm_rf(&build_dir);
        runners.push(Box::new(cmake.get_prepare_runner()));
    }
    runners.push(Box::new(cmake.get_build_runner()));
    runners.push(Box::new(cmake.get_install_runner()));

    runners
}
