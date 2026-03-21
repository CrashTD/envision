use ash::{
    Entry,
    vk::{ApplicationInfo, InstanceCreateInfo},
};

#[derive(Debug, Clone)]
pub struct VulkanInfo {
    pub gpu_names: Vec<String>,
    pub has_nvidia: bool,
}

const NVIDIA_VENDOR_ID: u32 = 0x10de;

impl VulkanInfo {
    /// # Safety
    ///
    /// Dlopens the vulkan library, so this is inherently unsafe. Should be fine in
    /// most circumstances
    pub fn get() -> anyhow::Result<Self> {
        let entry = unsafe { Entry::load() }?;
        let instance = unsafe {
            entry.create_instance(
                &InstanceCreateInfo::default().application_info(&ApplicationInfo::default()),
                None,
            )
        }?;
        let mut has_nvidia = false;
        let gpu_names = unsafe { instance.enumerate_physical_devices() }?
            .into_iter()
            .filter_map(|d| {
                let props = unsafe { instance.get_physical_device_properties(d) };
                if props.vendor_id == NVIDIA_VENDOR_ID {
                    has_nvidia = true;
                }
                Some(
                    props
                        .device_name_as_c_str()
                        .ok()?
                        .to_string_lossy()
                        .to_string(),
                )
            })
            .collect();
        unsafe { instance.destroy_instance(None) };
        Ok(Self {
            gpu_names,
            has_nvidia,
        })
    }
}
