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

pub fn get_build_opencomposite_jobs(profile: &Profile, clean_build: bool) -> VecDeque<WorkerJob> {
    let mut jobs = VecDeque::<WorkerJob>::new();

    let git = Git {
        repo: profile
            .opencomposite_repo
            .as_ref()
            .unwrap_or(&"https://gitlab.com/znixian/OpenOVR.git".into())
            .clone(),
        dir: profile.opencomposite_path.clone(),
        branch: profile
            .opencomposite_branch
            .as_ref()
            .unwrap_or(&"openxr".into())
            .clone(),
    };

    jobs.extend(git.get_pre_build_jobs(profile.pull_on_build));

    let build_dir = format!("{}/build", profile.opencomposite_path);
    let mut cmake_vars: HashMap<String, String> = HashMap::new();
    cmake_vars.insert("CMAKE_EXPORT_COMPILE_COMMANDS".into(), "ON".into());
    cmake_vars.insert("CMAKE_BUILD_TYPE".into(), "RelWithDebInfo".into());
    let cmake = Cmake {
        env: None,
        vars: Some(cmake_vars),
        source_dir: profile.opencomposite_path.clone(),
        build_dir: build_dir.clone(),
    };
    if !Path::new(&build_dir).is_dir() || clean_build {
        rm_rf(&build_dir);
        jobs.push_back(cmake.get_prepare_job());
    }
    jobs.push_back(cmake.get_build_job());

    jobs
}
