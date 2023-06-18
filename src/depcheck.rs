use expect_dialog::ExpectDialog;
use std::{env, fmt::Display, path::Path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DepType {
    SharedObject,
    Executable,
    Include,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dependency {
    pub name: String,
    pub dep_type: DepType,
    pub filename: String,
}

impl Ord for Dependency {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for Dependency {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyCheckResult {
    pub dependency: Dependency,
    pub found: bool,
}

impl Display for DependencyCheckResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{name} {result}",
            name = self.dependency.name,
            result = match self.found {
                true => "Found",
                false => "Not found",
            }
        ))
    }
}

fn shared_obj_paths() -> Vec<String> {
    vec![
        "/lib".to_string(),
        "/usr/lib".to_string(),
        "/usr/lib64".to_string(),
        "/usr/local/lib".to_string(),
        "/usr/local/lib64".to_string(),
        "/usr/lib/x86_64-linux-gnu".to_string(),
        "/usr/lib/aarch64-linux-gnu".to_string(),
        "/lib/x86_64-linux-gnu".to_string(),
        "/lib/aarch64-linux-gnu".to_string(),
        "/app/lib".to_string(),
    ]
}

fn include_paths() -> Vec<String> {
    vec!["/usr/include".to_string(), "/usr/local/include".to_string()]
}

pub fn check_dependency(dep: Dependency) -> bool {
    for dir in match dep.dep_type {
        DepType::SharedObject => shared_obj_paths(),
        DepType::Executable => env::var("PATH")
            .expect_dialog("PATH environment variable not defined")
            .split(':')
            .map(str::to_string)
            .collect(),
        DepType::Include => include_paths(),
    } {
        let path_s = &format!("{dir}/{fname}", dir = dir, fname = dep.filename);
        let path = Path::new(&path_s);
        if path.is_file() {
            return true;
        }
    }
    return false;
}

pub fn check_dependencies(deps: Vec<Dependency>) -> Vec<DependencyCheckResult> {
    deps.iter()
        .map(|dep| DependencyCheckResult {
            dependency: dep.clone(),
            found: check_dependency(dep.clone()),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{check_dependency, DepType, Dependency};

    #[test]
    fn can_find_libc() {
        let libc_dep = Dependency {
            name: "libc".into(),
            dep_type: DepType::SharedObject,
            filename: "libc.so".into(),
        };

        assert!(check_dependency(libc_dep));
    }

    #[test]
    fn can_find_ls() {
        let libc_dep = Dependency {
            name: "ls".into(),
            dep_type: DepType::Executable,
            filename: ("ls".into()),
        };

        assert!(check_dependency(libc_dep));
    }

    #[test]
    fn cannot_find_fake_lib() {
        let fake_dep = Dependency {
            name: "fakedep".into(),
            dep_type: DepType::SharedObject,
            filename: "fakedep.so".into(),
        };

        assert!(!check_dependency(fake_dep));
    }
}
