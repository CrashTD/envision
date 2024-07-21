use std::path::PathBuf;

use crate::cmd_runner::CmdRunner;

pub fn get_adb_install_runner(path: &PathBuf) -> CmdRunner {
    path.try_exists().expect("APK file provided does not exist");
    CmdRunner::new(
        None,
        "adb".into(),
        vec!["install".into(), path.to_string_lossy().to_string()],
    )
}
