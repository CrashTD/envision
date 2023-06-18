use expect_dialog::ExpectDialog;

use crate::{constants::PKG_DATA_DIR, profile::Profile, runner::Runner};

pub fn get_build_basalt_runner(profile: Profile) -> Runner {
    let runner = Runner::new(
        None,
        format!("{sysdata}/scripts/build_basalt.sh", sysdata = PKG_DATA_DIR),
        vec![
            profile
                .basalt_path
                .expect_dialog("Missing basalt path for given profile"),
            profile.prefix,
        ],
    );
    runner
}
