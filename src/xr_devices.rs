use libmonado_rs;
use std::{fmt::Display, slice::Iter};

const GENERIC_TRACKER_PREFIX: &str = "Found generic tracker device: ";

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum XRDeviceRole {
    Head,
    Left,
    Right,
    Gamepad,
    Eyes,
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
            Self::Left => "Left",
            Self::Right => "Right",
            Self::Gamepad => "Gamepad",
            Self::Eyes => "Eye Tracking",
            Self::HandTrackingLeft => "Hand tracking left",
            Self::HandTrackingRight => "Hand tracking right",

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
            Self::Left,
            Self::Right,
            Self::Gamepad,
            Self::Eyes,
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
            Self::Left => "left",
            Self::Right => "right",
            Self::Gamepad => "gamepad",
            Self::Eyes => "eyes",
            Self::HandTrackingLeft => "hand-tracking-left",
            Self::HandTrackingRight => "hand-tracking-right",

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
            Self::Left => 1,
            Self::Right => 2,
            Self::Gamepad => 3,
            Self::Eyes => 4,
            Self::HandTrackingLeft => 5,
            Self::HandTrackingRight => 6,

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

#[derive(Debug, Clone, PartialEq)]
pub struct XRDevice {
    pub dev_type: XRDeviceRole,
    pub name: String,
    pub index: String,
    pub serial: Option<String>,
    pub battery: Option<f32>, // battery percentage, from 0 to 1 maybe
                              // still need to implement it in monado
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

    /* pub fn from_libmonado_devices(monado: &libmonado_rs::Monado) -> Vec<Self> {
        if let Ok(devs) = monado.devices() {
            return devs
                .into_iter()
                .map(|xrd| Self {
                    id: xrd.id.to_string(),
                    name: xrd.name,
                    ..Default::default()
                })
                .collect();
        }

        vec![]
    } */

    pub fn from_libmonado(monado: &libmonado_rs::Monado) -> Vec<Self> {
        let mut res = vec![];
        let mut devs_with_role = vec![];
        [
            XRDeviceRole::Head,
            XRDeviceRole::Left,
            XRDeviceRole::Right,
            XRDeviceRole::HandTrackingLeft,
            XRDeviceRole::HandTrackingRight,
            XRDeviceRole::HandheldObject,
            XRDeviceRole::LeftFoot,
            XRDeviceRole::RightFoot,
            XRDeviceRole::LeftShoulder,
            XRDeviceRole::RightShoulder,
            XRDeviceRole::LeftElbow,
            XRDeviceRole::RightElbow,
            XRDeviceRole::LeftKnee,
            XRDeviceRole::RightKnee,
            XRDeviceRole::Waist,
            XRDeviceRole::Chest,
            XRDeviceRole::Camera,
            XRDeviceRole::Keyboard,
            XRDeviceRole::Gamepad,
            XRDeviceRole::Eyes,
        ]
        .iter()
        .for_each(|xrd| {
            if let Ok(dev) = monado.device_from_role(xrd.to_monado_str()) {
                devs_with_role.push(dev.id);
                // let serial = match dev.serial() {
                //     Ok(s) => {
                //         serials.push(s.clone());
                //         Some(s)
                //     }
                //     Err(e) => {
                //         eprintln!(
                //             "Warning: could not get serial for monado device {} ({}): {:#?}",
                //             dev.name, dev.id, e
                //         );
                //         None
                //     }
                // };
                res.push(Self {
                    name: dev.name,
                    index: dev.id.to_string(),
                    // serial,
                    serial: None,
                    dev_type: xrd.clone(),
                    ..Default::default()
                })
            }
        });
        if let Ok(all_devs) = monado.devices() {
            all_devs
                .into_iter()
                .filter(|dev| !devs_with_role.contains(&dev.id))
                .for_each(|dev_gt| {
                    res.push(Self {
                        name: dev_gt.name.clone(),
                        index: dev_gt.id.to_string(),
                        serial: dev_gt.serial().ok(),
                        dev_type: XRDeviceRole::GenericTracker,
                        ..Default::default()
                    })
                })
        }
        res
    }

    #[deprecated]
    pub fn from_log_message(s: &str) -> Vec<Self> {
        let mut res = vec![];
        let rows = s.split('\n');
        let mut in_section = false;
        for row in rows {
            if !in_section && row.starts_with("\tIn roles:") {
                in_section = true;
                continue;
            }
            if in_section {
                if row.starts_with("\tResult:") {
                    break;
                }
                match row.trim().split(": ").collect::<Vec<&str>>()[..] {
                    [_, "<none>"] => {}
                    [dev_type_s, name] => {
                        if let Some(xrdt) = XRDeviceRole::from_monado_str(dev_type_s) {
                            res.push(Self {
                                dev_type: xrdt,
                                name: name.to_string(),
                                ..Default::default()
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
        res
    }

    pub fn merge(old: &[Self], new: &[Self]) -> Vec<Self> {
        if old.is_empty() {
            return Vec::from(new);
        }
        let new_dev_types = new
            .iter()
            .filter_map(|d| {
                if d.dev_type == XRDeviceRole::GenericTracker {
                    return None;
                }
                Some(d.dev_type)
            })
            .collect::<Vec<XRDeviceRole>>();
        let mut res = old
            .iter()
            .filter(|d| !new_dev_types.contains(&d.dev_type))
            .map(Self::clone)
            .collect::<Vec<Self>>();
        let old_tracker_serials = old
            .iter()
            .filter_map(|d| {
                if d.dev_type == XRDeviceRole::GenericTracker {
                    return d.serial.clone();
                }
                None
            })
            .collect::<Vec<String>>();
        for n_dev in new {
            if n_dev.dev_type == XRDeviceRole::GenericTracker {
                if let Some(n_serial) = n_dev.serial.as_ref() {
                    if !old_tracker_serials.contains(n_serial) {
                        res.push(n_dev.clone());
                    }
                }
            } else {
                res.push(n_dev.clone());
            }
        }
        res
    }
}
