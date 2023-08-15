use std::path::Path;

use crate::{profile::Profile, runner::Runner};

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
            vec![
                "clone".into(),
                self.get_repo(),
                self.dir.clone(),
                "--recurse-submodules".into(),
            ],
        )
    }

    pub fn get_checkout_ref_runner(&self) -> Option<Runner> {
        self.get_ref().map(|r| {
            Runner::new(
                None,
                "git".into(),
                vec!["-C".into(), self.dir.clone(), "checkout".into(), r],
            )
        })
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
