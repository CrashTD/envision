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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackerRole {
    pub device_serial: String,
    pub role: XrtTrackerRole,
    pub xrt_input_name: XrtInputName,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct MonadoConfigV0 {
    #[serde(skip_serializing_if = "Option::is_none", rename = "$schema")]
    _schema: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tracker_roles: Vec<TrackerRole>,
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
