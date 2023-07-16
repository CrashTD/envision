use crate::{constants::pkg_data_dir, profile::Profile, runner::Runner};

pub fn get_build_basalt_runner(profile: &Profile) -> Runner {
    let mut args = vec![
        profile
            .features.basalt.path.clone()
            .expect("Missing basalt path for given profile"),
        profile.prefix.clone(),
        match profile.pull_on_build {
            true => "1".into(),
            false => "0".into(),
        },
    ];
    if profile.features.basalt.repo.is_some() {
        args.push(profile.features.basalt.repo.as_ref().unwrap().clone());
    }
    let runner = Runner::new(
        None,
        format!("{sysdata}/scripts/build_basalt.sh", sysdata = pkg_data_dir()),
        args,
    );
    runner
}
