use crate::{
    build_tools::{cmake::Cmake, git::Git},
    file_utils::rm_rf,
    profile::Profile,
    runner::Runner,
};
use std::{collections::HashMap, path::Path};

pub fn get_build_basalt_runners(profile: &Profile, clean_build: bool) -> Vec<Runner> {
    let mut runners: Vec<Runner> = vec![];
    let git = Git {
        repo: match profile.features.basalt.repo.as_ref() {
            Some(r) => r.clone(),
            None => "https://gitlab.freedesktop.org/mateosss/basalt.git".into(),
        },
        dir: profile.features.basalt.path.as_ref().unwrap().clone(),
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

    let build_dir = format!("{}/build", profile.features.basalt.path.as_ref().unwrap());
    let mut cmake_vars: HashMap<String, String> = HashMap::new();
    cmake_vars.insert("CMAKE_BUILD_TYPE".into(), "Release".into());
    cmake_vars.insert("CMAKE_INSTALL_PREFIX".into(), profile.prefix.clone());
    cmake_vars.insert("BUILD_TESTS".into(), "OFF".into());
    cmake_vars.insert("BASALT_INSTANTIATIONS_DOUBLE".into(), "OFF".into());
    cmake_vars.insert(
        "CMAKE_INSTALL_LIBDIR".into(),
        format!("{}/lib", profile.prefix),
    );

    let mut cmake_env: HashMap<String, String> = HashMap::new();
    cmake_env.insert("CMAKE_BUILD_PARALLEL_LEVEL".into(), "2".into());

    let cmake = Cmake {
        env: Some(cmake_env),
        vars: Some(cmake_vars),
        source_dir: profile.features.basalt.path.as_ref().unwrap().clone(),
        build_dir: build_dir.clone(),
    };
    runners.push(Runner::new(None, "bash".into(), vec![
        "-c".into(),
        format!(
            "cd {repo}/thirdparty/Pangolin && git checkout include/pangolin/utils/picojson.h && curl -sSL 'https://aur.archlinux.org/cgit/aur.git/plain/279c17d9c9eb9374c89489b449f92cb93350e8cd.patch?h=basalt-monado-git' -o picojson_fix.patch && git apply picojson_fix.patch &&	sed -i '1s/^/#include <stdint.h>\\n/' include/pangolin/platform.h",
            repo = git.dir
        ),
    ]));
    if !Path::new(&build_dir).is_dir() || clean_build {
        rm_rf(&build_dir);
        runners.push(cmake.get_prepare_runner());
    }
    runners.push(cmake.get_build_runner());
    runners.push(cmake.get_install_runner());

    runners.push(Runner::new(
        None,
        "mkdir".into(),
        vec![
            "-p".into(),
            format!(
                "{}/share/basalt/thirdparty/basalt-headers/thirdparty",
                profile.prefix
            ),
        ],
    ));
    runners.push(Runner::new(
        None,
        "cp".into(),
        vec![
            "-Ra".into(),
            format!(
                "{}/thirdparty/basalt-headers/thirdparty/eigen",
                profile.features.basalt.path.as_ref().unwrap().clone()
            ),
            format!("{}/share/basalt/thirdparty", profile.prefix),
        ],
    ));

    runners
}
