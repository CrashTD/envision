use crate::{profile::Profile, runner::Runner, constants::PKG_DATA_DIR};

pub fn get_build_wivrn_runner(profile: Profile) -> Runner {
    let runner = Runner::new(
        None,
        format!("{sysdata}/scripts/build_wivrn.sh", sysdata = PKG_DATA_DIR),
        vec![
            profile.xrservice_path,
            profile.prefix,
        ]
    );
    runner
}

