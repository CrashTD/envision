use phf::Map;
use phf_macros::phf_map;

pub static ENV_VAR_DESCRIPTIONS: Map<&str, &str> = phf_map! {
    "XRT_COMPOSITOR_SCALE_PECENTAGE" =>
        "Render resolution percentage. A percentage higher than the native resolution (>100) will help with antialiasing and image clarity.",
    "XRT_COMPOSITOR_COMPUTE" => "Set to 1 to use GPU compute for the OpenXR compositor.",
    "U_PACING_APP_USE_MIN_FRAME_PERIOD" => "Set to 1 to unlimit the compositor refresh from a power of two of your HMD refresh, typically provides a large performance boost.",
    "SURVIVE_GLOBALSCENESOLVER" =>
        "Continuously recalibrate lighthouse tracking during use. In the current state it's recommended to disable this feature by setting this value to 0.",
    // "SURVIVE_TIMECODE_OFFSET_MS" => "",
    "LD_LIBRARY_PATH" =>
        "Colon-separated list of directories where the dynamic linker will search for shared object libraries.",
    "XRT_DEBUG_GUI" => "Set to 1 to enable the Monado debug UI.",
    "XRT_CURATED_GUI" => "Set to 1 to enable the Monado preview UI. Requires XRT_DEBUG_GUI=1 to work.",
    "XRT_JSON_LOG" => "Set to 1 to enable JSON logging for Monado. This enables better log visualization and log level filtering.",
    "LH_DRIVER" => "Lighthouse driver, this overrides the \"Lighthouse driver option in the profile\"; Valid options are: \"vive\" for the default built-in driver, \"survive\" for Libsurvive, \"steamvr\" for the SteamVR based implementation.",
    "LH_LOG" => "Lighthouse log level. Can be one of: \"trace\", \"debug\", \"info\", \"warn\", \"error\".",
    "LIGHTHOUSE_LOG" => "Lighthouse driver log level. Can be one of: \"trace\", \"debug\", \"info\", \"warn\", \"error\"."
};

pub fn env_var_descriptions_as_paragraph() -> String {
    ENV_VAR_DESCRIPTIONS
        .into_iter()
        .map(|(k, v)| format!(" \u{2022} <b>{}</b>: {}", k, v))
        .collect::<Vec<String>>()
        .join("\n\n")
}
