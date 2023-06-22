use phf::Map;
use phf_macros::phf_map;

pub static ENV_VAR_DESCRIPTIONS: Map<&str, &str> = phf_map! {
    "XRT_COMPOSITOR_SCALE_PECENTAGE" =>
        "Render resolution percentage. A percentage higher than the native resolution (>100) will help with antialiasing and image clarity.",
    // "XRT_COMPOSITOR_COMPUTE" => "",
    "SURVIVE_GLOBALSCENESOLVER" =>
        "Continuously recalibrate lighthouse tracking during use. In the current state it's recommended to disable this feature by setting this value to 0.",
    // "SURVIVE_TIMECODE_OFFSET_MS" => "",
    "LD_LIBRARY_PATH" =>
        "Colon-separated list of directories where the dynamic linker will search for shared object libraries.",
};

pub fn env_var_descriptions_as_paragraph() -> String {
    ENV_VAR_DESCRIPTIONS.into_iter()
        .map(|(k, v)| format!("{}: {}", k, v))
        .collect::<Vec<String>>().join("\n\n")
}
