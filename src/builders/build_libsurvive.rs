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

pub fn get_build_libsurvive_jobs(profile: &Profile, clean_build: bool) -> VecDeque<WorkerJob> {
    let mut jobs = VecDeque::<WorkerJob>::new();

    let git = Git {
        repo: match profile.features.libsurvive.repo.as_ref() {
            Some(r) => r.clone(),
            None => "https://github.com/cntools/libsurvive".into(),
        },
        dir: profile.features.libsurvive.path.as_ref().unwrap().clone(),
        branch: match profile.features.libsurvive.branch.as_ref() {
            Some(r) => r.clone(),
            None => "master".into(),
        },
    };

    jobs.extend(git.get_pre_build_jobs(profile.pull_on_build));

    let build_dir = format!(
        "{}/build",
        profile.features.libsurvive.path.as_ref().unwrap()
    );
    let mut cmake_vars: HashMap<String, String> = HashMap::new();
    cmake_vars.insert("CMAKE_BUILD_TYPE".into(), "RelWithDebInfo".into());
    cmake_vars.insert("ENABLE_api_example".into(), "OFF".into());
    cmake_vars.insert("USE_HIDAPI".into(), "ON".into());
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
        jobs.push_back(cmake.get_prepare_job());
    }
    jobs.push_back(cmake.get_build_job());
    jobs.push_back(cmake.get_install_job());

    jobs
}
