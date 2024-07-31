use libmonado_rs::BatteryStatus;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct EnvisionBatteryStatus {
    pub battery_status: BatteryStatus,
}

impl Display for EnvisionBatteryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "{}%",
            (self.battery_status.charge * 100.0).round()
        ))
    }
}

impl From<BatteryStatus> for EnvisionBatteryStatus {
    fn from(battery_status: BatteryStatus) -> Self {
        Self { battery_status }
    }
}

impl EnvisionBatteryStatus {
    pub fn icon(&self) -> String {
        format!(
            "{}{}-symbolic",
            match self.battery_status.charge {
                n if n >= 1.0 => "battery-level-100",
                n if n >= 0.9 => "battery-level-90",
                n if n >= 0.8 => "battery-level-80",
                n if n >= 0.7 => "battery-level-70",
                n if n >= 0.6 => "battery-level-60",
                n if n >= 0.5 => "battery-level-50",
                n if n >= 0.4 => "battery-level-40",
                n if n >= 0.3 => "battery-level-30",
                n if n >= 0.2 => "battery-level-20",
                n if n >= 0.1 => "battery-level-10",
                _ => "battery-level-0",
            },
            if self.battery_status.charging {
                "-charging"
            } else {
                ""
            }
        )
    }
}
