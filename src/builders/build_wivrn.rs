use crate::{
    build_tools::{cmake::Cmake, git::Git},
    file_utils::rm_rf,
    profile::Profile,
    termcolor::TermColor,
    ui::job_worker::job::WorkerJob,
};
use std::{
    collections::{HashMap, VecDeque},
    path::Path,
};

pub fn get_build_wivrn_jobs(profile: &Profile, clean_build: bool) -> VecDeque<WorkerJob> {
    let mut jobs = VecDeque::<WorkerJob>::new();
    jobs.push_back(WorkerJob::new_printer(
        "Building WiVRn...",
        Some(TermColor::Blue),
    ));

    let git = Git {
        repo: profile
            .xrservice_repo
            .as_ref()
            .unwrap_or(&"https://github.com/WiVRn/WiVRn".into())
            .clone(),
        dir: profile.xrservice_path.clone(),
        branch: profile
            .xrservice_branch
            .as_ref()
            .unwrap_or(&"release".into())
            .clone(),
    };

    jobs.extend(git.get_pre_build_jobs(profile.pull_on_build));

    let build_dir = profile.xrservice_path.join("build");
    let mut cmake_vars: HashMap<String, String> = HashMap::new();
    cmake_vars.insert("CMAKE_EXPORT_COMPILE_COMMANDS".into(), "ON".into());
    cmake_vars.insert("CMAKE_BUILD_TYPE".into(), "RelWithDebInfo".into());
    cmake_vars.insert("XRT_HAVE_SYSTEM_CJSON".into(), "NO".into());
    cmake_vars.insert("WIVRN_BUILD_CLIENT".into(), "OFF".into());
    cmake_vars.insert(
        "CMAKE_INSTALL_PREFIX".into(),
        profile.prefix.to_string_lossy().to_string(),
    );

    profile.xrservice_cmake_flags.iter().for_each(|(k, v)| {
        cmake_vars.insert(k.clone(), v.clone());
    });

    let cmake = Cmake {
        env: None,
        vars: Some(cmake_vars),
        source_dir: profile.xrservice_path.clone(),
        build_dir: build_dir.clone(),
    };
    if !Path::new(&build_dir).is_dir() || clean_build {
        rm_rf(&build_dir);
        jobs.push_back(cmake.get_prepare_job());
    }
    jobs.push_back(cmake.get_build_job());
    jobs.push_back(cmake.get_install_job());

    jobs
}
