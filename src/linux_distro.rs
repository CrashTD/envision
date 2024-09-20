use crate::util::file_utils::get_reader;
use std::{
    fmt::Display,
    io::{BufRead, Read},
    path::Path,
};

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

impl Display for LinuxDistro {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Alpine => "Alpine",
            Self::Arch => "Arch",
            Self::Debian => "Debian",
            Self::Fedora => "Fedora",
            Self::Gentoo => "Gentoo",
            Self::Suse => "Suse",
        })
    }
}

impl LinuxDistro {
    pub fn get() -> Option<Self> {
        Self::get_from_etc_os_release().or_else(Self::get_from_etc_issue)
    }

    fn get_from_etc_os_release() -> Option<Self> {
        Self::get_from_etc_os_release_file(Path::new("/etc/os-release"))
    }

    // does it make sense to be here?
    pub fn get_specific_distro() -> Option<String> {
        let mut name: Option<String> = None;
        if let Some(mut reader) = get_reader(Path::new("/etc/os-release")) {
            let mut buf = String::new();
            loop {
                match reader.read_line(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(_) if buf.starts_with("PRETTY_NAME=\"") => {
                        return buf
                            .split('=')
                            .last()
                            .map(|b| b.trim().trim_matches('"').trim().to_string());
                    }
                    Ok(_) if buf.starts_with("NAME=\"") => {
                        name = buf
                            .split('=')
                            .last()
                            .map(|b| b.trim().trim_matches('"').trim().to_string());
                    }
                    _ => {}
                };
                buf.clear();
            }
        }
        name
    }

    fn get_from_etc_os_release_file(fp: &Path) -> Option<Self> {
        if let Some(mut reader) = get_reader(fp) {
            let mut buf = String::new();
            loop {
                match reader.read_line(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {
                        if buf.starts_with("NAME=\"")
                            || buf.starts_with("ID=\"")
                            || buf.starts_with("ID_LIKE=\"")
                        {
                            let name = buf
                                .split('=')
                                .last()
                                .unwrap_or_default()
                                .trim()
                                .trim_matches('"')
                                .to_lowercase();
                            let res = Self::name_matcher(&name);
                            if res.is_some() {
                                return res;
                            }
                        }
                    }
                }
                buf.clear();
            }
        }
        None
    }

    fn get_from_etc_issue() -> Option<Self> {
        if let Some(mut reader) = get_reader(Path::new("/etc/issue")) {
            let mut buf = String::new();
            if reader.read_to_string(&mut buf).is_ok() {
                buf = buf.trim().to_lowercase();
                return Self::name_matcher(&buf);
            }
        }

        None
    }

    fn name_matcher(s: &str) -> Option<Self> {
        if s.contains("arch")
            || s.contains("manjaro")
            || s.contains("steamos")
            || s.contains("steam os")
            || s.contains("endeavour")
            || s.contains("garuda")
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
        if s.contains("fedora") || s.contains("nobara") || s.contains("ultramarine linux") {
            return Some(Self::Fedora);
        }
        if s.contains("gentoo") {
            return Some(Self::Gentoo);
        }
        if s.contains("alpine") || s.contains("postmarket") {
            return Some(Self::Alpine);
        }
        if s.contains("suse") {
            return Some(Self::Suse);
        }
        // TODO: detect sles, rhel, nix

        None
    }

    pub fn install_command(&self, packages: &[String]) -> String {
        match self {
            Self::Arch => format!("sudo pacman -Syu {}", packages.join(" ")),
            Self::Alpine => format!("sudo apk add {}", packages.join(" ")),
            Self::Debian => format!("sudo apt install {}", packages.join(" ")),
            Self::Gentoo => format!("sudo emerge -av {}", packages.join(" ")),
            Self::Suse => format!("sudo zypper install {}", packages.join(" ")),
            Self::Fedora => {
                let mut install_rpmfusion_cmd: Option<String> = None;
                let mut swap_ffmpeg_cmd: Option<String> = None;

                if packages
                    .iter()
                    .any(|package| ["ffmpeg-devel", "x264-devel"].contains(&package.as_str()))
                {
                    let rpmfusion_free_path = Path::new("/etc/yum.repos.d/rpmfusion-free.repo");
                    if !rpmfusion_free_path.is_file() {
                        install_rpmfusion_cmd = Some(String::from(
                                "sudo dnf install https://download1.rpmfusion.org/free/fedora/rpmfusion-free-release-$(rpm -E %fedora).noarch.rpm"
                                ));
                    }

                    let ffmpeg_free_path = Path::new("/usr/share/doc/ffmpeg-free");
                    if ffmpeg_free_path.is_dir() {
                        swap_ffmpeg_cmd = Some(String::from(
                            "sudo dnf swap ffmpeg-free ffmpeg --allowerasing",
                        ));
                    }
                }

                [
                    install_rpmfusion_cmd,
                    swap_ffmpeg_cmd,
                    Some(format!("sudo dnf install {}", packages.join(" "))),
                ]
                .iter()
                .filter_map(|s| s.clone())
                .collect::<Vec<String>>()
                .join(" && ")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::LinuxDistro;

    #[test]
    fn can_detect_arch_linux_from_etc_os_release() {
        assert_eq!(
            LinuxDistro::get_from_etc_os_release_file(Path::new(
                "./test/files/archlinux-os-release"
            )),
            Some(LinuxDistro::Arch)
        )
    }
}
