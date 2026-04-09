use super::*;
#[path = "discovery/coverage.rs"]
mod coverage;

use coverage::{
    json_runtime_plugin_capability_coverage_summary, json_runtime_plugin_format_coverage_vec,
};

pub(super) fn json_runtime_plugin_discovery_snapshot(
    snapshot: &RuntimePluginDiscoverySnapshot,
) -> String {
    let last_scan = snapshot
        .last_scan
        .as_ref()
        .map(json_runtime_plugin_scan_receipt)
        .unwrap_or_else(|| "null".into());
    format!(
        concat!(
            "{{",
            "\"scan_count\":{},",
            "\"format_filtered_scan_count\":{},",
            "\"discovered_type_count\":{},",
            "\"discovered_format_count\":{},",
            "\"last_scan\":{},",
            "\"format_coverage\":{},",
            "\"parity_coverage\":{},",
            "\"capability_coverage\":{},",
            "\"discovered_types\":{},",
            "\"summary\":{}",
            "}}"
        ),
        snapshot.scan_count,
        snapshot.format_filtered_scan_count,
        snapshot.discovered_type_count,
        snapshot.discovered_format_count,
        last_scan,
        json_runtime_plugin_format_coverage_vec(&snapshot.format_coverage),
        json_runtime_plugin_parity_coverage_vec(&snapshot.parity_coverage),
        json_runtime_plugin_capability_coverage_summary(&snapshot.capability_coverage),
        super::detail::json_runtime_plugin_discovered_type_record_vec(&snapshot.discovered_types),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

fn json_runtime_plugin_scan_receipt(receipt: &RuntimePluginScanReceipt) -> String {
    format!(
        concat!(
            "{{",
            "\"scan_handle\":{},",
            "\"roots\":{},",
            "\"formats\":{},",
            "\"targeted_format_count\":{},",
            "\"discovered_type_count\":{},",
            "\"discovered_format_count\":{},",
            "\"discovery_diagnostic_count\":{},",
            "\"discovery_diagnostics\":{},",
            "\"format_coverage\":{},",
            "\"parity_coverage\":{},",
            "\"capability_coverage\":{},",
            "\"summary\":{}",
            "}}"
        ),
        receipt.scan_handle.0,
        json_string_vec(&receipt.roots),
        json_plugin_format_vec(&receipt.formats),
        receipt.targeted_format_count,
        receipt.discovered_type_count,
        receipt.discovered_format_count,
        receipt.discovery_diagnostic_count,
        json_runtime_plugin_scan_diagnostic_vec(&receipt.discovery_diagnostics),
        json_runtime_plugin_format_coverage_vec(&receipt.format_coverage),
        json_runtime_plugin_parity_coverage_vec(&receipt.parity_coverage),
        json_runtime_plugin_capability_coverage_summary(&receipt.capability_coverage),
        json_option_string(Some(receipt.summary.as_str())),
    )
}

fn json_runtime_plugin_scan_diagnostic_vec(
    diagnostics: &[RuntimePluginScanDiagnosticRecord],
) -> String {
    format!(
        "[{}]",
        diagnostics
            .iter()
            .map(json_runtime_plugin_scan_diagnostic)
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn json_runtime_plugin_scan_diagnostic(diagnostic: &RuntimePluginScanDiagnosticRecord) -> String {
    format!(
        concat!(
            "{{",
            "\"format\":{},",
            "\"root\":{},",
            "\"bundle_root\":{},",
            "\"manifest_path\":{},",
            "\"plugin_type_id\":{},",
            "\"kind\":{},",
            "\"detail\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_escape_string(&format!("{:?}", diagnostic.format)),
        json_escape_string(&diagnostic.root),
        json_escape_string(&diagnostic.bundle_root),
        json_option_string(diagnostic.manifest_path.as_deref()),
        json_option_string(diagnostic.plugin_type_id.as_deref()),
        json_escape_string(&format!("{:?}", diagnostic.kind)),
        json_escape_string(&diagnostic.detail),
        json_option_string(Some(diagnostic.summary.as_str())),
    )
}

pub(super) fn json_runtime_plugin_pin_group_identity_vec(
    values: &[RuntimePluginPinGroupIdentity],
) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| json_escape_string(&format!("{value:?}")))
            .collect::<Vec<_>>()
            .join(",")
    )
}

pub(super) fn json_runtime_plugin_parity_coverage_vec(
    records: &[RuntimePluginFormatParityRecord],
) -> String {
    coverage::json_runtime_plugin_parity_coverage_vec(records)
}

pub(super) fn json_runtime_plugin_complex_io_summary(
    summary: &RuntimePluginComplexIoSummary,
) -> String {
    coverage::json_runtime_plugin_complex_io_summary(summary)
}
