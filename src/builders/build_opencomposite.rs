use crate::{profile::Profile, runner::Runner, constants::PKG_DATA_DIR};

pub fn get_build_opencomposite_runner(profile: Profile) -> Runner {
    let runner = Runner::new(
        None,
        format!("{sysdata}/scripts/build_opencomposite.sh", sysdata = PKG_DATA_DIR),
        vec![
            profile.opencomposite_path,
        ]
    );
    runner
}
