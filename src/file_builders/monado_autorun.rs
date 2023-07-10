use expect_dialog::ExpectDialog;
use serde::{Deserialize, Serialize};

use crate::file_utils::{deserialize_file, get_writer, get_xdg_config_dir};

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

fn get_monado_autorun_config_path() -> String {
    format!(
        "{config}/monado/autorun_v0.json",
        config = get_xdg_config_dir()
    )
}

fn get_monado_autorun_config_from_path(path_s: &String) -> Option<MonadoAutorunConfig> {
    deserialize_file(path_s)
}

pub fn get_monado_autorun_config() -> MonadoAutorunConfig {
    get_monado_autorun_config_from_path(&get_monado_autorun_config_path())
        .unwrap_or(MonadoAutorunConfig::default())
}

fn dump_monado_autorun_config_to_path(config: &MonadoAutorunConfig, path_s: &String) {
    let writer = get_writer(path_s);
    serde_json::to_writer_pretty(writer, config)
        .expect_dialog("Unable to save Monado Autorun config");
}

pub fn dump_monado_autorun_config(config: &MonadoAutorunConfig) {
    dump_monado_autorun_config_to_path(config, &get_monado_autorun_config_path());
}

#[cfg(test)]
mod tests {
    use super::get_monado_autorun_config_from_path;

    #[test]
    fn can_read_monado_autorun_config() {
        let conf =
            get_monado_autorun_config_from_path(&"./test/files/monado_autorun_config.json".into())
                .expect("Could not find monado autorun config");
        assert_eq!(
            conf._schema,
            Some("https://monado.pages.freedesktop.org/monado/autorun_v0.schema.json".into())
        );
        assert_eq!(conf.autoruns.len(), 1);
        assert_eq!(conf.autoruns.get(0).unwrap().exec, "foobar");
        assert_eq!(conf.autoruns.get(0).unwrap().args.len(), 2);
        assert_eq!(conf.autoruns.get(0).unwrap().args.get(0).unwrap(), "bar");
        assert_eq!(conf.autoruns.get(0).unwrap().args.get(1).unwrap(), "baz");
    }
}
