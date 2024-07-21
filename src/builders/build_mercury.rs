use crate::{
    constants::pkg_data_dir, paths::get_cache_dir, profile::Profile, ui::job_worker::job::WorkerJob,
};

pub fn get_build_mercury_job(profile: &Profile) -> WorkerJob {
    WorkerJob::new_cmd(
        None,
        pkg_data_dir()
            .join("scripts/build_mercury.sh")
            .to_string_lossy()
            .to_string(),
        Some(vec![
            profile.prefix.to_string_lossy().to_string(),
            get_cache_dir().to_string_lossy().to_string(),
        ]),
    )
}
