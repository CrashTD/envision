use ash::{
    vk::{ApplicationInfo, InstanceCreateInfo},
    Entry,
};

#[derive(Debug, Clone)]
pub struct VulkanInfo {
    pub has_nvidia_gpu: bool,
    pub has_monado_vulkan_layers: bool,
    pub gpu_names: Vec<String>,
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
        let mut has_nvidia_gpu = false;
        let mut has_monado_vulkan_layers = false;
        let gpu_names = unsafe { instance.enumerate_physical_devices() }?
            .into_iter()
            .filter_map(|d| {
                let props = unsafe { instance.get_physical_device_properties(d) };
                if props.vendor_id == NVIDIA_VENDOR_ID {
                    has_nvidia_gpu = true;
                }
                if !has_monado_vulkan_layers {
                    has_monado_vulkan_layers =
                        unsafe { instance.enumerate_device_layer_properties(d) }
                            .ok()
                            .map(|layerprops| {
                                layerprops.iter().any(|lp| {
                                    lp.layer_name_as_c_str().is_ok_and(|name| {
                                        name.to_string_lossy()
                                            == "VK_LAYER_MND_enable_timeline_semaphore"
                                    })
                                })
                            })
                            == Some(true);
                }
                props
                    .device_name_as_c_str()
                    .ok()
                    .map(|cs| cs.to_string_lossy().to_string())
            })
            .collect();
        unsafe { instance.destroy_instance(None) };
        Ok(Self {
            gpu_names,
            has_nvidia_gpu,
            has_monado_vulkan_layers,
        })
    }
}
