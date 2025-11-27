use crate::{
    build_tools::{cmake::Cmake, git::Git},
    profile::Profile,
    termcolor::TermColor,
    ui::job_worker::job::WorkerJob,
    util::{
        file_utils::rm_rf,
        git_repos::{DEFAULT_WIVRN_BRANCH, DEFAULT_WIVRN_REPO},
    },
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
            .unwrap_or(&DEFAULT_WIVRN_REPO.into())
            .clone(),
        dir: profile.xrservice_path.clone(),
        branch: profile
            .xrservice_branch
            .as_ref()
            .unwrap_or(&DEFAULT_WIVRN_BRANCH.into())
            .clone(),
    };

    jobs.extend(git.get_pre_build_jobs(profile.pull_on_build));

    let build_dir = profile.xrservice_path.join("build");

    let cmake = Cmake {
        env: None,
        vars: Some({
            let mut cmake_vars: HashMap<String, String> = HashMap::new();
            for (k, v) in [
                ("CMAKE_EXPORT_COMPILE_COMMANDS", "ON"),
                ("CMAKE_BUILD_TYPE", "RelWithDebInfo"),
                ("XRT_HAVE_SYSTEM_CJSON", "NO"),
                ("WIVRN_BUILD_CLIENT", "OFF"),
                ("WIVRN_BUILD_WIVRNCTL", "OFF"),
                ("WIVRN_USE_SYSTEM_BOOST", "OFF"),
            ] {
                cmake_vars.insert(k.to_string(), v.to_string());
            }
            cmake_vars.insert(
                "CMAKE_INSTALL_PREFIX".into(),
                profile.prefix.to_string_lossy().to_string(),
            );
            profile.xrservice_cmake_flags.iter().for_each(|(k, v)| {
                cmake_vars.insert(k.clone(), v.clone());
            });
            cmake_vars
        }),
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
