use expect_dialog::ExpectDialog;
use crate::{constants::PKG_DATA_DIR, profile::Profile, runner::Runner};

pub fn get_build_basalt_runner(profile: Profile) -> Runner {
    let mut args = vec![
        profile
            .features.basalt.path
            .expect_dialog("Missing basalt path for given profile"),
        profile.prefix,
        match profile.pull_on_build {
            true => "1".into(),
            false => "0".into(),
        },
    ];
    if profile.features.basalt.repo.is_some() {
        args.push(profile.features.basalt.repo.unwrap());
    }
    let runner = Runner::new(
        None,
        format!("{sysdata}/scripts/build_basalt.sh", sysdata = PKG_DATA_DIR),
        args,
    );
    runner
}
