use crate::{
    file_utils::{deserialize_file, get_writer},
    paths::get_xdg_config_dir,
};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, slice::Iter};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Encoder {
    X264,
    Nvenc,
    Vaapi,
}

impl Display for Encoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::X264 => "x264",
            Self::Nvenc => "NVEnc",
            Self::Vaapi => "VAAPI",
        })
    }
}

impl Encoder {
    pub fn iter() -> Iter<'static, Self> {
        [Self::X264, Self::Nvenc, Self::Vaapi].iter()
    }

    pub fn as_vec() -> Vec<Self> {
        vec![Self::X264, Self::Nvenc, Self::Vaapi]
    }

    pub fn as_number(&self) -> u32 {
        match self {
            Self::X264 => 0,
            Self::Nvenc => 1,
            Self::Vaapi => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Codec {
    H264,
    H265,
    Avc,
    Hevc,
}

impl Display for Codec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::H264 => "h264",
            Self::H265 => "h265",
            Self::Avc => "AVC",
            Self::Hevc => "HEVC",
        })
    }
}

impl Codec {
    pub fn iter() -> Iter<'static, Self> {
        [Self::H264, Self::H265, Self::Avc, Self::Hevc].iter()
    }

    pub fn as_vec() -> Vec<Self> {
        vec![Self::H264, Self::H265, Self::Avc, Self::Hevc]
    }

    pub fn as_number(&self) -> u32 {
        match self {
            Self::H264 => 0,
            Self::H265 => 1,
            Self::Avc => 2,
            Self::Hevc => 3,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WivrnConfEncoder {
    pub encoder: Encoder,
    pub codec: Codec,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bitrate: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<i32>,
}

impl Default for WivrnConfEncoder {
    fn default() -> Self {
        Self {
            encoder: Encoder::X264,
            codec: Codec::H264,
            bitrate: None,
            width: None,
            height: None,
            group: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WivrnConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<[f32; 2]>,
    pub encoders: Vec<WivrnConfEncoder>,
}

impl Default for WivrnConfig {
    fn default() -> Self {
        Self {
            scale: Some([0.8, 0.8]),
            encoders: vec![WivrnConfEncoder {
                encoder: Encoder::X264,
                codec: Codec::H264,
                bitrate: Some(100000000),
                width: Some(1.0),
                height: Some(1.0),
                group: None,
            }],
        }
    }
}

fn get_wivrn_config_path() -> String {
    format!("{config}/wivrn/config.json", config = get_xdg_config_dir())
}

fn get_wivrn_config_from_path(path_s: &String) -> Option<WivrnConfig> {
    deserialize_file(path_s)
}

pub fn get_wivrn_config() -> WivrnConfig {
    get_wivrn_config_from_path(&get_wivrn_config_path()).unwrap_or(WivrnConfig::default())
}

fn dump_wivrn_config_to_path(config: &WivrnConfig, path_s: &String) {
    let writer = get_writer(path_s);
    serde_json::to_writer_pretty(writer, config).expect("Unable to save WiVRn config");
}

pub fn dump_wivrn_config(config: &WivrnConfig) {
    dump_wivrn_config_to_path(config, &get_wivrn_config_path());
}

#[cfg(test)]
mod tests {
    use crate::file_builders::wivrn_config::{Codec, Encoder};

    use super::get_wivrn_config_from_path;

    #[test]
    fn can_read_wivrn_config() {
        let conf = get_wivrn_config_from_path(&"./test/files/wivrn_config.json".into())
            .expect("Couldn't find wivrn config");
        assert_eq!(conf.scale, Some([0.8, 0.8]));
        assert_eq!(conf.encoders.len(), 1);
        assert_eq!(conf.encoders.get(0).unwrap().encoder, Encoder::X264);
        assert_eq!(conf.encoders.get(0).unwrap().codec, Codec::H264);
        assert_eq!(conf.encoders.get(0).unwrap().bitrate, Some(100000000));
        assert_eq!(conf.encoders.get(0).unwrap().width, Some(1.0));
        assert_eq!(conf.encoders.get(0).unwrap().height, Some(1.0));
    }
}
