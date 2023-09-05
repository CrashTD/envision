use crate::{cmd_runner::CmdRunner, ui::workers::runner_worker::WorkerJob};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Cmake {
    pub build_dir: String,
    pub source_dir: String,
    pub vars: Option<HashMap<String, String>>,
    pub env: Option<HashMap<String, String>>,
}

impl Cmake {
    #[deprecated]
    pub fn get_prepare_runner(&self) -> CmdRunner {
        let mut args = vec![
            "-B".into(),
            self.build_dir.clone(),
            "-G".into(),
            "Ninja".into(),
        ];
        if self.vars.is_some() {
            for (k, v) in self.vars.as_ref().unwrap() {
                if k.contains(' ') {
                    panic!("Cmake vars cannot contain spaces!");
                }
                if v.contains(' ') {
                    args.push(format!("-D{k}=\"{v}\"", k = k, v = v));
                } else {
                    args.push(format!("-D{k}={v}", k = k, v = v));
                }
            }
        }
        args.push(self.source_dir.clone());
        CmdRunner::new(self.env.clone(), "cmake".into(), args)
    }

    pub fn get_prepare_job(&self) -> WorkerJob {
        let mut args = vec![
            "-B".into(),
            self.build_dir.clone(),
            "-G".into(),
            "Ninja".into(),
        ];
        if self.vars.is_some() {
            for (k, v) in self.vars.as_ref().unwrap() {
                if k.contains(' ') {
                    panic!("Cmake vars cannot contain spaces!");
                }
                if v.contains(' ') {
                    args.push(format!("-D{k}=\"{v}\"", k = k, v = v));
                } else {
                    args.push(format!("-D{k}={v}", k = k, v = v));
                }
            }
        }
        args.push(self.source_dir.clone());
        WorkerJob::new_cmd(self.env.clone(), "cmake".into(), Some(args))
    }

    #[deprecated]
    pub fn get_build_runner(&self) -> CmdRunner {
        CmdRunner::new(
            self.env.clone(),
            "cmake".into(),
            vec!["--build".into(), self.build_dir.clone()],
        )
    }
    pub fn get_build_job(&self) -> WorkerJob {
        WorkerJob::new_cmd(
            self.env.clone(),
            "cmake".into(),
            Some(vec!["--build".into(), self.build_dir.clone()]),
        )
    }

    #[deprecated]
    pub fn get_install_runner(&self) -> CmdRunner {
        CmdRunner::new(
            self.env.clone(),
            "cmake".into(),
            vec!["--install".into(), self.build_dir.clone()],
        )
    }

    pub fn get_install_job(&self) -> WorkerJob {
        WorkerJob::new_cmd(
            self.env.clone(),
            "cmake".into(),
            Some(vec!["--install".into(), self.build_dir.clone()]),
        )
    }
}
