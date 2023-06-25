use crate::{constants::PKG_DATA_DIR, profile::Profile, runner::Runner};
use expect_dialog::ExpectDialog;

pub fn get_build_libsurvive_runner(profile: Profile) -> Runner {
    let mut args = vec![
        profile
            .libsurvive_path
            .expect_dialog("Missing libsurvive path for given profile"),
        profile.prefix,
        match profile.pull_on_build {
            true => "1".into(),
            false => "0".into(),
        },
    ];
    if profile.libsurvive_repo.is_some() {
        args.push(profile.libsurvive_repo.unwrap());
    }
    let runner = Runner::new(
        None,
        format!(
            "{sysdata}/scripts/build_libsurvive.sh",
            sysdata = PKG_DATA_DIR
        ),
        args,
    );
    runner
}
