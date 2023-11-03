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
            Self::GenericTracker,
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
            Self::GenericTracker => 7,
            Self::Other => 8,
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
            "Generic tracker" => Self::GenericTracker,
            _ => Self::GenericTracker,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct XRDevice {
    pub dev_type: XRDeviceRole,
    pub name: String,
    pub id: String,
    pub battery: f32, // battery percentage, from 0 to 1 maybe
                      // still need to implement it in monado & gui
}

impl Default for XRDevice {
    fn default() -> Self {
        Self {
            dev_type: XRDeviceRole::GenericTracker,
            name: String::default(),
            id: String::default(),
            battery: f32::default(),
        }
    }
}

impl XRDevice {
    pub fn generic_tracker_from_log_row(s: &str) -> Option<Self> {
        if s.starts_with(GENERIC_TRACKER_PREFIX) {
            let n_tracker = s.trim_start_matches(GENERIC_TRACKER_PREFIX);
            return Some(Self {
                dev_type: XRDeviceRole::GenericTracker,
                id: n_tracker.into(),
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
        [
            XRDeviceRole::Head,
            XRDeviceRole::Left,
            XRDeviceRole::Right,
            XRDeviceRole::HandTrackingLeft,
            XRDeviceRole::HandTrackingRight,
            XRDeviceRole::Gamepad,
            XRDeviceRole::Eyes,
        ]
        .iter()
        .for_each(|xrd| {
            if let Ok(dev) = monado.device_from_role(xrd.to_monado_str()) {
                res.push(Self {
                    name: dev.name,
                    id: dev.id.to_string(),
                    dev_type: xrd.clone(),
                    ..Default::default()
                })
            }
        });
        res
    }

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
        let old_tracker_ids = old
            .iter()
            .filter_map(|d| {
                if d.dev_type == XRDeviceRole::GenericTracker {
                    return Some(d.id.clone());
                }
                None
            })
            .collect::<Vec<String>>();
        for n_dev in new {
            if n_dev.dev_type == XRDeviceRole::GenericTracker {
                if !old_tracker_ids.contains(&n_dev.id) {
                    res.push(n_dev.clone());
                }
            } else {
                res.push(n_dev.clone());
            }
        }
        res
    }
}
