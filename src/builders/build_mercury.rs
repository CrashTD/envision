use std::collections::VecDeque;

use crate::{constants::pkg_data_dir, termcolor::TermColor, ui::job_worker::job::WorkerJob};

pub fn get_build_mercury_jobs() -> VecDeque<WorkerJob> {
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
        None,
    ));

    jobs
}
