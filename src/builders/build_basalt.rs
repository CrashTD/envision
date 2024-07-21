use crate::{
    build_tools::{cmake::Cmake, git::Git},
    file_utils::rm_rf,
    profile::Profile,
    ui::job_worker::job::WorkerJob,
};
use std::collections::{HashMap, VecDeque};

pub fn get_build_basalt_jobs(profile: &Profile, clean_build: bool) -> VecDeque<WorkerJob> {
    let mut jobs = VecDeque::<WorkerJob>::new();

    let git = Git {
        repo: profile
            .features
            .basalt
            .repo
            .as_ref()
            .unwrap_or(&"https://gitlab.freedesktop.org/mateosss/basalt.git".into())
            .clone(),
        dir: profile.features.basalt.path.as_ref().unwrap().clone(),
        branch: profile
            .features
            .basalt
            .branch
            .as_ref()
            .unwrap_or(&"main".into())
            .clone(),
    };

    jobs.extend(git.get_pre_build_jobs(profile.pull_on_build));

    let build_dir = profile.features.basalt.path.as_ref().unwrap().join("build");
    let mut cmake_vars: HashMap<String, String> = HashMap::new();
    cmake_vars.insert("CMAKE_EXPORT_COMPILE_COMMANDS".into(), "ON".into());
    cmake_vars.insert("CMAKE_BUILD_TYPE".into(), "RelWithDebInfo".into());
    cmake_vars.insert(
        "CMAKE_INSTALL_PREFIX".into(),
        profile.prefix.to_string_lossy().to_string(),
    );
    cmake_vars.insert("BUILD_TESTS".into(), "OFF".into());
    cmake_vars.insert("BASALT_INSTANTIATIONS_DOUBLE".into(), "OFF".into());
    cmake_vars.insert(
        "CMAKE_INSTALL_LIBDIR".into(),
        profile.prefix.join("lib").to_string_lossy().to_string(),
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

    if !build_dir.is_dir() || clean_build {
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
            profile
                .prefix
                .join("share/basalt/thirdparty/basalt-headers/thirdparty")
                .to_string_lossy()
                .to_string(),
        ]),
    ));
    jobs.push_back(WorkerJob::new_cmd(
        None,
        "cp".into(),
        Some(vec![
            "-Ra".into(),
            profile
                .features
                .basalt
                .path
                .as_ref()
                .unwrap()
                .join("thirdparty/basalt-headers/thirdparty/eigen")
                .to_string_lossy()
                .to_string(),
            profile
                .prefix
                .join("share/basalt/thirdparty")
                .to_string_lossy()
                .to_string(),
        ]),
    ));

    jobs
}
