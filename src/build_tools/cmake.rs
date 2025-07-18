use crate::ui::job_worker::job::WorkerJob;
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, Clone)]
pub struct Cmake {
    pub build_dir: PathBuf,
    pub source_dir: PathBuf,
    pub vars: Option<HashMap<String, String>>,
    pub env: Option<HashMap<String, String>>,
}

impl Cmake {
    pub fn get_prepare_job(&self) -> WorkerJob {
        let mut args = vec![
            "-B".into(),
            self.build_dir.to_string_lossy().to_string(),
            "-G".into(),
            "Ninja".into(),
        ];
        if self.vars.is_some() {
            for (k, v) in self.vars.as_ref().unwrap() {
                if k.contains(' ') {
                    panic!("Cmake vars cannot contain spaces!");
                }
                args.push(format!("-D{k}={v}"));
            }
        }
        args.push(self.source_dir.to_string_lossy().to_string());
        WorkerJob::new_cmd(self.env.clone(), "cmake".into(), Some(args))
    }

    pub fn get_build_job(&self) -> WorkerJob {
        WorkerJob::new_cmd(
            self.env.clone(),
            "cmake".into(),
            Some(vec![
                "--build".into(),
                self.build_dir.to_string_lossy().to_string(),
            ]),
        )
    }

    pub fn get_install_job(&self) -> WorkerJob {
        WorkerJob::new_cmd(
            self.env.clone(),
            "cmake".into(),
            Some(vec![
                "--install".into(),
                self.build_dir.to_string_lossy().to_string(),
            ]),
        )
    }
}
