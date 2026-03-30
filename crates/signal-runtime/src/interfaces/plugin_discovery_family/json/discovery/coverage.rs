use super::super::*;
#[path = "coverage/capability.rs"]
mod capability;
#[path = "coverage/format_parity.rs"]
mod format_parity;

pub(super) fn json_runtime_plugin_capability_coverage_summary(
    summary: &RuntimePluginCapabilityCoverageSummary,
) -> String {
    capability::json_runtime_plugin_capability_coverage_summary(summary)
}

pub(super) fn json_runtime_plugin_complex_io_summary(
    summary: &RuntimePluginComplexIoSummary,
) -> String {
    capability::json_runtime_plugin_complex_io_summary(summary)
}

pub(super) fn json_runtime_plugin_format_coverage_vec(
    records: &[RuntimePluginFormatCoverageRecord],
) -> String {
    format_parity::json_runtime_plugin_format_coverage_vec(records)
}

pub(super) fn json_runtime_plugin_parity_coverage_vec(
    records: &[RuntimePluginFormatParityRecord],
) -> String {
    format_parity::json_runtime_plugin_parity_coverage_vec(records)
}
