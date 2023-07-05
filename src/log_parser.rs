use crate::log_level::LogLevel;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonadoLog {
    pub level: LogLevel,
    pub file: String,
    pub func: String,
    pub message: String,
}

impl MonadoLog {
    pub fn from_str(s: &str) -> Option<Self> {
        serde_json::from_str::<Self>(s).ok()
    }
}
