use crate::cmd_runner::CmdRunner;
use std::path::Path;

pub fn get_adb_install_runner(apk_path: &String) -> CmdRunner {
    let path = Path::new(apk_path);
    path.try_exists().expect("APK file provided does not exist");
    CmdRunner::new(
        None,
        "adb".into(),
        vec!["install".into(), path.to_str().unwrap().to_string()],
    )
}
