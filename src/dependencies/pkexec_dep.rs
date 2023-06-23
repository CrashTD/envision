use crate::depcheck::{DepType, Dependency};

pub fn pkexec_dep() -> Dependency {
    Dependency {
        name: "pkexec".into(),
        dep_type: DepType::Executable,
        filename: "pkexec".into(),
    }
}
