use std::slice::Iter;

use serde::{Deserialize, Serialize};

use crate::{
    file_utils::{deserialize_file, get_writer},
    paths::get_xdg_config_dir,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum XrtInputName {
    #[serde(rename = "XRT_INPUT_GENERIC_TRACKER_POSE")]
    XrtInputGenericTrackerPose,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum XrtTrackerRole {
    #[serde(rename = "/user/vive_tracker_htcx/role/waist")]
    ViveTrackerHtcxWaist,
    #[serde(rename = "/user/vive_tracker_htcx/role/left_foot")]
    ViveTrackerHtcxLeftFoot,
    #[serde(rename = "/user/vive_tracker_htcx/role/right_foot")]
    ViveTrackerHtcxRightFoot,
    #[serde(rename = "/user/vive_tracker_htcx/role/handheld_object")]
    ViveTrackerHtcxHandheldObject,
    #[serde(rename = "/user/vive_tracker_htcx/role/left_shoulder")]
    ViveTrackerHtcxLeftShoulder,
    #[serde(rename = "/user/vive_tracker_htcx/role/right_shoulder")]
    ViveTrackerHtcxRightShoulder,
    #[serde(rename = "/user/vive_tracker_htcx/role/left_elbow")]
    ViveTrackerHtcxLeftElbow,
    #[serde(rename = "/user/vive_tracker_htcx/role/right_elbow")]
    ViveTrackerHtcxRightElbow,
    #[serde(rename = "/user/vive_tracker_htcx/role/left_knee")]
    ViveTrackerHtcxLeftKnee,
    #[serde(rename = "/user/vive_tracker_htcx/role/right_knee")]
    ViveTrackerHtcxRightKnee,
    #[serde(rename = "/user/vive_tracker_htcx/role/chest")]
    ViveTrackerHtcxChest,
    #[serde(rename = "/user/vive_tracker_htcx/role/camera")]
    ViveTrackerHtcxCamera,
    #[serde(rename = "/user/vive_tracker_htcx/role/keyboard")]
    ViveTrackerHtcxKeyboard,
}

impl XrtTrackerRole {
    pub fn to_picker_string(&self) -> &str {
        match self {
            Self::ViveTrackerHtcxWaist => "Waist",
            Self::ViveTrackerHtcxLeftFoot => "Left foot",
            Self::ViveTrackerHtcxRightFoot => "Right foot",
            Self::ViveTrackerHtcxHandheldObject => "Handheld object",
            Self::ViveTrackerHtcxLeftShoulder => "Left shoulder",
            Self::ViveTrackerHtcxRightShoulder => "Right shoulder",
            Self::ViveTrackerHtcxLeftElbow => "Left elbow",
            Self::ViveTrackerHtcxRightElbow => "Right elbow",
            Self::ViveTrackerHtcxLeftKnee => "Left knee",
            Self::ViveTrackerHtcxRightKnee => "Right knee",
            Self::ViveTrackerHtcxChest => "Chest",
            Self::ViveTrackerHtcxCamera => "Camera",
            Self::ViveTrackerHtcxKeyboard => "Keyboard",
        }
    }

    pub fn from_picker_string(s: &str) -> Self {
        match s.to_lowercase().trim() {
            "waist" => Self::ViveTrackerHtcxWaist,
            "left foot" => Self::ViveTrackerHtcxLeftFoot,
            "right foot" => Self::ViveTrackerHtcxRightFoot,
            "handheld object" => Self::ViveTrackerHtcxHandheldObject,
            "left shoulder" => Self::ViveTrackerHtcxLeftShoulder,
            "right shoulder" => Self::ViveTrackerHtcxRightShoulder,
            "left elbow" => Self::ViveTrackerHtcxLeftElbow,
            "right elbow" => Self::ViveTrackerHtcxRightElbow,
            "left knee" => Self::ViveTrackerHtcxLeftKnee,
            "right knee" => Self::ViveTrackerHtcxRightKnee,
            "chest" => Self::ViveTrackerHtcxChest,
            "camera" => Self::ViveTrackerHtcxCamera,
            "keyboard" => Self::ViveTrackerHtcxKeyboard,
            _ => Self::ViveTrackerHtcxWaist,
        }
    }

    pub fn iter() -> Iter<'static, Self> {
        [
            Self::ViveTrackerHtcxWaist,
            Self::ViveTrackerHtcxLeftFoot,
            Self::ViveTrackerHtcxRightFoot,
            Self::ViveTrackerHtcxHandheldObject,
            Self::ViveTrackerHtcxLeftShoulder,
            Self::ViveTrackerHtcxRightShoulder,
            Self::ViveTrackerHtcxLeftElbow,
            Self::ViveTrackerHtcxRightElbow,
            Self::ViveTrackerHtcxLeftKnee,
            Self::ViveTrackerHtcxRightKnee,
            Self::ViveTrackerHtcxChest,
            Self::ViveTrackerHtcxCamera,
            Self::ViveTrackerHtcxKeyboard,
        ]
        .iter()
    }

    pub fn to_number(&self) -> u32 {
        match self {
            Self::ViveTrackerHtcxWaist => 0,
            Self::ViveTrackerHtcxLeftFoot => 1,
            Self::ViveTrackerHtcxRightFoot => 2,
            Self::ViveTrackerHtcxHandheldObject => 3,
            Self::ViveTrackerHtcxLeftShoulder => 4,
            Self::ViveTrackerHtcxRightShoulder => 5,
            Self::ViveTrackerHtcxLeftElbow => 6,
            Self::ViveTrackerHtcxRightElbow => 7,
            Self::ViveTrackerHtcxLeftKnee => 8,
            Self::ViveTrackerHtcxRightKnee => 9,
            Self::ViveTrackerHtcxChest => 10,
            Self::ViveTrackerHtcxCamera => 11,
            Self::ViveTrackerHtcxKeyboard => 12,
        }
    }

    pub fn from_number(n: &u32) -> Self {
        match n {
            0 => Self::ViveTrackerHtcxWaist,
            1 => Self::ViveTrackerHtcxLeftFoot,
            2 => Self::ViveTrackerHtcxRightFoot,
            3 => Self::ViveTrackerHtcxHandheldObject,
            4 => Self::ViveTrackerHtcxLeftShoulder,
            5 => Self::ViveTrackerHtcxRightShoulder,
            6 => Self::ViveTrackerHtcxLeftElbow,
            7 => Self::ViveTrackerHtcxRightElbow,
            8 => Self::ViveTrackerHtcxLeftKnee,
            9 => Self::ViveTrackerHtcxRightKnee,
            10 => Self::ViveTrackerHtcxChest,
            11 => Self::ViveTrackerHtcxCamera,
            12 => Self::ViveTrackerHtcxKeyboard,
            _ => Self::ViveTrackerHtcxWaist,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackerRole {
    pub device_serial: String,
    pub role: XrtTrackerRole,
    pub xrt_input_name: XrtInputName,
}

impl Default for TrackerRole {
    fn default() -> Self {
        Self {
            device_serial: "".into(),
            role: XrtTrackerRole::ViveTrackerHtcxWaist,
            xrt_input_name: XrtInputName::XrtInputGenericTrackerPose,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MonadoConfigV0 {
    #[serde(skip_serializing_if = "Option::is_none", rename = "$schema")]
    _schema: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tracker_roles: Vec<TrackerRole>,
}

impl Default for MonadoConfigV0 {
    fn default() -> Self {
        Self {
            _schema: Some("https://monado.pages.freedesktop.org/monado/config_v0.schema.json".to_string()),
            tracker_roles: vec![],
        }
    }
}

impl MonadoConfigV0 {
    pub fn has_tracker_serial(&self, serial: &str) -> bool {
        self.tracker_roles.iter().any(|t| t.device_serial == serial)
    }

    pub fn get_next_free_xrt_tracker_role(&self) -> XrtTrackerRole {
        XrtTrackerRole::iter()
            .find(|role| !self.tracker_roles.iter().any(|et| et.role == **role))
            .unwrap_or(&XrtTrackerRole::ViveTrackerHtcxHandheldObject)
            .clone()
    }
}

fn get_monado_config_v0_path() -> String {
    format!(
        "{config}/monado/config_v0.json",
        config = get_xdg_config_dir()
    )
}

fn get_monado_config_v0_from_path(path_s: &String) -> Option<MonadoConfigV0> {
    deserialize_file(path_s)
}

pub fn get_monado_config_v0() -> MonadoConfigV0 {
    get_monado_config_v0_from_path(&get_monado_config_v0_path())
        .unwrap_or(MonadoConfigV0::default())
}

fn dump_monado_config_v0_to_path(config: &MonadoConfigV0, path_s: &String) {
    let writer = get_writer(path_s);
    serde_json::to_writer_pretty(writer, config).expect("Unable to save Monado config V0");
}

pub fn dump_monado_config_v0(config: &MonadoConfigV0) {
    dump_monado_config_v0_to_path(config, &get_monado_config_v0_path());
}

pub fn dump_generic_trackers(trackers: &[String]) -> usize {
    let mut conf = get_monado_config_v0();
    let mut added: usize = 0;
    trackers.iter().for_each(|serial| {
        if !conf.has_tracker_serial(serial) {
            conf.tracker_roles.push(TrackerRole {
                device_serial: serial.to_string(),
                role: conf.get_next_free_xrt_tracker_role(),
                ..TrackerRole::default()
            });
            added += 1;
        }
    });

    if added > 0 {
        dump_monado_config_v0(&conf);
    }

    added
}

#[cfg(test)]
mod tests {
    use super::get_monado_config_v0_from_path;
    use crate::file_builders::monado_config_v0::{XrtInputName, XrtTrackerRole};

    #[test]
    fn can_read_monado_config_v0() {
        let conf = get_monado_config_v0_from_path(&"./test/files/monado_config_v0.json".into())
            .expect("Couldn't find monado config v0");
        assert_eq!(conf.tracker_roles.len(), 3);
        assert_eq!(
            conf.tracker_roles
                .get(0)
                .expect("could not get tracker role zero")
                .role,
            XrtTrackerRole::ViveTrackerHtcxWaist
        );
        assert_eq!(
            conf.tracker_roles
                .get(0)
                .expect("could not get tracker role zero")
                .xrt_input_name,
            XrtInputName::XrtInputGenericTrackerPose
        );
    }
}
