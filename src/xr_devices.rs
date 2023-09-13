use std::slice::Iter;

use crate::profile::Profile;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XRDevice {
    Head,
    Left,
    Right,
    Gamepad,
    Eyes,
    HandTrackingLeft,
    HandTrackingRight,
    GenericTracker,
}

impl XRDevice {
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
            Self::HandTrackingLeft => "hand_tracking.left",
            Self::HandTrackingRight => "hand_tracking.right",
            Self::GenericTracker => "generic_tracker",
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct XRDevices {
    pub head: Option<String>,
    pub left: Option<String>,
    pub right: Option<String>,
    pub gamepad: Option<String>,
    pub eyes: Option<String>,
    pub hand_tracking_left: Option<String>,
    pub hand_tracking_right: Option<String>,
    pub generic_trackers: Vec<String>,
}

const GENERIC_TRACKER_PREFIX: &str = "Found generic tracker device: ";

impl XRDevices {
    fn set_device(&mut self, dev: &XRDevice, name: String) {
        match dev {
            XRDevice::Head => self.head = Some(name),
            XRDevice::Left => self.left = Some(name),
            XRDevice::Right => self.right = Some(name),
            XRDevice::Gamepad => self.gamepad = Some(name),
            XRDevice::Eyes => self.eyes = Some(name),
            XRDevice::HandTrackingLeft => self.hand_tracking_left = Some(name),
            XRDevice::HandTrackingRight => self.hand_tracking_right = Some(name),
            _ => {}
        }
    }

    pub fn from_libmonado(prof: &Profile) -> Self {
        let mut res = Self::default();
        if let Some(libmonado_path) = prof.libmonado_so() {
            if let Ok(monado) = libmonado_rs::Monado::create(libmonado_path) {
                [
                    XRDevice::Head,
                    XRDevice::Left,
                    XRDevice::Right,
                    XRDevice::Gamepad,
                    XRDevice::Eyes,
                ]
                .iter()
                .for_each(|xrd| {
                    if let Ok(dev) = monado.device_from_role(xrd.to_monado_str()) {
                        res.set_device(&xrd, dev.name);
                    }
                });
            }
        }
        res
    }

    pub fn from_log_message(s: &str) -> Option<Self> {
        let rows = s.split('\n');
        let mut in_section = false;
        let mut devs = Self::default();
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
                    ["head", val] => devs.head = Some(val.to_string()),
                    ["left", val] => devs.left = Some(val.to_string()),
                    ["right", val] => devs.right = Some(val.to_string()),
                    ["gamepad", val] => devs.gamepad = Some(val.to_string()),
                    ["eyes", val] => devs.eyes = Some(val.to_string()),
                    ["hand_tracking.left", val] => devs.hand_tracking_left = Some(val.to_string()),
                    ["hand_tracking.right", val] => {
                        devs.hand_tracking_right = Some(val.to_string())
                    }
                    _ => {}
                }
            }
        }
        if in_section {
            return Some(devs);
        }
        None
    }

    pub fn merge(&mut self, new: Self) {
        if new.head.is_some() {
            self.head = new.head;
        }
        if new.left.is_some() {
            self.left = new.left;
        }
        if new.right.is_some() {
            self.right = new.right;
        }
        if new.gamepad.is_some() {
            self.gamepad = new.gamepad;
        }
        if new.eyes.is_some() {
            self.eyes = new.eyes;
        }
        if new.hand_tracking_left.is_some() {
            self.hand_tracking_left = new.hand_tracking_left;
        }
        if new.hand_tracking_right.is_some() {
            self.hand_tracking_right = new.hand_tracking_right;
        }
        if !new.generic_trackers.is_empty() {
            self.generic_trackers.extend(
                new.generic_trackers
                    .iter()
                    .filter(|t| !self.generic_trackers.contains(t))
                    .cloned()
                    .collect::<Vec<String>>(),
            );
        }
    }

    pub fn search_log_for_generic_trackers(&mut self, s: &str) {
        if s.starts_with(GENERIC_TRACKER_PREFIX) {
            let n_tracker = s.trim_start_matches(GENERIC_TRACKER_PREFIX);
            if !self.generic_trackers.contains(&n_tracker.to_string()) {
                self.generic_trackers.push(n_tracker.into());
            }
        }
    }
}
