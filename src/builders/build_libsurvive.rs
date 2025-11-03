use crate::{
    build_tools::{cmake::Cmake, git::Git},
    profile::Profile,
    termcolor::TermColor,
    ui::job_worker::job::WorkerJob,
    util::{
        file_utils::rm_rf,
        git_repos::{DEFAULT_LIBSURVIVE_BRANCH, DEFAULT_LIBSURVIVE_REPO},
    },
};
use std::{
    collections::{HashMap, VecDeque},
    path::Path,
};

pub fn get_build_libsurvive_jobs(profile: &Profile, clean_build: bool) -> VecDeque<WorkerJob> {
    let mut jobs = VecDeque::<WorkerJob>::new();
    jobs.push_back(WorkerJob::new_printer(
        "Building Libsurvive...",
        Some(TermColor::Blue),
    ));

    let git = Git {
        repo: profile
            .features
            .libsurvive
            .repo
            .as_ref()
            .unwrap_or(&DEFAULT_LIBSURVIVE_REPO.into())
            .clone(),
        dir: profile.features.libsurvive.path.as_ref().unwrap().clone(),
        branch: profile
            .features
            .libsurvive
            .branch
            .as_ref()
            .unwrap_or(&DEFAULT_LIBSURVIVE_BRANCH.into())
            .clone(),
    };

    jobs.extend(git.get_pre_build_jobs(profile.pull_on_build));

    let build_dir = profile
        .features
        .libsurvive
        .path
        .as_ref()
        .unwrap()
        .join("build");

    let cmake = Cmake {
        env: None,
        vars: Some({
            let mut cmake_vars: HashMap<String, String> = HashMap::new();
            for (k, v) in [
                ("CMAKE_EXPORT_COMPILE_COMMANDS", "ON"),
                ("CMAKE_BUILD_TYPE", "RelWithDebInfo"),
                ("ENABLE_api_example", "OFF"),
                ("USE_HIDAPI", "ON"),
                ("CMAKE_SKIP_INSTALL_RPATH", "YES"),
            ] {
                cmake_vars.insert(k.to_string(), v.to_string());
            }
            cmake_vars.insert(
                "CMAKE_INSTALL_PREFIX".into(),
                profile.prefix.to_string_lossy().to_string(),
            );
            cmake_vars.insert(
                "CMAKE_INSTALL_LIBDIR".into(),
                profile.prefix.join("lib").to_string_lossy().to_string(),
            );
            cmake_vars
        }),
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
