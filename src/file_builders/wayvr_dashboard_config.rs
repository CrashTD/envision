use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WayVrDashboardConfigFragmentInner {
    pub exec: String,
    pub args: Option<String>,
    pub env: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WayVrDashboardConfigFragment {
    pub dashboard: WayVrDashboardConfigFragmentInner,
}
