use crate::{constants::APP_NAME, paths::data_opencomposite_path, profile::Profile};
use std::collections::HashMap;

pub fn system_valve_index_profile() -> Profile {
    let mut environment: HashMap<String, String> = HashMap::new();
    environment.insert("XRT_COMPOSITOR_SCALE_PERCENTAGE".into(), "140".into());
    environment.insert("XRT_COMPOSITOR_COMPUTE".into(), "1".into());
    environment.insert("SURVIVE_GLOBALSCENESOLVER".into(), "0".into());
    environment.insert("SURVIVE_TIMECODE_OFFSET_MS".into(), "-6.94".into());
    Profile {
        name: format!("Valve Index (System) - {name} Default", name = APP_NAME),
        opencomposite_path: data_opencomposite_path(),
        monado_path: "".into(),
        libsurvive_path: None,
        basalt_path: None,
        mercury_path: None,
        libsurvive_enabled: false,
        mercury_enabled: false,
        basalt_enabled: false,
        environment,
        prefix: "/usr".into(),
        can_be_built: false,
    }
}
