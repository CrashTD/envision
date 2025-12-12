use crate::{
    constants::APP_NAME,
    paths::{data_monado_path, data_ovr_comp_path, get_data_dir},
    profile::{
        LighthouseDriver, Profile, ProfileFeatures, ProfileOvrCompatibilityModule, XRServiceType,
        prepare_ld_library_path,
    },
};
use std::collections::HashMap;

pub fn lighthouse_profile() -> Profile {
    let data_dir = get_data_dir();
    let prefix = data_dir.join("prefixes/lighthouse_default");
    let mut environment: HashMap<String, String> = HashMap::new();
    environment.insert("XRT_JSON_LOG".into(), "1".into());
    environment.insert("XRT_COMPOSITOR_SCALE_PERCENTAGE".into(), "140".into());
    environment.insert("XRT_COMPOSITOR_COMPUTE".into(), "1".into());
    environment.insert("XRT_DEBUG_GUI".into(), "1".into());
    environment.insert("XRT_CURATED_GUI".into(), "1".into());
    environment.insert("U_PACING_APP_USE_MIN_FRAME_PERIOD".into(), "1".into());
    environment.insert("LD_LIBRARY_PATH".into(), prepare_ld_library_path(&prefix));
    Profile {
        uuid: "lighthouse-default".into(),
        name: format!("Lighthouse Driver - {APP_NAME} Default"),
        xrservice_path: data_monado_path(),
        xrservice_type: XRServiceType::Monado,
        ovr_comp: ProfileOvrCompatibilityModule {
            path: data_ovr_comp_path(),
            ..Default::default()
        },
        features: ProfileFeatures::default(),
        environment,
        prefix,
        is_system_profile: false,
        editable: false,
        lighthouse_driver: LighthouseDriver::SteamVR,
        ..Default::default()
    }
}
