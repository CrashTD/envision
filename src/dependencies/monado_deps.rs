use crate::depcheck::{check_dependencies, DepType, Dependency, DependencyCheckResult};

fn monado_deps() -> Vec<Dependency> {
    vec![
        Dependency {
            name: "libdrm".into(),
            dep_type: DepType::SharedObject,
            filename: "libdrm.so".into(),
        },
        Dependency {
            name: "libgl".into(),
            dep_type: DepType::SharedObject,
            filename: "libdrm.so".into(),
        },
        Dependency {
            name: "openxr".into(),
            dep_type: DepType::SharedObject,
            filename: "libopenxr_loader.so".into(),
        },
        Dependency {
            name: "vulkan-icd-loader".into(),
            dep_type: DepType::SharedObject,
            filename: "libvulkan.so".into(),
        },
        Dependency {
            name: "wayland".into(),
            dep_type: DepType::SharedObject,
            filename: "libwayland-client.so".into(),
        },
        Dependency {
            name: "cmake".into(),
            dep_type: DepType::Executable,
            filename: "cmake".into(),
        },
        Dependency {
            name: "eigen".into(),
            dep_type: DepType::Include,
            filename: "eigen3/Eigen/src/Core/EigenBase.h".into(),
        },
        Dependency {
            name: "git".into(),
            dep_type: DepType::Executable,
            filename: "git".into(),
        },
        Dependency {
            name: "ninja".into(),
            dep_type: DepType::Executable,
            filename: "ninja".into(),
        },
        Dependency {
            name: "shaderc".into(),
            dep_type: DepType::Executable,
            filename: "glslc".into(),
        },
        Dependency {
            name: "vulkan-headers".into(),
            dep_type: DepType::Include,
            filename: "vulkan/vulkan.h".into(),
        },
        Dependency {
            name: "vulkan-headers".into(),
            dep_type: DepType::Include,
            filename: "vulkan/vulkan.h".into(),
        },
    ]
}

pub fn check_monado_deps() -> Vec<DependencyCheckResult> {
    check_dependencies(monado_deps())
}
