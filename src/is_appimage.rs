use lazy_static::lazy_static;
use std::env;

fn is_appimage() -> bool {
    env::var("APPIMAGE").is_ok_and(|s| !s.trim().is_empty())
}

lazy_static! {
    pub static ref IS_APPIMAGE: bool = is_appimage();
}
