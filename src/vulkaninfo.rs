use ash::{
    vk::{ApplicationInfo, InstanceCreateInfo},
    Entry,
};

/// # Safety
///
/// Dlopens the vulkan library, so this is inherently unsafe. Should be fine in
/// most circumstances
pub unsafe fn gpu_names() -> anyhow::Result<Vec<String>> {
    let entry = Entry::load()?;
    let instance = entry.create_instance(
        &InstanceCreateInfo::default().application_info(&ApplicationInfo::default()),
        None,
    )?;
    let names = instance
        .enumerate_physical_devices()?
        .into_iter()
        .filter_map(|d| {
            instance
                .get_physical_device_properties(d)
                .device_name_as_c_str()
                .ok()
                .map(|cs| cs.to_string_lossy().to_string())
        })
        .collect();
    instance.destroy_instance(None);
    Ok(names)
}
