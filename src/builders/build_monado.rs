use crate::{constants::pkg_data_dir, profile::Profile, runner::Runner};

pub fn get_build_monado_runner(profile: Profile) -> Runner {
    let mut args = vec![
        profile.xrservice_path,
        profile.prefix,
        match profile.pull_on_build {
            true => "1".into(),
            false => "0".into(),
        },
    ];
    if profile.xrservice_repo.is_some() {
        args.push(profile.xrservice_repo.unwrap());
    }
    let runner = Runner::new(
        None,
        format!("{sysdata}/scripts/build_monado.sh", sysdata = pkg_data_dir()),
        args,
    );
    runner
}
