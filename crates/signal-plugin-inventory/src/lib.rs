//! Shared plugin inventory domain for Signal consumers.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscoveredPlugin {
    pub plugin_id: String,
    pub display_name: String,
    pub vendor_name: String,
    pub plugin_kind: String,
    pub primary_format: String,
    pub format_specific_id: Option<String>,
    pub install_fingerprint: String,
    pub scan_source: String,
    pub binary_path: String,
    pub bundle_path: Option<String>,
    pub architecture: Option<String>,
    pub version_text: Option<String>,
    pub vendor_supplied_categories: Vec<String>,
    pub health_state: String,
    pub native_hostable: bool,
    pub bridge_hostable: bool,
    pub runtime_origin: String,
    pub bridge_source: Option<String>,
    pub hostability_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InventoryListItem {
    pub plugin_id: String,
    pub display_name: String,
    pub vendor_name: String,
    pub primary_format: String,
    pub format_specific_id: Option<String>,
    pub install_fingerprint: String,
    pub version_text: Option<String>,
    pub binary_path: String,
    pub native_hostable: bool,
    pub bridge_hostable: bool,
    pub runtime_origin: String,
    pub health_state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScanReceipt {
    pub scan_mode: String,
    pub roots_checked: Vec<String>,
    pub discovered_plugins: Vec<DiscoveredPlugin>,
}
