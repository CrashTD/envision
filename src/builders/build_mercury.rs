use crate::{constants::pkg_data_dir, paths::get_cache_dir, profile::Profile, runner::Runner};

pub fn get_build_mercury_runner(profile: &Profile) -> Runner {
    let args = vec![profile.prefix.clone(), get_cache_dir()];
    Runner::new(
        None,
        format!(
            "{sysdata}/scripts/build_mercury.sh",
            sysdata = pkg_data_dir()
        ),
        args,
    )
}
