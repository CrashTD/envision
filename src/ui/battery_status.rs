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
    pub fn icon(&self) -> &str {
        match self.battery_status.charge {
            n if n >= 1.0 => "battery-level-100-symbolic",
            n if n >= 0.9 => "battery-level-90-symbolic",
            n if n >= 0.8 => "battery-level-80-symbolic",
            n if n >= 0.7 => "battery-level-70-symbolic",
            n if n >= 0.6 => "battery-level-60-symbolic",
            n if n >= 0.5 => "battery-level-50-symbolic",
            n if n >= 0.4 => "battery-level-40-symbolic",
            n if n >= 0.3 => "battery-level-30-symbolic",
            n if n >= 0.2 => "battery-level-20-symbolic",
            n if n >= 0.1 => "battery-level-10-symbolic",
            _ => "battery-level-0-symbolic",
        }
    }
}
