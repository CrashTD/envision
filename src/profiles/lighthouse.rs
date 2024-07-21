use crate::{
    constants::APP_NAME,
    paths::{data_monado_path, data_opencomposite_path, get_data_dir},
    profile::{prepare_ld_library_path, LighthouseDriver, Profile, ProfileFeatures, XRServiceType},
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
        name: format!("Lighthouse Driver - {name} Default", name = APP_NAME),
        xrservice_path: data_monado_path(),
        xrservice_type: XRServiceType::Monado,
        opencomposite_path: data_opencomposite_path(),
        features: ProfileFeatures::default(),
        environment,
        prefix,
        can_be_built: true,
        editable: false,
        lighthouse_driver: LighthouseDriver::SteamVR,
        ..Default::default()
    }
}
