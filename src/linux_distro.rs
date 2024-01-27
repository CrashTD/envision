use crate::file_utils::get_reader;
use std::io::Read;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum LinuxDistro {
    Alpine,
    Arch,
    Debian,
    Fedora,
    Gentoo,
    // TODO: add Nix,
    Suse,
}

impl LinuxDistro {
    pub fn get() -> Option<Self> {
        if let Some(mut reader) = get_reader("/etc/issue") {
            let mut buf = String::default();
            if reader.read_to_string(&mut buf).is_ok() {
                buf = buf.trim().to_lowercase();
                if buf.contains("arch linux")
                    || buf.contains("manjaro")
                    || buf.contains("steamos")
                    || buf.contains("steam os")
                {
                    return Some(Self::Arch);
                }
                if buf.contains("debian")
                    || buf.contains("ubuntu")
                    || buf.contains("mint")
                    || buf.contains("elementary")
                    || buf.contains("pop")
                {
                    return Some(Self::Debian);
                }
                if buf.contains("fedora") || buf.contains("nobara") {
                    return Some(Self::Fedora);
                }
                if buf.contains("gentoo") {
                    return Some(Self::Gentoo);
                }
                if buf.contains("alpine") || buf.contains("postmarket") {
                    return Some(Self::Alpine);
                }

                // TODO: detect suse, sles, rhel, nix
            }
        }

        None
    }

    pub fn install_command(&self, packages: &[String]) -> String {
        match self {
            Self::Arch => format!("sudo pacman -Syu {}", packages.join(" ")),
            Self::Alpine => format!("sudo apk add {}", packages.join(" ")),
            Self::Debian => format!("sudo apt install {}", packages.join(" ")),
            Self::Fedora => format!("sudo dnf install {}", packages.join(" ")),
            Self::Gentoo => format!("sudo emerge {}", packages.join(" ")),
            Self::Suse => format!("sudo zypper install {}", packages.join(" ")),
        }
    }
}
