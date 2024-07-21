use crate::ui::job_worker::job::{FuncWorkerOut, WorkerJob};
use git2::Repository;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Git {
    pub repo: String,
    pub dir: PathBuf,
    pub branch: String,
}

impl Git {
    fn cmd(&self, args: Vec<String>) -> WorkerJob {
        let mut nargs = vec!["-C".into(), self.dir.to_string_lossy().to_string()];
        nargs.extend(args);
        WorkerJob::new_cmd(None, "git".into(), Some(nargs))
    }

    fn get_repo(&self) -> String {
        self.repo
            .split('#')
            .next()
            .expect("Could not get repo url")
            .into()
    }

    fn get_ref(&self) -> Option<String> {
        let mut split = self.repo.split('#');
        split.next().expect("Could not get repo url");
        split.next().map(|s| s.into())
    }

    pub fn get_reset_job(&self) -> WorkerJob {
        self.cmd(vec!["reset".into(), "--hard".into()])
    }

    pub fn get_override_remote_url_job(&self) -> WorkerJob {
        let dir = self.dir.clone();
        let n_remote_url = self.get_repo();
        WorkerJob::new_func(Box::new(move || {
            if let Ok(repo) = Repository::open(dir) {
                if let Ok(remote) = repo.find_remote("origin") {
                    if remote.url().unwrap_or("") != n_remote_url {
                        if repo.remote_set_url("origin", &n_remote_url).is_ok() {
                            return FuncWorkerOut {
                                success: true,
                                out: vec![],
                            };
                        }
                        return FuncWorkerOut {
                            success: false,
                            out: vec!["Failed to set origin remote url".into()],
                        };
                    }
                } else {
                    return FuncWorkerOut {
                        success: false,
                        out: vec!["Could not find remote origin".into()],
                    };
                }
                return FuncWorkerOut {
                    success: true,
                    out: vec![],
                };
            }
            FuncWorkerOut {
                success: true,
                out: vec![],
            }
        }))
    }

    pub fn get_pull_job(&self) -> WorkerJob {
        self.cmd(vec!["pull".into()])
    }

    pub fn get_clone_job(&self) -> WorkerJob {
        WorkerJob::new_cmd(
            None,
            "git".into(),
            Some(vec![
                "clone".into(),
                self.get_repo(),
                self.dir.to_string_lossy().to_string(),
                "--recurse-submodules".into(),
            ]),
        )
    }

    pub fn get_checkout_ref_job(&self) -> WorkerJob {
        let gref = self.get_ref().unwrap_or(self.branch.clone());
        self.cmd(vec!["checkout".into(), gref])
    }

    pub fn get_fetch_job(&self) -> WorkerJob {
        self.cmd(vec!["fetch".into()])
    }

    pub fn get_merge_job(&self) -> WorkerJob {
        self.cmd(vec!["merge".into(), "--ff-only".into()])
    }

    pub fn clone_exists(&self) -> bool {
        self.dir.join(".git").is_dir()
    }

    pub fn get_pre_build_jobs(&self, pull_on_build: bool) -> Vec<WorkerJob> {
        let mut jobs = Vec::<WorkerJob>::new();
        if self.clone_exists() {
            if pull_on_build {
                jobs.push(self.get_override_remote_url_job());
                jobs.push(self.get_fetch_job());
                jobs.push(self.get_checkout_ref_job());
                jobs.push(self.get_pull_job());
            }
        } else {
            jobs.push(self.get_clone_job());
            jobs.push(self.get_checkout_ref_job());
        }
        jobs
    }
}
