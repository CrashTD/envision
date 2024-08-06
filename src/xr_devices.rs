use libmonado_rs::{self, BatteryStatus};
use std::{fmt::Display, slice::Iter};

const GENERIC_TRACKER_PREFIX: &str = "Found generic tracker device: ";

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum XRDeviceRole {
    Head,
    Eyes,
    Left,
    Right,
    Gamepad,
    HandTrackingLeft,
    HandTrackingRight,

    HandheldObject,
    LeftFoot,
    RightFoot,
    LeftShoulder,
    RightShoulder,
    LeftElbow,
    RightElbow,
    LeftKnee,
    RightKnee,
    Waist,
    Chest,
    Camera,
    Keyboard,

    GenericTracker,
    /**
     * Devices with no role
     */
    Other,
}

impl Default for XRDeviceRole {
    fn default() -> Self {
        Self::Other
    }
}

impl Display for XRDeviceRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Head => "Head",
            Self::Eyes => "Eye Tracking",
            Self::Left => "Left",
            Self::Right => "Right",
            Self::Gamepad => "Gamepad",
            Self::HandTrackingLeft => "Hand tracking left",
            Self::HandTrackingRight => "Hand tracking right",

            // the following are not in libmonado
            Self::HandheldObject => "Handheld object",
            Self::LeftFoot => "Left foot",
            Self::RightFoot => "Right foot",
            Self::LeftShoulder => "Left shoulder",
            Self::RightShoulder => "Right shoulder",
            Self::LeftElbow => "Left elbow",
            Self::RightElbow => "Right elbow",
            Self::LeftKnee => "Left knee",
            Self::RightKnee => "Right knee",
            Self::Waist => "Waist",
            Self::Chest => "Chest",
            Self::Camera => "Camera",
            Self::Keyboard => "Keyboard",

            Self::GenericTracker => "Generic tracker",
            Self::Other => "",
        })
    }
}

impl PartialOrd for XRDeviceRole {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.as_number().cmp(&other.as_number()))
    }
}

impl Ord for XRDeviceRole {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl XRDeviceRole {
    pub fn iter() -> Iter<'static, Self> {
        [
            Self::Head,
            Self::Eyes,
            Self::Left,
            Self::Right,
            Self::Gamepad,
            // the following are not in libmonado
            Self::HandTrackingLeft,
            Self::HandTrackingRight,
            Self::HandheldObject,
            Self::LeftFoot,
            Self::RightFoot,
            Self::LeftShoulder,
            Self::RightShoulder,
            Self::LeftElbow,
            Self::RightElbow,
            Self::LeftKnee,
            Self::RightKnee,
            Self::Waist,
            Self::Chest,
            Self::Camera,
            Self::Keyboard,
        ]
        .iter()
    }

    pub fn to_monado_str(&self) -> &str {
        match self {
            Self::Head => "head",
            Self::Eyes => "eyes",
            Self::Left => "left",
            Self::Right => "right",
            Self::Gamepad => "gamepad",
            Self::HandTrackingLeft => "hand-tracking-left",
            Self::HandTrackingRight => "hand-tracking-right",

            // the following are not in libmonado
            Self::HandheldObject => "handheld-object",
            Self::LeftFoot => "left-foot",
            Self::RightFoot => "right-foot",
            Self::LeftShoulder => "left-shoulder",
            Self::RightShoulder => "right-shoulder",
            Self::LeftElbow => "left-elbow",
            Self::RightElbow => "right-elbow",
            Self::LeftKnee => "left-knee",
            Self::RightKnee => "right-knee",
            Self::Waist => "waist",
            Self::Chest => "chest",
            Self::Camera => "camera",
            Self::Keyboard => "keyboard",

            Self::GenericTracker => "generic-tracker", // not actually in monado
            Self::Other => "other",                    // not actually in monado
        }
    }

    pub fn as_number(&self) -> u32 {
        match self {
            Self::Head => 0,
            Self::Eyes => 1,
            Self::Left => 2,
            Self::Right => 3,
            Self::Gamepad => 4,
            Self::HandTrackingLeft => 5,
            Self::HandTrackingRight => 6,

            // the following are not in libmonado
            Self::HandheldObject => 7,
            Self::LeftFoot => 8,
            Self::RightFoot => 9,
            Self::LeftShoulder => 10,
            Self::RightShoulder => 11,
            Self::LeftElbow => 12,
            Self::RightElbow => 13,
            Self::LeftKnee => 14,
            Self::RightKnee => 15,
            Self::Waist => 16,
            Self::Chest => 17,
            Self::Camera => 18,
            Self::Keyboard => 19,

            Self::GenericTracker => 20,
            Self::Other => 21,
        }
    }

