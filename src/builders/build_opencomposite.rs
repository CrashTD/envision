use crate::{
    build_tools::{cmake::Cmake, git::Git},
    file_utils::rm_rf,
    profile::Profile,
    runner::Runner,
};
use std::{collections::HashMap, path::Path};

pub fn get_build_opencomposite_runners(profile: &Profile, clean_build: bool) -> Vec<Runner> {
    let mut runners: Vec<Runner> = vec![];
    let git = Git {
        repo: match profile.opencomposite_repo.as_ref() {
            Some(r) => r.clone(),
            None => "https://gitlab.com/znixian/OpenOVR.git".into(),
        },
        dir: profile.opencomposite_path.clone(),
    };
    match git.clone_or_pull(profile) {
        Some(r) => runners.push(r),
        None => {}
    };
    match git.get_checkout_ref_runner() {
        Some(r) => {
            runners.push(r);
            if profile.pull_on_build {
                runners.push(git.get_pull_runner());
            }
        }
        None => {}
    }

    let build_dir = format!("{}/build", profile.opencomposite_path);
    let mut cmake_vars: HashMap<String, String> = HashMap::new();
    cmake_vars.insert("CMAKE_BUILD_TYPE".into(), "Release".into());
    let cmake = Cmake {
        env: None,
        vars: Some(cmake_vars),
        source_dir: profile.opencomposite_path.clone(),
        build_dir: build_dir.clone(),
    };
    if !Path::new(&build_dir).is_dir() || clean_build {
        rm_rf(&build_dir);
        runners.push(cmake.get_prepare_runner());
    }
    runners.push(cmake.get_build_runner());

    runners
}
