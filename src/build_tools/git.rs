use std::path::Path;

use crate::{profile::Profile, runner::Runner};

#[derive(Debug, Clone)]
pub struct Git {
    pub repo: String,
    pub dir: String,
}

impl Git {
    pub fn get_reset_runner(&self) -> Runner {
        Runner::new(
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

    pub fn get_pull_runner(&self) -> Runner {
        Runner::new(
            None,
            "git".into(),
            vec!["-C".into(), self.dir.clone(), "pull".into()],
        )
    }

    pub fn get_clone_runner(&self) -> Runner {
        Runner::new(
            None,
            "git".into(),
            vec!["clone".into(), self.repo.clone(), self.dir.clone(), "--recurse-submodules".into()],
        )
    }

    pub fn get_clone_or_not_runner(&self) -> Option<Runner> {
        let path_s = format!("{}/.git", self.dir.clone());
        let path = Path::new(&path_s);
        if path.is_dir() {
            return None;
        }
        Some(self.get_clone_runner())
    }

    pub fn clone_or_pull(&self, profile: &Profile) -> Option<Runner> {
        match self.get_clone_or_not_runner() {
            Some(r) => Some(r),
            None => match profile.pull_on_build {
                true => Some(self.get_pull_runner()),
                false => None,
            },
        }
    }
}
