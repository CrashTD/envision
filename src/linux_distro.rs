use std::io::Read;

use crate::file_utils::get_reader;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum LinuxDistro {
    Alpine,
    Arch,
    Debian,
    Fedora,
}

pub fn get_distro() -> Option<LinuxDistro> {
    if let Some(mut reader) = get_reader("/etc/issue") {
        let mut buf = String::default();
        if reader.read_to_string(&mut buf).is_ok() {
            buf = buf.trim().to_lowercase();
            if buf.contains("arch linux")
                || buf.contains("manjaro")
                || buf.contains("steamos")
                || buf.contains("steam os")
            {
                return Some(LinuxDistro::Arch);
            }
            if buf.contains("debian")
                || buf.contains("ubuntu")
                || buf.contains("mint")
                || buf.contains("elementary")
            {
                return Some(LinuxDistro::Debian);
            }
            if buf.contains("fedora") || buf.contains("nobara") {
                return Some(LinuxDistro::Fedora);
            }
            if buf.contains("alpine") || buf.contains("postmarket") {
                return Some(LinuxDistro::Alpine);
            }
        }
    }

    None
}
