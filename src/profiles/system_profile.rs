use std::collections::HashMap;

use crate::{
    config::Config,
    depcheck::{DepType, Dependency},
    profile::{
        LighthouseDriver, Profile, ProfileFeature, ProfileFeatureType, ProfileFeatures,
        ProfileOvrCompatibilityModule, XRServiceType,
    },
};

pub fn monado_dep() -> Dependency {
    Dependency {
        name: "monado".into(),
        dep_type: DepType::Executable,
        filename: XRServiceType::Monado.binary_name().into(),
        packages: HashMap::default(),
    }
}

/// will generate a system profile on the fly
pub fn system_profile(config: &Config) -> Profile {
    Profile {
        uuid: "envision-system-profile".into(),
        name: "envision-system-profile".into(),
        prefix: "/usr".into(),
        is_system_profile: true,
        ovr_comp: ProfileOvrCompatibilityModule {
            repo: None,
            branch: None,
            path: "/dev/null".into(),
            mod_type: config.global_preferences.ovr_comp,
        },
        features: ProfileFeatures {
            basalt: ProfileFeature {
                path: None,
                repo: None,
                branch: None,
                enabled: false,
                feature_type: ProfileFeatureType::Basalt,
            },
            openhmd: ProfileFeature {
                feature_type: ProfileFeatureType::OpenHmd,
                enabled: false,
                path: None,
                repo: None,
                branch: None,
            },
            libsurvive: ProfileFeature {
                feature_type: ProfileFeatureType::Libsurvive,
                enabled: false,
                path: None,
                repo: None,
                branch: None,
            },
            mercury_enabled: false,
        },
        editable: false,
        environment: config.global_preferences.get_env_vars(),
        pull_on_build: false,
        xrservice_type: config.global_preferences.xrservice_type,
        xrservice_path: "/dev/null".into(),
        xrservice_repo: None,
        xrservice_branch: None,
        lighthouse_driver: LighthouseDriver::SteamVR,
        xrservice_cmake_flags: HashMap::default(),
        skip_dependency_check: true,
        xrservice_launch_options: String::default(),
        ..Default::default()
    }
}
