use freedesktop_desktop_entry::DesktopEntry;
use std::path::Path;
use tracing::warn;

#[derive(Debug, Clone)]
pub struct XRPlugin {
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    /// this is either `X-XR-Plugin-Exec` or `Exec`, in this order
    pub exec: String,
    /// the path to the .desktop file
    pub path: String,
}

impl XRPlugin {
    pub fn from_desktop_file(p: &Path) -> Option<Self> {
        let p = p.canonicalize().ok()?;
        if !p.is_file() {
            return None;
        }

        match DesktopEntry::from_path(&p, None as Option<&[&str]>) {
            Err(e) => {
                warn!(
                    "failed to parse desktop file `{}`: {e:?}",
                    p.to_string_lossy()
                );
                None
            }
            Ok(entry) => {
                let dg = entry.groups.desktop_entry()?;

                if dg.entry_bool("X-XR-Plugin") != Some(true) {
                    return None;
                }

                Some(Self {
                    name: entry.name(&[] as &[&str])?.to_string(),
                    exec: dg
                        .entry("X-XR-Plugin-Exec")
                        .or_else(|| entry.exec())?
                        .into(),
                    icon: entry.icon().map(String::from),
                    description: entry.comment(&[] as &[&str]).map(String::from),
                    path: p.to_string_lossy().to_string(),
                })
            }
        }
    }

    pub fn find_plugins() -> Vec<Self> {
        freedesktop_desktop_entry::Iter::new(freedesktop_desktop_entry::default_paths())
            .filter_map(|p| Self::from_desktop_file(&p))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::plugins::XRPlugin;
    use std::path::PathBuf;

    #[test]
    fn rejects_non_xr_destkop_file() {
        assert!(XRPlugin::from_desktop_file(&PathBuf::from("./test/files/nonxr.desktop")).is_none())
    }

    #[test]
    fn parses_xr_destkop_file() {
        let p =
            XRPlugin::from_desktop_file(&PathBuf::from("./test/files/xrplugin.desktop")).unwrap();
        assert_eq!(p.name, "xr plugin");
        assert_eq!(p.exec, "foo -bar");
    }

    #[test]
    fn parses_xr_destkop_file_with_xr_exec_precedence() {
        let p = XRPlugin::from_desktop_file(&PathBuf::from(
            "./test/files/xrplugin_with_xr_exec.desktop",
        ))
        .unwrap();
        assert_eq!(p.name, "xr plugin");
        assert_eq!(p.exec, "foo -baz");
    }
}
