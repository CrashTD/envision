use crate::{constants::PKG_DATA_DIR, file_utils::get_cache_dir, profile::Profile, runner::Runner};

pub fn get_build_mercury_runner(profile: Profile) -> Runner {
    let args = vec![profile.prefix, get_cache_dir()];
    let runner = Runner::new(
        None,
        format!(
            "{sysdata}/scripts/build_mercury.sh",
            sysdata = PKG_DATA_DIR
        ),
        args,
    );
    runner
}
