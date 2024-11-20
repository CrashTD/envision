use crate::{
    util::file_utils::{deserialize_file, get_writer},
    xdg::XDG,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
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
    Vulkan,
}

impl Display for Encoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::X264 => "x264",
            Self::Nvenc => "NVEnc",
            Self::Vaapi => "VAAPI",
            Self::Vulkan => "Vulkan",
        })
    }
}

impl Encoder {
    pub fn iter() -> Iter<'static, Self> {
        [Self::X264, Self::Nvenc, Self::Vaapi, Self::Vulkan].iter()
    }

    pub fn as_vec() -> Vec<Self> {
        vec![Self::X264, Self::Nvenc, Self::Vaapi, Self::Vulkan]
    }
}

impl From<&Encoder> for u32 {
    fn from(value: &Encoder) -> Self {
        match value {
            Encoder::X264 => 0,
            Encoder::Nvenc => 1,
            Encoder::Vaapi => 2,
            Encoder::Vulkan => 3,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Codec {
    H264,
    H265,
    AV1,
}

impl Display for Codec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::H264 => "h264",
            Self::H265 => "h265",
            Self::AV1 => "av1",
        })
    }
}

impl Codec {
    pub fn iter() -> Iter<'static, Self> {
        [Self::H264, Self::H265, Self::AV1].iter()
    }

    pub fn as_vec() -> Vec<Self> {
        vec![Self::H264, Self::H265, Self::AV1]
    }
}

impl From<&Codec> for u32 {
    fn from(value: &Codec) -> Self {
        match value {
            Codec::H264 => 0,
            Codec::H265 => 1,
            Codec::AV1 => 2,
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
    /// contains unknown fields
    #[serde(flatten)]
    pub other: Map<String, Value>,
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
            other: Map::default(),
        }
    }
}

/// An application to start when connection with the headset is established,
/// can be a string or an array of strings if parameters need to be provided.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum WivrnConfigApplication {
    Str(String),
    StrArr(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WivrnConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<[f32; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bitrate: Option<u32>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub encoders: Vec<WivrnConfEncoder>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application: Option<WivrnConfigApplication>,
    #[serde(default)]
    pub tcp_only: bool,
    /// contains unknown fields
    #[serde(flatten)]
    other: Map<String, Value>,
}

impl Default for WivrnConfig {
    fn default() -> Self {
        Self {
            scale: Some([0.5, 0.5]),
            bitrate: Some(50000000),
            encoders: vec![],
            application: None,
            tcp_only: false,
            other: Map::default(),
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

    use crate::file_builders::wivrn_config::{Codec, Encoder, WivrnConfigApplication};

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
        assert_eq!(
            conf.application,
            Some(WivrnConfigApplication::Str("foobar".into()))
        );
        assert!(!conf.tcp_only);
        assert!(serde_json::to_string(&conf)
            .unwrap()
            .contains("\"extraneous\":\"field\",\"other\":{\"weird\":[\"stuff\"]}"));
        let conf = get_wivrn_config_from_path(Path::new("./test/files/wivrn_config2.json"))
            .expect("Couln't find wivrn config 2");
        assert_eq!(
            conf.application,
            Some(WivrnConfigApplication::StrArr(vec![
                "foobar".into(),
                "baz".into()
            ]))
        );
        assert!(conf.tcp_only);
    }
}
