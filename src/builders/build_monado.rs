use crate::{runner::Runner, profile::Profile, constants::PKG_DATA_DIR};

pub fn get_build_monado_runner(profile: Profile) -> Runner {
    let runner = Runner::new(
        None,
        format!("{sysdata}/scripts/build_monado.sh", sysdata = PKG_DATA_DIR),
        vec![
            profile.monado_path,
            profile.prefix,
        ]
    );
    runner
}
