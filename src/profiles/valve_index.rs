use crate::{constants::APP_NAME, file_utils::get_data_dir, profile::Profile};
use std::collections::HashMap;

pub fn valve_index_profile() -> Profile {
    let data_dir = get_data_dir();
    let prefix = format!("{data}/prefixes/valve_index_default", data = data_dir);
    let mut environment: HashMap<String, String> = HashMap::new();
    environment.insert("XRT_COMPOSITOR_SCALE_PERCENTAGE".into(), "140".into());
    environment.insert("XRT_COMPOSITOR_COMPUTE".into(), "1".into());
    environment.insert("SURVIVE_GLOBALSCENESOLVER".into(), "0".into());
    environment.insert("SURVIVE_TIMECODE_OFFSET_MS".into(), "-6.94".into());
    environment.insert("LD_LIBRARY_PATH".into(), format!("{pfx}/lib", pfx = prefix));
    Profile {
        name: format!("Valve Index - {name} Default", name = APP_NAME),
        monado_path: format!("{data}/monado", data = data_dir),
        opencomposite_path: format!("{data}/opencomposite", data = data_dir),
        libsurvive_path: Some(format!("{data}/libsurvive", data = data_dir)),
        basalt_path: None,
        mercury_path: None,
        libsurvive_enabled: true,
        basalt_enabled: false,
        mercury_enabled: false,
        environment,
        prefix,
    }
}
