use crate::{
    cmd_runner::CmdRunner, constants::pkg_data_dir, paths::get_cache_dir, profile::Profile,
};

pub fn get_build_mercury_runner(profile: &Profile) -> CmdRunner {
    let args = vec![profile.prefix.clone(), get_cache_dir()];
    CmdRunner::new(
        None,
        format!(
            "{sysdata}/scripts/build_mercury.sh",
            sysdata = pkg_data_dir()
        ),
        args,
    )
}
