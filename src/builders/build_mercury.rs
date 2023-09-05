use crate::{
    constants::pkg_data_dir, paths::get_cache_dir, profile::Profile, ui::job_worker::job::WorkerJob,
};

pub fn get_build_mercury_job(profile: &Profile) -> WorkerJob {
    WorkerJob::new_cmd(
        None,
        format!(
            "{sysdata}/scripts/build_mercury.sh",
            sysdata = pkg_data_dir()
        ),
        Some(vec![profile.prefix.clone(), get_cache_dir()]),
    )
}
