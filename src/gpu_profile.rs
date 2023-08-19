use crate::file_utils::get_reader;
use std::{error::Error, fmt::Display, io::Read, str::FromStr};

const POW_PROF_PATH: &str = "/sys/class/drm/card0/device/pp_power_profile_mode";

pub fn get_set_vr_pow_prof_cmd() -> String {
    format!("sudo sh -c \"echo '4' > {}\"", POW_PROF_PATH)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuPowerProfile {
    // 3D Full Screen, inverted cause can't start with number
    Fullscreen3D,
    PowerSaving,
    Video,
    VR,
    Compute,
    Custom,
}

#[derive(Debug)]
pub struct GpuPowerProfileParseErr;

impl GpuPowerProfileParseErr {
    pub fn new() -> Self {
        Self {}
    }
}

impl Display for GpuPowerProfileParseErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("GpuPowerProfileParseErr")
    }
}

impl Error for GpuPowerProfileParseErr {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

impl FromStr for GpuPowerProfile {
    type Err = GpuPowerProfileParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        let s = s.trim();
        if !s.contains('*') {
            return Err(Self::Err::new());
        }
        if s.contains("3d_full_screen") {
            return Ok(Self::Fullscreen3D);
        }
        if s.contains("power_saving") {
            return Ok(Self::PowerSaving);
        }
        if s.contains("video") {
            return Ok(Self::Video);
        }
        if s.contains("vr") {
            return Ok(Self::VR);
        }
        if s.contains("compute") {
            return Ok(Self::Compute);
        }
        if s.contains("custom") {
            return Ok(Self::Custom);
        }

        Err(Self::Err::new())
    }
}

pub fn get_gpu_power_profile() -> Option<GpuPowerProfile> {
    if let Some(mut reader) = get_reader(&POW_PROF_PATH.into()) {
        let mut txt = String::new();
        reader.read_to_string(&mut txt).ok()?;
        for line in txt.split('\n') {
            if let Ok(pp) = GpuPowerProfile::from_str(line) {
                return Some(pp);
            }
        }
    }
    None
}
