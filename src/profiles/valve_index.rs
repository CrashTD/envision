use crate::{
    constants::APP_NAME,
    paths::{data_libsurvive_path, data_monado_path, data_opencomposite_path, get_data_dir},
    profile::{Profile, ProfileFeature, ProfileFeatureType, ProfileFeatures, XRServiceType},
};
use std::collections::HashMap;

pub fn valve_index_profile() -> Profile {
    let data_dir = get_data_dir();
    let prefix = format!("{data}/prefixes/valve_index_default", data = data_dir);
    let mut environment: HashMap<String, String> = HashMap::new();
    environment.insert("XRT_JSON_LOG".into(), "1".into());
    environment.insert("XRT_COMPOSITOR_SCALE_PERCENTAGE".into(), "140".into());
    environment.insert("XRT_COMPOSITOR_COMPUTE".into(), "1".into());
    environment.insert("SURVIVE_GLOBALSCENESOLVER".into(), "0".into());
    environment.insert("SURVIVE_TIMECODE_OFFSET_MS".into(), "-6.94".into());
    environment.insert(
        "LD_LIBRARY_PATH".into(),
        format!("{pfx}/lib:{pfx}/lib64", pfx = prefix),
    );
    Profile {
        uuid: "valve-index-default".into(),
        name: format!("Valve Index - {name} Default", name = APP_NAME),
        xrservice_path: data_monado_path(),
        xrservice_type: XRServiceType::Monado,
        opencomposite_path: data_opencomposite_path(),
        features: ProfileFeatures {
            libsurvive: ProfileFeature {
                feature_type: ProfileFeatureType::Libsurvive,
                enabled: true,
                path: Some(data_libsurvive_path()),
                repo: None,
            },
            ..Default::default()
        },
        environment,
        prefix,
        can_be_built: true,
        editable: false,
        ..Default::default()
    }
}
