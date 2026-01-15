use libmonado::{self, BatteryStatus, DeviceLogic, DeviceRole, MndProperty, MndResult};
use std::{collections::HashMap, fmt::Display, rc::Rc, slice::Iter};
use tracing::error;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
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

    #[default]
    GenericTracker,
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
        })
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

            Self::GenericTracker => "generic-tracker",
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

impl From<&XRDeviceRole> for u32 {
    fn from(value: &XRDeviceRole) -> Self {
        match value {
            XRDeviceRole::Head => 0,
            XRDeviceRole::Eyes => 1,
            XRDeviceRole::Left => 2,
            XRDeviceRole::Right => 3,
            XRDeviceRole::Gamepad => 4,
            XRDeviceRole::HandTrackingLeft => 5,
            XRDeviceRole::HandTrackingRight => 6,

            // the following are not in libmonado
            XRDeviceRole::HandheldObject => 7,
            XRDeviceRole::LeftFoot => 8,
            XRDeviceRole::RightFoot => 9,
            XRDeviceRole::LeftShoulder => 10,
            XRDeviceRole::RightShoulder => 11,
            XRDeviceRole::LeftElbow => 12,
            XRDeviceRole::RightElbow => 13,
            XRDeviceRole::LeftKnee => 14,
            XRDeviceRole::RightKnee => 15,
            XRDeviceRole::Waist => 16,
            XRDeviceRole::Chest => 17,
            XRDeviceRole::Camera => 18,
            XRDeviceRole::Keyboard => 19,

            XRDeviceRole::GenericTracker => 20,
        }
    }
}

impl PartialOrd for XRDeviceRole {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for XRDeviceRole {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        u32::from(self).cmp(&other.into())
    }
}

impl From<DeviceRole> for XRDeviceRole {
    fn from(value: DeviceRole) -> Self {
        match value {
            DeviceRole::Head => XRDeviceRole::Head,
            DeviceRole::Eyes => XRDeviceRole::Eyes,
            DeviceRole::Left => XRDeviceRole::Left,
            DeviceRole::Right => XRDeviceRole::Right,
            DeviceRole::Gamepad => XRDeviceRole::Gamepad,
            DeviceRole::HandTrackingLeft => XRDeviceRole::HandTrackingLeft,
            DeviceRole::HandTrackingRight => XRDeviceRole::HandTrackingRight,
        }
    }
}

#[derive(Clone)]
pub struct XRDevice {
    pub roles: Vec<XRDeviceRole>,
    pub name: String,
    pub id: u32,
    pub index: u32,
    pub serial: Option<String>,
    pub battery: Option<BatteryStatus>,
    pub brightness: Option<f32>,
    pub has_brightness_control: bool,
    device: libmonado::DeviceRc,
}

impl std::fmt::Debug for XRDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("XRDevice")
            .field("roles", &self.roles)
            .field("name", &self.name)
            .field("id", &self.id)
            .field("index", &self.index)
            .field("serial", &self.serial)
            .field("battery", &self.battery)
            .field("has_brightness_control", &self.has_brightness_control)
            .finish_non_exhaustive()
    }
}

impl PartialEq for XRDevice {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.has_brightness_control == other.has_brightness_control
            && self.name == other.name
            && self.roles == other.roles
    }
}
impl Eq for XRDevice {}

impl XRDevice {
    pub fn from_libmonado(monado: Rc<libmonado::Monado>) -> Vec<Self> {
        if let Ok(monado_devs) = libmonado::Monado::devices_rc(&monado) {
            let mut devs: HashMap<u32, XRDevice> = monado_devs
                .into_iter()
                .map(|dev| {
                    let has_brightness_control = dev
                        .get_info_bool(MndProperty::PropertySupportsBrightnessBool)
                        .unwrap_or(false);
                    (
                        dev.index,
                        Self {
                            id: dev.name_id,
                            index: dev.index,
                            serial: dev.serial().ok(),
                            battery: dev
                                .battery_status()
                                .ok()
                                .and_then(|bs| if bs.present { Some(bs) } else { None }),
                            has_brightness_control,
                            brightness: if has_brightness_control {
                                dev.brightness().ok()
                            } else {
                                None
                            },
                            name: dev.name.clone(),
                            roles: Vec::default(),
                            device: dev.clone(),
                        },
                    )
                })
                .collect();
            [
                DeviceRole::Head,
                DeviceRole::Eyes,
                DeviceRole::Left,
                DeviceRole::Right,
                DeviceRole::Gamepad,
                DeviceRole::HandTrackingLeft,
                DeviceRole::HandTrackingRight,
            ]
            .into_iter()
            .for_each(|role| {
                if let Ok(index) = monado.device_index_from_role(role) {
                    if let Some(target) = devs.get_mut(&index) {
                        target.roles.push(role.into());
                    } else {
                        error!(
                            "could not find device index {index} for role {}",
                            XRDeviceRole::from(role)
                        )
                    }
                }
            });
            devs.drain().map(|(_, d)| d).collect()
        } else {
            Vec::default()
        }
    }

    pub fn update(&mut self) {
        self.battery = self
            .device
            .battery_status()
            .ok()
            .and_then(|bs| if bs.present { Some(bs) } else { None });
        self.brightness = if self.has_brightness_control {
            self.device.brightness().ok()
        } else {
            None
        };
    }

    /// IMPORTANT: value must be between 0 and 1; values outside this range
    /// will be clamped
    pub fn set_brightness(&self, value: f32) -> Result<(), MndResult> {
        // clamp the value for bonus correctness
        let value = value.clamp(0.0, 1.0);
        self.device.set_brightness(value, false)
    }
}
