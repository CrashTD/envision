use crate::{
    constants::APP_NAME,
    paths::{data_opencomposite_path, SYSTEM_PREFIX},
    profile::{LighthouseDriver, Profile, ProfileFeatures, XRServiceType},
};
use std::collections::HashMap;

pub fn system_valve_index_profile() -> Profile {
    let mut environment: HashMap<String, String> = HashMap::new();
    environment.insert("XRT_JSON_LOG".into(), "1".into());
    environment.insert("XRT_COMPOSITOR_SCALE_PERCENTAGE".into(), "140".into());
    environment.insert("XRT_COMPOSITOR_COMPUTE".into(), "1".into());
    environment.insert("SURVIVE_GLOBALSCENESOLVER".into(), "0".into());
    environment.insert("SURVIVE_TIMECODE_OFFSET_MS".into(), "-6.94".into());
    Profile {
        uuid: "system-valve-index-default".into(),
        name: format!("Valve Index (System) - {name} Default", name = APP_NAME),
        opencomposite_path: data_opencomposite_path(),
        xrservice_path: "".into(),
        xrservice_type: XRServiceType::Monado,
        features: ProfileFeatures {
            ..Default::default()
        },
        environment,
        prefix: SYSTEM_PREFIX.into(),
        can_be_built: false,
        editable: false,
        lighthouse_driver: LighthouseDriver::Survive,
        ..Default::default()
    }
}
