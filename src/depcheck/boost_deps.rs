use crate::linux_distro::LinuxDistro;

use super::{DepType, Dependency};
use std::collections::HashMap;

pub fn boost_deps() -> Vec<Dependency> {
    [
        // ("", "libboost_atomic-devel"),
        // ("", "libboost_chrono-devel"),
        // ("", "libboost_container-devel"),
        // ("", "libboost_context-devel"),
        // ("", "libboost_contract-devel"),
        // ("", "libboost_coroutine-devel"),
        ("libboost_date_time.so", "libboost_date_time-devel"),
        // ("", "libboost_fiber-devel"),
        ("libboost_filesystem.so", "libboost_filesystem-devel"),
        // ("", "libboost_graph-devel"),
        // ("", "libboost_graph_parallel-devel"),
        // ("", "libboost_headers-devel"),
        // ("", "libboost_iostreams-devel"),
        // ("", "libboost_locale-devel"),
        // ("", "libboost_log-devel"),
        // ("", "libboost_math-devel"),
        // ("", "libboost_mpi-devel"),
        // ("", "libboost_mpi_python3-devel"),
        // ("", "libboost_nowide-devel"),
        // ("", "libboost_numpy3-devel"),
        // ("", "libboost_process-devel"),
        (
            "libboost_program_options.so",
            "libboost_program_options-devel",
        ),
        // ("", "libboost_python3-devel"),
        // ("", "libboost_random-devel"),
        ("libboost_regex.so", "libboost_regex-devel"),
        ("libboost_serialization.so", "libboost_serialization-devel"),
        // ("", "libboost_stacktrace-devel"),
        ("libboost_system.so", "libboost_system-devel"),
        // ("", "libboost_test-devel"),
        // ("", "libboost_thread-devel"),
        // ("", "libboost_timer-devel"),
        // ("", "libboost_type_erasure-devel"),
        // ("", "libboost_url1_86_0-devel"),
        // ("", "libboost_wave-devel"),
    ]
    .into_iter()
    .map(|(name, package)| Dependency {
        name: "boost".into(),
        dep_type: DepType::SharedObject,
        // just one of the many shared objects boost provides
        filename: name.into(),
        packages: HashMap::from([
            (LinuxDistro::Arch, "boost".into()),
            (LinuxDistro::Debian, "libboost-all-dev".into()),
            (LinuxDistro::Fedora, "boost".into()),
            (LinuxDistro::Alpine, "boost".into()),
            (LinuxDistro::Gentoo, "dev-libs/boost".into()),
            (LinuxDistro::Suse, package.into()),
        ]),
    })
    .collect()
}
