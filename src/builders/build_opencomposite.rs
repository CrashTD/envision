use crate::{profile::Profile, runner::Runner};

pub fn get_build_opencomposite_runner(profile: Profile) -> Runner {
    let runner = Runner::new(
        None,
        "./scripts/build_opencomposite.sh".into(),
        vec![
            profile.opencomposite_path,
        ]
    );
    runner
}
