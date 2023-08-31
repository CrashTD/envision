use crate::{
    constants::APP_NAME,
    paths::{data_opencomposite_path, data_wivrn_path, get_data_dir},
    profile::{Profile, ProfileFeatures, XRServiceType},
};
use std::collections::HashMap;

pub fn wivrn_profile() -> Profile {
    let data_dir = get_data_dir();
    let prefix = format!("{data}/prefixes/wivrn_default", data = data_dir);
    let mut environment: HashMap<String, String> = HashMap::new();
    environment.insert(
        "LD_LIBRARY_PATH".into(),
        format!("{pfx}/lib:{pfx}/lib64", pfx = prefix),
    );
    environment.insert("XRT_DEBUG_GUI".into(), "0".into());
    environment.insert("XRT_CURATED_GUI".into(), "1".into());
    Profile {
        uuid: "wivrn-default".into(),
        name: format!("WiVRn - {name} Default", name = APP_NAME),
        xrservice_path: data_wivrn_path(),
        xrservice_type: XRServiceType::Wivrn,
        opencomposite_path: data_opencomposite_path(),
        features: ProfileFeatures {
            ..Default::default()
        },
        environment,
        prefix,
        can_be_built: true,
        editable: false,
        ..Default::default()
    }
}
