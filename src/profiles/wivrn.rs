use crate::{
    constants::APP_NAME,
    paths::{data_opencomposite_path, data_wivrn_path, get_data_dir},
    profile::{
        prepare_ld_library_path, Profile, ProfileFeatures, ProfileOvrCompatibilityModule,
        XRServiceType,
    },
    ui::job_worker::internal_worker::LAUNCH_OPTS_CMD_PLACEHOLDER,
};
use std::collections::HashMap;

pub fn wivrn_profile() -> Profile {
    let data_dir = get_data_dir();
    let prefix = data_dir.join("prefixes/wivrn_default");
    let mut environment: HashMap<String, String> = HashMap::new();
    environment.insert("LD_LIBRARY_PATH".into(), prepare_ld_library_path(&prefix));
    environment.insert("XRT_DEBUG_GUI".into(), "1".into());
    environment.insert("XRT_CURATED_GUI".into(), "1".into());
    environment.insert("U_PACING_APP_USE_MIN_FRAME_PERIOD".into(), "1".into());
    Profile {
        uuid: "wivrn-default".into(),
        name: format!("WiVRn - {name} Default", name = APP_NAME),
        xrservice_path: data_wivrn_path(),
        xrservice_type: XRServiceType::Wivrn,
        ovr_comp: ProfileOvrCompatibilityModule {
            path: data_opencomposite_path(),
            ..Default::default()
        },
        features: ProfileFeatures {
            ..Default::default()
        },
        xrservice_launch_options: format!(
            "{LAUNCH_OPTS_CMD_PLACEHOLDER} --no-instructions --no-manage-active-runtime"
        ),
        environment,
        prefix,
        can_be_built: true,
        editable: false,
        ..Default::default()
    }
}
