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

pub fn get_build_monado_jobs(
    profile: &Profile,
    clean_build: bool,
    debug_build: bool,
) -> VecDeque<WorkerJob> {
    let mut jobs = VecDeque::<WorkerJob>::new();

    let git = Git {
        repo: match profile.xrservice_repo.as_ref() {
            Some(r) => r.clone(),
            None => "https://gitlab.freedesktop.org/monado/monado".into(),
        },
        dir: profile.xrservice_path.clone(),
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

    let build_dir = format!("{}/build", profile.xrservice_path);
    let mut env: HashMap<String, String> = HashMap::new();
    env.insert(
        "PKG_CONFIG_PATH".into(),
        format!("{}/lib/pkgconfig", profile.prefix),
    );
    let mut cmake_vars: HashMap<String, String> = HashMap::new();
    cmake_vars.insert(
        "CMAKE_BUILD_TYPE".into(),
        (if debug_build { "Debug" } else { "Release" }).into(),
    );
    cmake_vars.insert("XRT_HAVE_SYSTEM_CJSON".into(), "NO".into());
    cmake_vars.insert("CMAKE_LIBDIR".into(), format!("{}/lib", profile.prefix));
    cmake_vars.insert("CMAKE_INSTALL_PREFIX".into(), profile.prefix.clone());
    cmake_vars.insert(
        "CMAKE_C_FLAGS".into(),
        format!(
            "-Wl,-rpath='{}/lib'{}",
            profile.prefix,
            if debug_build {
                " -g -march=native -O3 -fno-omit-frame-pointer"
            } else {
                ""
            }
        ),
    );
    cmake_vars.insert(
        "CMAKE_CXX_FLAGS".into(),
        format!(
            "-Wl,-rpath='{}/lib'{}",
            profile.prefix,
            if debug_build {
                " -g -march=native -O3 -fno-omit-frame-pointer"
            } else {
                ""
            }
        ),
    );
    profile.xrservice_cmake_flags.iter().for_each(|(k, v)| {
        if k == "CMAKE_C_FLAGS" || k == "CMAKE_CXX_FLAGS" {
            cmake_vars.insert(k.clone(), format!("{} {}", cmake_vars.get(k).unwrap(), v));
        } else {
            cmake_vars.insert(k.clone(), v.clone());
        }
    });

    let cmake = Cmake {
        env: Some(env),
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
