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
        repo: match profile.opencomposite_repo.as_ref() {
            Some(r) => r.clone(),
            None => "https://gitlab.com/znixian/OpenOVR.git".into(),
        },
        dir: profile.opencomposite_path.clone(),
    };

    jobs.push_back(git.get_override_remote_url_job());

    git.get_clone_or_pull_job(profile).map(|j| {
        jobs.push_back(j);
    });

    git.get_checkout_ref_job().map(|j| {
        jobs.push_back(j);
        if profile.pull_on_build {
            jobs.push_back(git.get_pull_job());
        }
    });

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
        jobs.push_back(cmake.get_prepare_job());
    }
    jobs.push_back(cmake.get_build_job());

    jobs
}
