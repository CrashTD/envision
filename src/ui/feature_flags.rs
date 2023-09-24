use std::env;
use lazy_static::lazy_static;

fn get_ff_libmonado_device_enumeration_enabled() -> bool {
    env::var("ENVISION_FF_USE_LIBMONADO").unwrap_or_default() == "1"
}

lazy_static! {
    pub static ref FF_LIBMONADO_DEVICE_ENUMERATION_ENABLED: bool = get_ff_libmonado_device_enumeration_enabled();
}
