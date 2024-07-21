use lazy_static::lazy_static;
use xdg::BaseDirectories;

lazy_static! {
    pub static ref XDG: BaseDirectories =
        BaseDirectories::new().expect("Failed to get XDG base directories");
}
