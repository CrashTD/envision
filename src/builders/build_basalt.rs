use crate::{
    build_tools::{cmake::Cmake, git::Git},
    file_utils::rm_rf,
    profile::Profile,
    ui::job_worker::job::WorkerJob,
};
use std::{
    collections::{HashMap, VecDeque},
    path::Path,
};

pub fn get_build_basalt_jobs(profile: &Profile, clean_build: bool) -> VecDeque<WorkerJob> {
    let mut jobs = VecDeque::<WorkerJob>::new();

    let git = Git {
        repo: match profile.features.basalt.repo.as_ref() {
            Some(r) => r.clone(),
            None => "https://gitlab.freedesktop.org/mateosss/basalt.git".into(),
        },
        dir: profile.features.basalt.path.as_ref().unwrap().clone(),
        branch: match profile.features.basalt.branch.as_ref() {
            Some(r) => r.clone(),
            None => "main".into(),
        },
    };

    jobs.extend(git.get_pre_build_jobs(profile.pull_on_build));

    let build_dir = format!("{}/build", profile.features.basalt.path.as_ref().unwrap());
    let mut cmake_vars: HashMap<String, String> = HashMap::new();
    cmake_vars.insert("CMAKE_EXPORT_COMPILE_COMMANDS".into(), "ON".into());
    cmake_vars.insert("CMAKE_BUILD_TYPE".into(), "RelWithDebInfo".into());
    cmake_vars.insert("CMAKE_INSTALL_PREFIX".into(), profile.prefix.clone());
    cmake_vars.insert("BUILD_TESTS".into(), "OFF".into());
    cmake_vars.insert("BASALT_INSTANTIATIONS_DOUBLE".into(), "OFF".into());
    cmake_vars.insert(
        "CMAKE_INSTALL_LIBDIR".into(),
        format!("{}/lib", profile.prefix),
    );

    let mut cmake_env: HashMap<String, String> = HashMap::new();
    cmake_env.insert("CMAKE_BUILD_PARALLEL_LEVEL".into(), "2".into());
    cmake_env.insert("CMAKE_BUILD_TYPE".into(), "RelWithDebInfo".into());
    cmake_env.insert("BUILD_TESTS".into(), "off".into());

    let cmake = Cmake {
        env: Some(cmake_env),
        vars: Some(cmake_vars),
        source_dir: profile.features.basalt.path.as_ref().unwrap().clone(),
        build_dir: build_dir.clone(),
    };

    jobs.push_back(WorkerJob::new_cmd(None, "bash".into(), Some(vec![
        "-c".into(),
        format!(
            "cd {repo}/thirdparty/Pangolin && git checkout include/pangolin/utils/picojson.h && curl -sSL 'https://aur.archlinux.org/cgit/aur.git/plain/279c17d9c9eb9374c89489b449f92cb93350e8cd.patch?h=basalt-monado-git' -o picojson_fix.patch && git apply picojson_fix.patch &&	sed -i '1s/^/#include <stdint.h>\\n/' include/pangolin/platform.h",
            repo = git.dir
        ),
    ])));

    if !Path::new(&build_dir).is_dir() || clean_build {
        rm_rf(&build_dir);
        jobs.push_back(cmake.get_prepare_job());
    }
    jobs.push_back(cmake.get_build_job());
    jobs.push_back(cmake.get_install_job());

    jobs.push_back(WorkerJob::new_cmd(
        None,
        "mkdir".into(),
        Some(vec![
            "-p".into(),
            format!(
                "{}/share/basalt/thirdparty/basalt-headers/thirdparty",
                profile.prefix
            ),
        ]),
    ));
    jobs.push_back(WorkerJob::new_cmd(
        None,
        "cp".into(),
        Some(vec![
            "-Ra".into(),
            format!(
                "{}/thirdparty/basalt-headers/thirdparty/eigen",
                profile.features.basalt.path.as_ref().unwrap().clone()
            ),
            format!("{}/share/basalt/thirdparty", profile.prefix),
        ]),
    ));

    jobs
}
