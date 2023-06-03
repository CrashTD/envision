use std::path::Path;

pub enum DepType {
    SharedObject(String),
}

pub struct Dependency {
    pub name: String,
    pub dep_type: DepType,
}

fn shared_obj_paths() -> Vec<String> {
    vec![
        "/lib".to_string(),
        "/usr/lib".to_string(),
        "/usr/local/lib".to_string(),
        "/usr/lib/x86_64-linux-gnu".to_string(),
        "/usr/lib/aarch64-linux-gnu".to_string(),
        "/lib/x86_64-linux-gnu".to_string(),
        "/lib/aarch64-linux-gnu".to_string(),
        "/app/lib".to_string(),
    ]
}

pub fn check_dependency(dep: Dependency) -> bool {
    match dep.dep_type {
        DepType::SharedObject(filename) => {
            for lib_path in shared_obj_paths() {
                let path_s = &format!("{lp}/{fn}", lp=lib_path, fn=filename);
                let path = Path::new(&path_s);
                if path.is_file() {
                    return true;
                }
            }
            return false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Dependency, DepType, check_dependency};

    #[test]
    fn can_find_libc() {
        let libc_dep = Dependency {
            name: "libc".into(),
            dep_type: DepType::SharedObject("libc.so".into()),
        };

        assert!(check_dependency(libc_dep));
    }

    #[test]
    fn cannot_find_fake_lib() {
        let fake_dep = Dependency {
            name: "fakedep".into(),
            dep_type: DepType::SharedObject("fakedep.so".into()),
        };

        assert!(!check_dependency(fake_dep));
    }
}
