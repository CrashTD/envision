use std::fmt::Display;

#[derive(Debug, Clone, Default)]
pub struct BatteryStatus {
    /// between 0 and 100
    pub percentage: u8,
}

impl Display for BatteryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{}%", self.percentage))
    }
}

impl BatteryStatus {
    pub fn new(percentage: u8) -> Self {
        Self { percentage }
    }

    pub fn icon(&self) -> &str {
        match self.percentage {
            n if n >= 100 => "battery-level-100-symbolic",
            n if n >= 90 => "battery-level-90-symbolic",
            n if n >= 80 => "battery-level-80-symbolic",
            n if n >= 70 => "battery-level-70-symbolic",
            n if n >= 60 => "battery-level-60-symbolic",
            n if n >= 50 => "battery-level-50-symbolic",
            n if n >= 40 => "battery-level-40-symbolic",
            n if n >= 30 => "battery-level-30-symbolic",
            n if n >= 20 => "battery-level-20-symbolic",
            n if n >= 10 => "battery-level-10-symbolic",
            _ => "battery-level-0-symbolic",
        }
    }
}
