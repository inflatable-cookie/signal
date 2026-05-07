//! Plugin discovery domain. Holds discovered plugin descriptors and scan receipts, shared between the inventory scanner and the plugin library.

#![warn(missing_docs)]

use serde::{Deserialize, Serialize};

/// Full descriptor for a plugin discovered during a scan. Contains identity, hostability, and origin metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscoveredPlugin {
    /// Unique stable identifier for this plugin.
    pub plugin_id: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Plugin vendor name.
    pub vendor_name: String,
    /// Plugin kind string (e.g. `Instrument`, `Effect`).
    pub plugin_kind: String,
    /// Primary plugin format (e.g. `clap`, `vst3`).
    pub primary_format: String,
    /// Format-specific plugin identifier, if applicable.
    pub format_specific_id: Option<String>,
    /// Fingerprint of the installed binary, used for staleness detection.
    pub install_fingerprint: String,
    /// Source that produced this entry (e.g. scan root path).
    pub scan_source: String,
    /// Path to the plugin binary on disk.
    pub binary_path: String,
    /// Path to the plugin bundle directory, if applicable.
    pub bundle_path: Option<String>,
    /// CPU architecture string, if known.
    pub architecture: Option<String>,
    /// Version string from the plugin metadata, if available.
    pub version_text: Option<String>,
    /// Categories declared by the plugin vendor.
    pub vendor_supplied_categories: Vec<String>,
    /// Health state string for the plugin.
    pub health_state: String,
    /// Whether the plugin can be loaded natively (same architecture/platform).
    pub native_hostable: bool,
    /// Whether the plugin can be loaded via an architecture bridge.
    pub bridge_hostable: bool,
    /// Runtime origin string describing how this plugin was discovered.
    pub runtime_origin: String,
    /// Bridge source identifier, if bridge hosting is used.
    pub bridge_source: Option<String>,
    /// Human-readable explanation for the hostability determination, if applicable.
    pub hostability_reason: Option<String>,
}

/// Lightweight inventory entry for a plugin, suitable for list views without carrying full discovery metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InventoryListItem {
    /// Unique stable identifier for this plugin.
    pub plugin_id: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Plugin vendor name.
    pub vendor_name: String,
    /// Primary plugin format string.
    pub primary_format: String,
    /// Format-specific plugin identifier, if applicable.
    pub format_specific_id: Option<String>,
    /// Fingerprint of the installed binary.
    pub install_fingerprint: String,
    /// Version string from the plugin metadata, if available.
    pub version_text: Option<String>,
    /// Path to the plugin binary on disk.
    pub binary_path: String,
    /// Whether the plugin can be loaded natively.
    pub native_hostable: bool,
    /// Whether the plugin can be loaded via a bridge.
    pub bridge_hostable: bool,
    /// Runtime origin string.
    pub runtime_origin: String,
    /// Health state string.
    pub health_state: String,
}

/// Receipt produced at the end of a scan run: the mode, all roots checked, and the full list of discovered plugins.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScanReceipt {
    /// Scan mode string (e.g. `full`, `incremental`).
    pub scan_mode: String,
    /// Filesystem roots that were checked during the scan.
    pub roots_checked: Vec<String>,
    /// All plugins discovered during the scan.
    pub discovered_plugins: Vec<DiscoveredPlugin>,
}
