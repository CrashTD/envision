use crate::util::file_utils::get_reader;
use std::{
    error::Error,
    fmt::Display,
    io::Read,
    path::{Path, PathBuf},
    str::FromStr,
};

// const POW_PROF_PATH: &str = "/sys/class/drm/card0/device/pp_power_profile_mode";

fn power_profile_mode_file(card_dir: &Path) -> PathBuf {
    card_dir.join("device/pp_power_profile_mode")
}

pub fn get_set_amd_vr_pow_prof_cmd(card_dir: &Path) -> String {
    format!(
        "sudo sh -c \"echo '4' > {}\"",
        power_profile_mode_file(card_dir).to_string_lossy()
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuPowerProfile {
    // 3D Full Screen, inverted cause can't start with number
    BootupDefault,
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

impl Default for GpuPowerProfileParseErr {
    fn default() -> Self {
        Self::new()
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
        if s.contains("bootup_default") {
            return Ok(Self::BootupDefault);
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

const AMD_VENDOR_ID: &str = "0x1002";
// TODO: add these other ids
// const INTEL_VENDOR_ID: &str = "0xGGGG";
// const NVIDIA_VENDOR_ID: &str = "0xGGGG";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GpuSysDrm {
    Amd(PathBuf),
    Intel(PathBuf),
    Nvidia(PathBuf),
    Other(PathBuf),
}

// usize field is total memory size
fn list_gpus() -> Vec<(GpuSysDrm, usize)> {
    let mut res = Vec::new();

    for i in 0..5 {
        // arbitrary range, find a better way
        let card_dir = PathBuf::from(format!("/sys/class/drm/card{i}"));
        let vendor_file = card_dir.join("device/vendor");
        let vram_total_file = card_dir.join("device/mem_info_vram_total");
        let mut vendor: Option<GpuSysDrm> = None;
        if let Some(mut reader) = get_reader(&vendor_file) {
            let mut buf = String::new();
            if reader.read_to_string(&mut buf).is_ok() {
                vendor = Some(match buf.to_lowercase().trim() {
                    AMD_VENDOR_ID => GpuSysDrm::Amd(card_dir),
                    _ => GpuSysDrm::Other(card_dir),
                });
            }
        }
        let mut vram_total: usize = 0;
        if let Some(mut reader) = get_reader(&vram_total_file) {
            let mut buf = String::new();
            if reader.read_to_string(&mut buf).is_ok() {
                vram_total = buf.trim().parse().unwrap_or_default();
            }
        }
        if let Some(vendor) = vendor {
            res.push((vendor, vram_total));
        }
    }

    res
}

fn get_first_dedicated_amd_gpu() -> Option<GpuSysDrm> {
    if let Some((g, _)) = list_gpus()
        .iter()
        .filter(|(g, _)| matches!(g, GpuSysDrm::Amd(_)))
        // assume that the gpu with the most vram is the dedicated one
        .max_by_key(|(_, vram)| vram)
    {
        Some(g.clone())
    } else {
        None
    }
}

pub fn get_amd_gpu_power_profile() -> Option<GpuPowerProfile> {
    let amd_gpu = get_first_dedicated_amd_gpu();
    if let Some(GpuSysDrm::Amd(card_dir)) = amd_gpu
        && let Some(mut reader) = get_reader(&power_profile_mode_file(&card_dir))
    {
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
