use std::collections::VecDeque;

use crate::{
    constants::pkg_data_dir, paths::get_cache_dir, profile::Profile, termcolor::TermColor,
    ui::job_worker::job::WorkerJob,
};

pub fn get_build_mercury_jobs(profile: &Profile) -> VecDeque<WorkerJob> {
    let mut jobs = VecDeque::new();
    jobs.push_back(WorkerJob::new_printer(
        "Building Mercury...",
        Some(TermColor::Blue),
    ));
    jobs.push_back(WorkerJob::new_cmd(
        None,
        pkg_data_dir()
            .join("scripts/build_mercury.sh")
            .to_string_lossy()
            .to_string(),
        Some(vec![
            profile.prefix.to_string_lossy().to_string(),
            get_cache_dir().to_string_lossy().to_string(),
        ]),
    ));

    jobs
}
