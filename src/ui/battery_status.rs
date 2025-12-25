use libmonado::BatteryStatus;
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
            "battery-level-{}{}-symbolic",
            match self.battery_status.charge {
                n if n >= 1.0 => "100",
                n if n >= 0.9 => "90",
                n if n >= 0.8 => "80",
                n if n >= 0.7 => "70",
                n if n >= 0.6 => "60",
                n if n >= 0.5 => "50",
                n if n >= 0.4 => "40",
                n if n >= 0.3 => "30",
                n if n >= 0.2 => "20",
                n if n >= 0.1 => "10",
                _ => "0",
            },
            if self.battery_status.charging {
                if self.battery_status.charge >= 1.0 {
                    "-charged"
                } else {
                    "-charging"
                }
            } else {
                ""
            }
        )
    }
}
