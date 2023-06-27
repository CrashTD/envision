use expect_dialog::ExpectDialog;
use serde::{Serialize, Deserialize};

use crate::file_utils::{get_xdg_config_dir, deserialize_file, get_writer};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Encoder {
    X264,
    X265,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Codec {
    H264,
    H265,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WivrnConfEncoder {
    pub encoder: Encoder,
    pub codec: Codec,
    pub bitrate: u32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WivrnConfig {
    pub scale: [f32; 2],
    pub encoders: Vec<WivrnConfEncoder>,
}

impl Default for WivrnConfig {
    fn default() -> Self {
        Self {
            scale: [0.8, 0.8],
            encoders: vec![
                WivrnConfEncoder {
                    encoder: Encoder::X264,
                    codec: Codec::H264,
                    bitrate: 100000000,
                    width: 1.0,
                    height: 1.0,
                }
            ],
        }
    }
}

fn get_wivrn_config_path() -> String {
    format!(
        "{config}/wivrn/config.json",
        config = get_xdg_config_dir()
    )
}

fn get_wivrn_config_from_path(path_s: &String) -> Option<WivrnConfig> {
    deserialize_file(path_s)
}

pub fn get_wivrn_config() -> Option<WivrnConfig> {
    get_wivrn_config_from_path(&get_wivrn_config_path())
}

fn dump_wivrn_config_to_path(config: &WivrnConfig, path_s: &String) {
    let writer = get_writer(path_s);
    serde_json::to_writer_pretty(writer, config)
        .expect_dialog("Unable to save WiVRn config");
}

pub fn dump_wivrn_config(config: &WivrnConfig, path_s: &String) {
    dump_wivrn_config_to_path(config, path_s);
}

#[cfg(test)]
mod tests {
    use crate::file_builders::wivrn_config::{Encoder, Codec};

    use super::get_wivrn_config_from_path;


    #[test]
    fn can_read_wivrn_config() {
        let conf = get_wivrn_config_from_path(&"./test/files/wivrn_config.json".into()).expect("Couldn't find wivrn config");
        assert_eq!(conf.scale, [0.8, 0.8]);
        assert_eq!(conf.encoders.len(), 1);
        assert_eq!(conf.encoders.get(0).unwrap().encoder, Encoder::X264);
        assert_eq!(conf.encoders.get(0).unwrap().codec, Codec::H264);
        assert_eq!(conf.encoders.get(0).unwrap().bitrate, 100000000);
        assert_eq!(conf.encoders.get(0).unwrap().width, 1.0);
        assert_eq!(conf.encoders.get(0).unwrap().height, 1.0);
    }
}
