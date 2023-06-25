use crate::{constants::PKG_DATA_DIR, profile::Profile, runner::Runner};

pub fn get_build_opencomposite_runner(profile: Profile) -> Runner {
    let mut args = vec![
        profile.opencomposite_path,
        match profile.pull_on_build {
            true => "1".into(),
            false => "0".into(),
        },
    ];
    if profile.opencomposite_repo.is_some() {
        args.push(profile.opencomposite_repo.unwrap());
    }
    let runner = Runner::new(
        None,
        format!(
            "{sysdata}/scripts/build_opencomposite.sh",
            sysdata = PKG_DATA_DIR
        ),
        args,
    );
    runner
}
