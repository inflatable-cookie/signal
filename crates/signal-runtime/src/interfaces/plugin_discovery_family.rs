use super::*;

mod catalog;
mod json;
mod lv2;

pub use catalog::*;
pub use lv2::*;

pub(super) fn json_runtime_plugin_discovery_snapshot(
    snapshot: &RuntimePluginDiscoverySnapshot,
) -> String {
    json::json_runtime_plugin_discovery_snapshot(snapshot)
}

pub(super) fn json_runtime_lv2_extension_snapshot(
    snapshot: &RuntimeLv2ExtensionSnapshot,
) -> String {
    json::json_runtime_lv2_extension_snapshot(snapshot)
}

pub(super) fn json_runtime_plugin_complex_io_summary(
    summary: &RuntimePluginComplexIoSummary,
) -> String {
    json::json_runtime_plugin_complex_io_summary(summary)
}

pub(super) fn json_runtime_plugin_parity_coverage_vec(
    records: &[RuntimePluginFormatParityRecord],
) -> String {
    json::json_runtime_plugin_parity_coverage_vec(records)
}

pub(super) fn json_runtime_plugin_pin_group_identity_vec(
    values: &[RuntimePluginPinGroupIdentity],
) -> String {
    json::json_runtime_plugin_pin_group_identity_vec(values)
}
