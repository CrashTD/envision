use lazy_static::lazy_static;

fn env_var_descriptions() -> Vec<(&'static str, &'static str)> {
    vec![
        (
            "XRT_COMPOSITOR_SCALE_PERCENTAGE",
            "Render resolution percentage. A percentage higher than the native resolution (>100) will help with antialiasing and image clarity."
        ),
        (
            "XRT_COMPOSITOR_COMPUTE",
            "Set to 1 to use GPU compute for the OpenXR compositor."
        ),
        (
            "U_PACING_APP_USE_MIN_FRAME_PERIOD",
            "Set to 1 to unlimit the compositor refresh from a power of two of your HMD refresh, typically provides a large performance boost."
        ),
        (
            "SURVIVE_GLOBALSCENESOLVER",
            "Continuously recalibrate lighthouse tracking during use. In the current state it's recommended to disable this feature by setting this value to 0."
        ),
        // ("SURVIVE_TIMECODE_OFFSET_MS", ""),
        (
            "LD_LIBRARY_PATH",
            "Colon-separated list of directories where the dynamic linker will search for shared object libraries."
        ),
        (
            "XRT_DEBUG_GUI",
            "Set to 1 to enable the Monado debug UI."
        ),
        (
            "XRT_CURATED_GUI",
            "Set to 1 to enable the Monado preview UI. Requires XRT_DEBUG_GUI=1 to work."
        ),
        (
            "XRT_JSON_LOG",
            "Set to 1 to enable JSON logging for Monado. This enables better log visualization and log level filtering."
        ),
        (
            "QWERTY_ENABLE",
            "Set to 1 to enable QWERTY Simulation driver. This enables simulated driver that allows you to use Monado without HMD and controllers. It's also possible to mix and match different profiles with this."
        ),
        (
            "LH_DRIVER",
            "Lighthouse driver, this overrides the \"Lighthouse driver\" option in the profile; Valid options are:\n\"vive\" for the default built-in driver;\n\"survive\" for Libsurvive;\n\"steamvr\" for the SteamVR based implementation."
        ),
        (
            "LH_LOG",
            "Lighthouse log level. Can be one of: \"trace\", \"debug\", \"info\", \"warn\", \"error\"."
        ),
        (
            "LIGHTHOUSE_LOG",
            "Lighthouse driver log level. Can be one of: \"trace\", \"debug\", \"info\", \"warn\", \"error\"."
        ),
        (
            "LH_HANDTRACKING",
            "Controls when to enable mercury optical hand tracking. Valid options are:\n\"0\" (off) to disable hand tracking;\n\"1\" (auto) to only enable hand tracking if no controllers are found;\n\"2\" (on) to enable hand tracking even if controllers are found."
        ),
        (
            "WMR_HANDTRACKING",
            "Controls whether to enable hand tracking for WMR contorllers. Valid options are:\n\"0\" (off) to disable hand tracking;\n\"1\" (on) to enable hand tracking."
        ),
    ]
}

fn env_var_descriptions_as_paragraph() -> String {
    ENV_VAR_DESCRIPTIONS
        .iter()
        .map(|(k, v)| format!("<span size=\"large\" weight=\"bold\">{k}</span>\n{v}"))
        .collect::<Vec<String>>()
        .join("\n\n")
}

lazy_static! {
    pub static ref ENV_VAR_DESCRIPTIONS: Vec<(&'static str, &'static str)> = env_var_descriptions();
    pub static ref ENV_VAR_DESCRIPTIONS_AS_PARAGRAPH: String = env_var_descriptions_as_paragraph();
}
