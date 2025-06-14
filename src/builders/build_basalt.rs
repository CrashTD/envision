use crate::{
    build_tools::{cmake::Cmake, git::Git},
    profile::Profile,
    termcolor::TermColor,
    ui::job_worker::job::WorkerJob,
    util::file_utils::rm_rf,
};
use std::{
    collections::{HashMap, VecDeque},
    num::NonZero,
};

pub fn get_build_basalt_jobs(profile: &Profile, clean_build: bool) -> VecDeque<WorkerJob> {
    let mut jobs = VecDeque::<WorkerJob>::new();
    jobs.push_back(WorkerJob::new_printer(
        "Building Basalt...",
        Some(TermColor::Blue),
    ));

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

    let cmake = Cmake {
        env: Some({
            let mut cmake_env: HashMap<String, String> = HashMap::new();
            for (k, v) in [
                // The basalt build uses a lot of RAM, so we have to limit the number of
                // build processes to not starve the system of memory
                // Limit to 6 build processes at most
                (
                    "CMAKE_BUILD_PARALLEL_LEVEL",
                    std::cmp::min(
                        6,
                        std::thread::available_parallelism()
                            .map(NonZero::get)
                            .unwrap_or(2),
                    )
                    .to_string(),
                ),
                ("CMAKE_BUILD_TYPE", "RelWithDebInfo".into()),
                ("CMAKE_POLICY_VERSION_MINIMUM", "3.5".into()),
                ("BUILD_TESTS", "off".into()),
            ] {
                cmake_env.insert(k.to_string(), v);
            }
            cmake_env
        }),
        vars: Some({
            let mut cmake_vars: HashMap<String, String> = HashMap::new();
            for (k, v) in [
                ("CMAKE_EXPORT_COMPILE_COMMANDS", "ON"),
                ("CMAKE_BUILD_TYPE", "RelWithDebInfo"),
                ("BUILD_TESTS", "OFF"),
                ("BASALT_INSTANTIATIONS_DOUBLE", "OFF"),
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
