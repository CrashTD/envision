use crate::termcolor::TermColor;
use serde::{Deserialize, Serialize, de::Visitor};
use std::{fmt::Display, slice::Iter, str::FromStr};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warning,
    Error,
}

struct LogLevelStringVisitor;
impl<'de> Visitor<'de> for LogLevelStringVisitor {
    type Value = LogLevel;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(
            "A case-insensitive string among trace, debug, info, warning, warn, error, err",
        )
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(LogLevel::from_str(v).unwrap())
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(LogLevel::from_str(&v).unwrap())
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(LogLevel::from_str(v).unwrap())
    }
}

impl<'de> Deserialize<'de> for LogLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_string(LogLevelStringVisitor)
    }
}

impl FromStr for LogLevel {
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "trace" => Self::Trace,
            "debug" => Self::Debug,
            "info" => Self::Info,
            "warning" => Self::Warning,
            "warn" => Self::Warning,
            "error" => Self::Error,
            "err" => Self::Error,
            _ => Self::Debug,
        })
    }

    type Err = ();
}

impl LogLevel {
    pub fn iter() -> Iter<'static, LogLevel> {
        [
            Self::Trace,
            Self::Debug,
            Self::Info,
            Self::Warning,
            Self::Error,
        ]
        .iter()
    }

    pub fn colored(&self) -> String {
        match self {
            Self::Trace => TermColor::Gray,
            Self::Debug => TermColor::Gray,
            Self::Info => TermColor::Blue,
            Self::Warning => TermColor::Yellow,
            Self::Error => TermColor::Red,
        }
        .colorize(&self.to_string())
    }
}

impl From<&LogLevel> for u32 {
    fn from(value: &LogLevel) -> Self {
        match value {
            LogLevel::Trace => 0,
            LogLevel::Debug => 1,
            LogLevel::Info => 2,
            LogLevel::Warning => 3,
            LogLevel::Error => 99,
        }
    }
}

impl PartialOrd for LogLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LogLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        u32::from(self).cmp(&other.into())
    }
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            LogLevel::Trace => "Trace",
            LogLevel::Debug => "Debug",
            LogLevel::Info => "Info",
            LogLevel::Warning => "Warning",
            LogLevel::Error => "Error",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonadoLog {
    pub level: LogLevel,
    pub file: String,
    pub func: String,
    pub message: String,
}

impl MonadoLog {
    pub fn new_from_str(s: &str) -> Option<Self> {
        serde_json::from_str::<Self>(s).ok()
    }
}
