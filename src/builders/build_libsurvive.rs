use crate::{profile::Profile, runner::Runner, constants::PKG_DATA_DIR};
use expect_dialog::ExpectDialog;

pub fn get_build_libsurvive_runner(profile: Profile) -> Runner {
    let runner = Runner::new(
        None,
        format!("{sysdata}/scripts/build_libsurvive.sh", sysdata = PKG_DATA_DIR),
        vec![
            profile
                .libsurvive_path
                .expect_dialog("Missing libsurvive path for given profile"),
            profile.prefix,
        ],
    );
    runner
}
