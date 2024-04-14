use crate::file_utils::get_reader;
use std::io::{BufRead, Read};

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
        Self::get_from_etc_os_release().or_else(|| Self::get_from_etc_issue())
    }

    fn get_from_etc_os_release() -> Option<Self> {
        Self::get_from_etc_os_release_file("/etc/os-release")
    }

    fn get_from_etc_os_release_file(fp: &str) -> Option<Self> {
        if let Some(mut reader) = get_reader(fp) {
            let mut buf = String::new();
            loop {
                match reader.read_line(&mut buf) {
                    Ok(0) => break,
                    Ok(_) => {
                        if buf.starts_with("NAME=\"") {
                            let name = buf
                                .split("=")
                                .last()
                                .unwrap_or_default()
                                .to_string()
                                .trim_matches('"')
                                .to_lowercase();
                            return Self::name_matcher(&name);
                        }
                    }
                    Err(_) => break,
                }
            }
        }
        None
    }

    fn get_from_etc_issue() -> Option<Self> {
        if let Some(mut reader) = get_reader("/etc/issue") {
            let mut buf = String::new();
            if reader.read_to_string(&mut buf).is_ok() {
                buf = buf.trim().to_lowercase();
            }
        }

        None
    }

    fn name_matcher(s: &str) -> Option<Self> {
        if s.contains("arch linux")
            || s.contains("manjaro")
            || s.contains("steamos")
            || s.contains("steam os")
        {
            return Some(Self::Arch);
        }
        if s.contains("debian")
            || s.contains("ubuntu")
            || s.contains("mint")
            || s.contains("elementary")
            || s.contains("pop")
        {
            return Some(Self::Debian);
        }
        if s.contains("fedora") || s.contains("nobara") {
            return Some(Self::Fedora);
        }
        if s.contains("gentoo") {
            return Some(Self::Gentoo);
        }
        if s.contains("alpine") || s.contains("postmarket") {
            return Some(Self::Alpine);
        }
        // TODO: detect suse, sles, rhel, nix

        None
    }

    pub fn install_command(&self, packages: &[String]) -> String {
        match self {
            Self::Arch => format!("sudo pacman -Syu {}", packages.join(" ")),
            Self::Alpine => format!("sudo apk add {}", packages.join(" ")),
            Self::Debian => format!("sudo apt install {}", packages.join(" ")),
            Self::Fedora => format!("sudo dnf install {}", packages.join(" ")),
            Self::Gentoo => format!("sudo emerge -av {}", packages.join(" ")),
            Self::Suse => format!("sudo zypper install {}", packages.join(" ")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LinuxDistro;

    #[test]
    fn can_detect_arch_linux_from_etc_os_release() {
        assert_eq!(
            LinuxDistro::get_from_etc_os_release_file("./test/files/archlinux-os-release"),
            Some(LinuxDistro::Arch)
        )
    }
}
