use crate::{
    build_tools::git::Git, profile::Profile, termcolor::TermColor, ui::job_worker::job::WorkerJob,
    util::file_utils::rm_rf,
};
use std::{collections::VecDeque, path::Path};

pub fn get_build_openhmd_jobs(profile: &Profile, clean_build: bool) -> VecDeque<WorkerJob> {
    let mut jobs = VecDeque::<WorkerJob>::new();
    jobs.push_back(WorkerJob::new_printer(
        "Building OpenHMD...",
        Some(TermColor::Blue),
    ));

    let git = Git {
        repo: profile
            .features
            .openhmd
            .repo
            .as_ref()
            .unwrap_or(&"https://github.com/OpenHMD/OpenHMD".into())
            .clone(),
        dir: profile.features.openhmd.path.as_ref().unwrap().clone(),
        branch: profile
            .features
            .openhmd
            .branch
            .as_ref()
            .unwrap_or(&"master".into())
            .clone(),
    };

    jobs.extend(git.get_pre_build_jobs(profile.pull_on_build));

    let build_dir = profile
        .features
        .openhmd
        .path
        .as_ref()
        .unwrap()
        .join("build");

    if !Path::new(&build_dir).is_dir() || clean_build {
        rm_rf(&build_dir);
        // prepare job
        jobs.push_back(WorkerJob::new_cmd(
            None,
            "meson".into(),
            Some(vec![
                "setup".into(),
                format!("--prefix={}", profile.prefix.to_string_lossy()),
                format!("--libdir={}", profile.prefix.join("lib").to_string_lossy()),
                "--buildtype=debugoptimized".into(),
                "-Ddrivers=deepoon,xgvr,vrtek,external".into(),
                "-Dtests=false".into(),
                build_dir.to_string_lossy().to_string(),
                profile
                    .features
                    .openhmd
                    .path
                    .as_ref()
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
            ]),
        ));
    }
    // build job
    jobs.push_back(WorkerJob::new_cmd(
        None,
        "ninja".into(),
        Some(vec!["-C".into(), build_dir.to_string_lossy().to_string()]),
    ));
    // install job
    jobs.push_back(WorkerJob::new_cmd(
        None,
        "ninja".into(),
        Some(vec![
            "-C".into(),
            build_dir.to_string_lossy().to_string(),
            "install".into(),
        ]),
    ));

    jobs
}
