use crate::{runner::Runner, profile::Profile};

// TODO: link non-system libsurvive
pub fn get_build_monado_runner(profile: Profile) -> Runner {
    let runner = Runner::new(
        None,
        "./scripts/build_monado.sh".into(),
        vec![
            profile.monado_path
        ]
    );
    runner
}
