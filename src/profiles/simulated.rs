use crate::{
    constants::APP_NAME,
    paths::{data_monado_path, data_opencomposite_path, get_data_dir},
    profile::{Profile, ProfileFeatures, ProfileOvrCompatibilityModule, XRServiceType},
};
use std::collections::HashMap;

pub fn simulated_profile() -> Profile {
    let data_dir = get_data_dir();
    let prefix = data_dir.join("prefixes/simulated_default");
    let mut environment: HashMap<String, String> = HashMap::new();
    environment.insert("QWERTY_ENABLE".into(), "1".into());
    environment.insert("XRT_JSON_LOG".into(), "1".into());
    environment.insert("XRT_COMPOSITOR_SCALE_PERCENTAGE".into(), "140".into());
    environment.insert("XRT_COMPOSITOR_COMPUTE".into(), "1".into());
    environment.insert("XRT_DEBUG_GUI".into(), "1".into());
    environment.insert("XRT_CURATED_GUI".into(), "1".into());
    environment.insert("U_PACING_APP_USE_MIN_FRAME_PERIOD".into(), "1".into());
    environment.insert(
        "LD_LIBRARY_PATH".into(),
        format!("{pfx}/lib:{pfx}/lib64", pfx = prefix.to_string_lossy()),
    );
    Profile {
        uuid: "simulated-default".into(),
        name: format!("Simulated Driver - {APP_NAME} Default"),
        xrservice_path: data_monado_path(),
        xrservice_type: XRServiceType::Monado,
        ovr_comp: ProfileOvrCompatibilityModule {
            path: data_opencomposite_path(),
            ..Default::default()
        },
        features: ProfileFeatures::default(),
        environment,
        prefix,
        can_be_built: true,
        editable: false,
        ..Default::default()
    }
}
