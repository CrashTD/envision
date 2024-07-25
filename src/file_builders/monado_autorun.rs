use std::path::{Path, PathBuf};

use crate::{
    file_utils::{deserialize_file, get_writer},
    xdg::XDG,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonadoAutorun {
    pub exec: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonadoAutorunConfig {
    #[serde(skip_serializing_if = "Option::is_none", rename = "$schema")]
    _schema: Option<String>,
    pub autoruns: Vec<MonadoAutorun>,
}

impl Default for MonadoAutorunConfig {
    fn default() -> Self {
        Self {
            _schema: Some(
                "https://monado.pages.freedesktop.org/monado/autorun_v0.schema.json".into(),
            ),
            autoruns: vec![],
        }
    }
}

fn get_monado_autorun_config_path() -> PathBuf {
    XDG.get_config_home().join("monado/autorun_v0.json")
}

fn get_monado_autorun_config_from_path(path: &Path) -> Option<MonadoAutorunConfig> {
    deserialize_file(path)
}

pub fn get_monado_autorun_config() -> MonadoAutorunConfig {
    get_monado_autorun_config_from_path(&get_monado_autorun_config_path()).unwrap_or_default()
}

fn dump_monado_autorun_config_to_path(config: &MonadoAutorunConfig, path: &Path) {
    let writer = get_writer(path).expect("Unable to save Monado Autorun config");
    serde_json::to_writer_pretty(writer, config).expect("Unable to save Monado Autorun config");
}

pub fn dump_monado_autorun_config(config: &MonadoAutorunConfig) {
    dump_monado_autorun_config_to_path(config, &get_monado_autorun_config_path());
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::get_monado_autorun_config_from_path;

    #[test]
    fn can_read_monado_autorun_config() {
        let conf = get_monado_autorun_config_from_path(Path::new(
            "./test/files/monado_autorun_config.json",
        ))
        .expect("Could not find monado autorun config");
        assert_eq!(
            conf._schema,
            Some("https://monado.pages.freedesktop.org/monado/autorun_v0.schema.json".into())
        );
        assert_eq!(conf.autoruns.len(), 1);
        assert_eq!(conf.autoruns.first().unwrap().exec, "foobar");
        assert_eq!(conf.autoruns.first().unwrap().args.len(), 2);
        assert_eq!(conf.autoruns.first().unwrap().args.first().unwrap(), "bar");
        assert_eq!(conf.autoruns.first().unwrap().args.get(1).unwrap(), "baz");
    }
}
