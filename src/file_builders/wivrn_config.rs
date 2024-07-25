use crate::{
    file_utils::{deserialize_file, get_writer},
    xdg::XDG,
};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    path::{Path, PathBuf},
    slice::Iter,
};

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
}

impl Display for Codec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::H264 => "h264",
            Self::H265 => "h265",
        })
    }
}

impl Codec {
    pub fn iter() -> Iter<'static, Self> {
        [Self::H264, Self::H265].iter()
    }

    pub fn as_vec() -> Vec<Self> {
        vec![Self::H264, Self::H265]
    }

    pub fn as_number(&self) -> u32 {
        match self {
            Self::H264 => 0,
            Self::H265 => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WivrnConfEncoder {
    pub encoder: Encoder,
    pub codec: Codec,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset_x: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset_y: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<i32>,
}

impl Default for WivrnConfEncoder {
    fn default() -> Self {
        Self {
            encoder: Encoder::X264,
            codec: Codec::H264,
            width: Some(1.0),
            height: Some(1.0),
            offset_x: Some(0.0),
            offset_y: Some(0.0),
            group: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WivrnConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<[f32; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bitrate: Option<u32>,
    pub encoders: Vec<WivrnConfEncoder>,
}

impl Default for WivrnConfig {
    fn default() -> Self {
        Self {
            scale: Some([0.8, 0.8]),
            bitrate: Some(50000000),
            encoders: vec![WivrnConfEncoder::default()],
        }
    }
}

fn get_wivrn_config_path() -> PathBuf {
    XDG.get_config_home().join("wivrn/config.json")
}

fn get_wivrn_config_from_path(path: &Path) -> Option<WivrnConfig> {
    deserialize_file(path)
}

pub fn get_wivrn_config() -> WivrnConfig {
    get_wivrn_config_from_path(&get_wivrn_config_path()).unwrap_or_default()
}

fn dump_wivrn_config_to_path(config: &WivrnConfig, path: &Path) {
    let writer = get_writer(path).expect("Unable to save WiVRn config");
    serde_json::to_writer_pretty(writer, config).expect("Unable to save WiVRn config");
}

pub fn dump_wivrn_config(config: &WivrnConfig) {
    dump_wivrn_config_to_path(config, &get_wivrn_config_path());
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::file_builders::wivrn_config::{Codec, Encoder};

    use super::get_wivrn_config_from_path;

    #[test]
    fn can_read_wivrn_config() {
        let conf = get_wivrn_config_from_path(Path::new("./test/files/wivrn_config.json"))
            .expect("Couldn't find wivrn config");
        assert_eq!(conf.scale, Some([0.8, 0.8]));
        assert_eq!(conf.encoders.len(), 1);
        assert_eq!(conf.encoders.first().unwrap().encoder, Encoder::X264);
        assert_eq!(conf.encoders.first().unwrap().codec, Codec::H264);
        assert_eq!(conf.bitrate, Some(100000000));
        assert_eq!(conf.encoders.first().unwrap().width, Some(1.0));
        assert_eq!(conf.encoders.first().unwrap().height, Some(1.0));
        assert_eq!(conf.encoders.first().unwrap().offset_x, Some(0.0));
        assert_eq!(conf.encoders.first().unwrap().offset_y, Some(0.0));
    }
}
