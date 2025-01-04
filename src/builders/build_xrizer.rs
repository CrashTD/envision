use crate::{
    build_tools::git::Git, profile::Profile, termcolor::TermColor, ui::job_worker::job::WorkerJob,
    util::file_utils::rm_rf,
};
use std::{collections::VecDeque, path::Path};

pub fn get_build_xrizer_jobs(profile: &Profile, clean_build: bool) -> VecDeque<WorkerJob> {
    let mut jobs = VecDeque::<WorkerJob>::new();
    jobs.push_back(WorkerJob::new_printer(
        "Building xrizer...",
        Some(TermColor::Blue),
    ));

    let git = Git {
        repo: profile
            .ovr_comp
            .repo
            .as_ref()
            .unwrap_or(&"https://github.com/Supreeeme/xrizer".into())
            .clone(),
        dir: profile.ovr_comp.path.clone(),
        branch: profile
            .ovr_comp
            .branch
            .as_ref()
            .unwrap_or(&"main".into())
            .clone(),
    };

    jobs.extend(git.get_pre_build_jobs(profile.pull_on_build));

    let build_dir = profile.ovr_comp.path.join("target");
    if !Path::new(&build_dir).is_dir() || clean_build {
        rm_rf(&build_dir);
    }

    jobs.push_back(WorkerJob::new_cmd(
        None,
        "sh".into(),
        Some(vec![
            "-c".into(),
            format!(
                "cd '{}' && cargo xbuild --release",
                profile.ovr_comp.path.to_string_lossy()
            ),
        ]),
    ));

    jobs
}
