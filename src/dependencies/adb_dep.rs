use crate::depcheck::{DepType, Dependency};

pub fn adb_dep() -> Dependency {
    Dependency {
        name: "adb".into(),
        dep_type: DepType::Executable,
        filename: "adb".into(),
    }
}
