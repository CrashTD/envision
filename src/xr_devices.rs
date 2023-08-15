#[derive(Debug, Clone, PartialEq, Eq)]
pub enum XRDevice {
    Head,
    Left,
    Right,
    Gamepad,
    Eyes,
    HandTrackingLeft,
    HandTrackingRight,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XRDevices {
    pub head: Option<String>,
    pub left: Option<String>,
    pub right: Option<String>,
    pub gamepad: Option<String>,
    pub eyes: Option<String>,
    pub hand_tracking_left: Option<String>,
    pub hand_tracking_right: Option<String>,
}

impl Default for XRDevices {
    fn default() -> Self {
        Self {
            head: None,
            left: None,
            right: None,
            gamepad: None,
            eyes: None,
            hand_tracking_left: None,
            hand_tracking_right: None,
        }
    }
}

impl XRDevices {
    pub fn from_log_message(s: String) -> Option<Self> {
        let rows = s.split("\n");
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
}
