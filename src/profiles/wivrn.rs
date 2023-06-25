use std::collections::HashMap;

use crate::{file_utils::get_data_dir, profile::{Profile, XRServiceType}, paths::{data_wivrn_path, data_opencomposite_path}, constants::APP_NAME};

pub fn wivrn_profile() -> Profile {
    let data_dir = get_data_dir();
    let prefix = format!("{data}/prefixes/wivrn_default", data = data_dir);
    let mut environment: HashMap<String, String> = HashMap::new();
    environment.insert("LD_LIBRARY_PATH".into(), format!("{pfx}/lib", pfx = prefix));
    Profile {
        uuid: "wivrn-default".into(),
        name: format!("WiVRn - {name} Default", name = APP_NAME),
        xrservice_path: data_wivrn_path(),
        xrservice_type: XRServiceType::Wivrn,
        opencomposite_path: data_opencomposite_path(),
        libsurvive_path: None,
        basalt_path: None,
        mercury_path: None,
        libsurvive_enabled: false,
        basalt_enabled: false,
        mercury_enabled: false,
        environment,
        prefix,
        can_be_built: true,
        editable: false,
        ..Default::default()
    }
}
