use crate::{profile::Profile, runner::Runner};
use expect_dialog::ExpectDialog;

pub fn get_build_libsurvive_runner(profile: Profile) -> Runner {
    let runner = Runner::new(
        None,
        "./scripts/build_libsurvive.sh".into(),
        vec![profile
            .libsurvive_path
            .expect_dialog("Missing libsurvive path for given profile")],
    );
    runner
}
