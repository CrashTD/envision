use std::path::Path;
use crate::runner::Runner;

pub fn get_adb_install_runner(apk_path: &String) -> Runner {
    let path = Path::new(apk_path);
    path.try_exists().expect("APK file provided does not exist");
    let runner = Runner::new(None, "adb".into(), vec![
        "install".into(), path.to_str().unwrap().to_string()
    ]);
    runner
}
