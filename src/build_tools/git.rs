use crate::{
    cmd_runner::CmdRunner,
    func_runner::{FuncRunner, FuncRunnerOut},
    profile::Profile,
};
use git2::Repository;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Git {
    pub repo: String,
    pub dir: String,
}

impl Git {
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

    pub fn get_reset_runner(&self) -> CmdRunner {
        CmdRunner::new(
            None,
            "git".into(),
            vec![
                "-C".into(),
                self.dir.clone(),
                "reset".into(),
                "--hard".into(),
            ],
        )
    }

    pub fn get_override_remote_url_runner(&self) -> FuncRunner {
        let dir = self.dir.clone();
        let n_remote_url = self.get_repo();
        FuncRunner::new(Box::new(move || {
            if let Ok(repo) = Repository::open(dir) {
                if let Ok(remote) = repo.find_remote("origin") {
                    if remote.url().unwrap_or("") != n_remote_url {
                        if repo.remote_set_url("origin", &n_remote_url).is_ok() {
                            return FuncRunnerOut {
                                success: true,
                                out: vec![],
                            };
                        }
                        return FuncRunnerOut {
                            success: false,
                            out: vec!["Failed to set origin remote url".into()],
                        };
                    }
                } else {
                    return FuncRunnerOut {
                        success: false,
                        out: vec!["Could not find remote origin".into()],
                    };
                }
                return FuncRunnerOut {
                    success: true,
                    out: vec![],
                };
            }
            FuncRunnerOut {
                success: true,
                out: vec![],
            }
        }))
    }

    pub fn get_pull_runner(&self) -> CmdRunner {
        CmdRunner::new(
            None,
            "git".into(),
            vec!["-C".into(), self.dir.clone(), "pull".into()],
        )
    }

    pub fn get_clone_runner(&self) -> CmdRunner {
        CmdRunner::new(
            None,
            "git".into(),
            vec![
                "clone".into(),
                self.get_repo(),
                self.dir.clone(),
                "--recurse-submodules".into(),
            ],
        )
    }

    pub fn get_checkout_ref_runner(&self) -> Option<CmdRunner> {
        self.get_ref().map(|r| {
            CmdRunner::new(
                None,
                "git".into(),
                vec!["-C".into(), self.dir.clone(), "checkout".into(), r],
            )
        })
    }

    pub fn get_clone_or_not_runner(&self) -> Option<CmdRunner> {
        let path_s = format!("{}/.git", self.dir.clone());
        let path = Path::new(&path_s);
        if path.is_dir() {
            return None;
        }
        Some(self.get_clone_runner())
    }

    pub fn clone_or_pull(&self, profile: &Profile) -> Option<CmdRunner> {
        match self.get_clone_or_not_runner() {
            Some(r) => Some(r),
            None => match profile.pull_on_build {
                true => Some(self.get_pull_runner()),
                false => None,
            },
        }
    }
}
