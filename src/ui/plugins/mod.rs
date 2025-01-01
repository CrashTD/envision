pub mod add_custom_plugin_win;
pub mod store;
mod store_detail;
mod store_row_factory;

use crate::{paths::get_plugins_dir, util::file_utils::mark_as_executable};
use anyhow::bail;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct Plugin {
    pub appid: String,
    pub name: String,
    pub icon_url: Option<String>,
    pub version: Option<String>,
    pub short_description: Option<String>,
    pub description: Option<String>,
    pub hompage_url: Option<String>,
    pub screenshots: Vec<String>,
    /// either one of exec_url or exec_path must be provided
    pub exec_url: Option<String>,
    /// either one of exec_url or exec_path must be provided
    pub exec_path: Option<PathBuf>,
}

impl Plugin {
    pub fn executable(&self) -> Option<PathBuf> {
        if self.exec_path.is_some() {
            self.exec_path.clone()
        } else {
            let canonical = self.canonical_exec_path();
            if canonical.is_file() {
                Some(canonical)
            } else {
                None
            }
        }
    }

    pub fn canonical_exec_path(&self) -> PathBuf {
        get_plugins_dir().join(&self.appid)
    }

    pub fn is_installed(&self) -> bool {
        self.executable().as_ref().is_some_and(|p| p.is_file())
    }

    pub fn mark_as_executable(&self) -> anyhow::Result<()> {
        if let Some(p) = self.executable().as_ref() {
            mark_as_executable(p)
        } else {
            bail!("no executable found for plugin")
        }
    }

    /// validate if the plugin can be displayed correctly and run
    pub fn validate(&self) -> bool {
        !self.appid.is_empty()
            && !self.name.is_empty()
            && self.executable().as_ref().is_some_and(|p| p.is_file())
    }
}