    pub fn from_monado_str(s: &str) -> Option<Self> {
        match s {
            "head" => Some(Self::Head),
            "left" => Some(Self::Left),
            "right" => Some(Self::Right),
            "gamepad" => Some(Self::Gamepad),
            "eyes" => Some(Self::Eyes),
            "hand-tracking-left" => Some(Self::HandTrackingLeft),
            "hand-tracking-right" => Some(Self::HandTrackingRight),
            "handheld-object" => Some(Self::HandheldObject),
            "left-foot" => Some(Self::LeftFoot),
            "right-foot" => Some(Self::RightFoot),
            "left-shoulder" => Some(Self::LeftShoulder),
            "right-shoulder" => Some(Self::RightShoulder),
            "left-elbow" => Some(Self::LeftElbow),
            "right-elbow" => Some(Self::RightElbow),
            "left-knee" => Some(Self::LeftKnee),
            "right-knee" => Some(Self::RightKnee),
            "waist" => Some(Self::Waist),
            "chest" => Some(Self::Chest),
            "camera" => Some(Self::Camera),
            "keyboard" => Some(Self::Keyboard),
            _ => None,
        }
    }

    pub fn from_display_str(s: &str) -> Self {
        match s {
            "Head" => Self::Head,
            "Left" => Self::Left,
            "Right" => Self::Right,
            "Gamepad" => Self::Gamepad,
            "Eye Tracking" => Self::Eyes,
            "Hand tracking left" => Self::HandTrackingLeft,
            "Hand tracking right" => Self::HandTrackingRight,

            "Handheld object" => Self::HandheldObject,
            "Left foot" => Self::LeftFoot,
            "Right foot" => Self::RightFoot,
            "Left shoulder" => Self::LeftShoulder,
            "Right shoulder" => Self::RightShoulder,
            "Left elbow" => Self::LeftElbow,
            "Right elbow" => Self::RightElbow,
            "Left knee" => Self::LeftKnee,
            "Right knee" => Self::RightKnee,
            "Waist" => Self::Waist,
            "Chest" => Self::Chest,
            "Camera" => Self::Camera,
            "Keyboard" => Self::Keyboard,

            "Generic tracker" => Self::GenericTracker,
            _ => Self::GenericTracker,
        }
    }
}

#[derive(Debug, Clone)]
pub struct XRDevice {
    pub dev_type: XRDeviceRole,
    pub name: String,
    pub index: String,
    pub serial: Option<String>,
    pub battery: Option<BatteryStatus>,
}

impl Default for XRDevice {
    fn default() -> Self {
        Self {
            dev_type: XRDeviceRole::GenericTracker,
            name: String::default(),
            index: String::default(),
            serial: None,
            battery: Option::default(),
        }
    }
}

impl XRDevice {
    #[deprecated]
    pub fn generic_tracker_from_log_row(s: &str) -> Option<Self> {
        if s.starts_with(GENERIC_TRACKER_PREFIX) {
            let n_tracker = s.trim_start_matches(GENERIC_TRACKER_PREFIX);
            return Some(Self {
                dev_type: XRDeviceRole::GenericTracker,
                index: n_tracker.into(),
                ..Default::default()
            });
        }
        None
    }

    pub fn from_libmonado(monado: &libmonado_rs::Monado) -> Vec<Self> {
        let mut devs_with_role = vec![];
        // only roles in src/xrt/targets/libmonado/monado.c:role_enum
        let mut res = [
            XRDeviceRole::Head,
            XRDeviceRole::Eyes,
            XRDeviceRole::Left,
            XRDeviceRole::Right,
            XRDeviceRole::Gamepad,
            XRDeviceRole::HandTrackingLeft,
            XRDeviceRole::HandTrackingRight,
        ]
        .into_iter()
        .filter_map(|xrd| {
            if let Ok(dev) = monado.device_from_role(xrd.to_monado_str()) {
                devs_with_role.push(dev.id);
                Some(Self {
                    index: dev.id.to_string(),
                    serial: dev.serial().ok(),
                    battery: dev.battery_status().ok().and_then(|bs| {
                        if bs.present {
                            Some(bs)
                        } else {
                            None
                        }
                    }),
                    name: dev.name,
                    dev_type: xrd,
                })
            } else {
                None
            }
        })
        .collect::<Vec<Self>>();
        if let Ok(all_devs) = monado.devices() {
            res.extend(all_devs.into_iter().filter_map(|dev| {
                if devs_with_role.contains(&dev.id) {
                    None
                } else {
                    Some(Self {
                        index: dev.id.to_string(),
                        serial: dev.serial().ok(),
                        battery: dev.battery_status().ok().and_then(|bs| {
                            if bs.present {
                                Some(bs)
                            } else {
                                None
                            }
                        }),
                        name: dev.name,
                        dev_type: XRDeviceRole::GenericTracker,
                    })
                }
            }))
        }
        res
    }
}
