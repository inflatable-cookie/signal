use super::*;

pub(crate) fn format_runtime_plugin_discovery_snapshot_compact(
    snapshot: &RuntimePluginDiscoverySnapshot,
) -> String {
    format!(
        " plugin_scans={} plugin_filtered_scans={} plugin_discovered_types={} plugin_discovered_formats={} plugin_parity_formats={} plugin_capability_coverage={} plugin_last_scan={}",
        snapshot.scan_count,
        snapshot.format_filtered_scan_count,
        snapshot.discovered_type_count,
        snapshot.discovered_format_count,
        snapshot.parity_coverage.len(),
        snapshot.capability_coverage.summary,
        snapshot
            .last_scan
            .as_ref()
            .map(|scan| scan.summary.as_str())
            .unwrap_or("none"),
    )
}

pub(crate) fn format_runtime_plugin_discovery_snapshot_multiline(
    snapshot: &RuntimePluginDiscoverySnapshot,
) -> String {
    let last_scan = snapshot
        .last_scan
        .as_ref()
        .map(|scan| {
            format!(
                "\nplugin_last_scan_handle={}\nplugin_last_scan_roots={:?}\nplugin_last_scan_formats={:?}\nplugin_last_scan_targeted_format_count={}\nplugin_last_scan_discovered_type_count={}\nplugin_last_scan_summary={}",
                scan.scan_handle.0,
                scan.roots,
                scan.formats,
                scan.targeted_format_count,
                scan.discovered_type_count,
                scan.summary,
            )
        })
        .unwrap_or_default();
    let format_coverage_lines = snapshot
        .format_coverage
        .iter()
        .enumerate()
        .map(|(index, coverage)| {
            format!(
                "\nplugin_format_coverage_{}={:?}/types={}/features={}/{}/{}/{}/{} snapshot={} prepare={} activate={} midi_in={} note_expression={} midi_out={} max_audio_buses={} max_parameters={}",
                index,
                coverage.format,
                coverage.discovered_type_count,
                coverage.audio_effect_count,
                coverage.instrument_count,
                coverage.analyzer_count,
                coverage.utility_count,
                coverage.note_effect_count,
                coverage.supports_snapshot_count,
                coverage.supports_prepare_count,
                coverage.supports_activate_count,
                coverage.accepts_midi_count,
                coverage.supports_note_expression_count,
                coverage.produces_midi_count,
                coverage.max_audio_bus_count,
                coverage.max_parameter_count,
            )
        })
        .collect::<String>();
    let parity_coverage_lines = snapshot
        .parity_coverage
        .iter()
        .enumerate()
        .map(|(index, parity)| {
            format!(
                "\nplugin_parity_coverage_{}={:?}/{:?}/linux={:?}/linux_supported={}/linux_policy={:?}/linux_strict_default={}/supported={:?}/unsupported={:?}/types={}/prepare_capable={}/activate_capable={}/sandboxes={}/in_process={}/shared={}/isolated={}/ready={}/restarting={}/rebindable={}/degraded={}/faulted={}/quarantined={}/terminal={}/transport_active={}/placement_rules={}",
                index,
                parity.format,
                parity.parity_band,
                parity.linux_parity_band,
                parity.linux_supported,
                parity.linux_preferred_sandbox_outcome,
                parity.linux_strict_sandbox_default,
                parity.supported_platforms,
                parity.unsupported_platforms,
                parity.discovered_type_count,
                parity.prepare_capable_type_count,
                parity.activate_capable_type_count,
                parity.sandbox_count,
                parity.in_process_sandbox_count,
                parity.shared_sandbox_count,
                parity.isolated_sandbox_count,
                parity.ready_sandbox_count,
                parity.restarting_sandbox_count,
                parity.rebindable_sandbox_count,
                parity.degraded_sandbox_count,
                parity.faulted_sandbox_count,
                parity.quarantined_sandbox_count,
                parity.terminal_sandbox_count,
                parity.active_transport_count,
                parity.explicit_placement_rule_count,
            )
        })
        .collect::<String>();
    let discovered_type_lines = snapshot
        .discovered_types
        .iter()
        .enumerate()
        .map(|(index, record)| {
            format!(
                "\nplugin_discovered_type_{}={}/plugin_id={}/vendor={}/name={}/format={:?}/version={:?}/features={:?}/io={:?}/audio_buses={}/parameters={}/lv2_extension={}",
                index,
                record.plugin_type_id,
                record.plugin_id,
                record.vendor,
                record.name,
                record.format,
                record.version,
                record.features,
                record.default_io_layout,
                record.audio_bus_count,
                record.parameter_count,
                record
                    .lv2_extension_capabilities
                    .as_ref()
                    .map(|summary| summary.summary.as_str())
                    .unwrap_or("none"),
            )
        })
        .collect::<String>();
    format!(
        "\nplugin_scan_count={}\nplugin_format_filtered_scan_count={}\nplugin_discovered_type_count={}\nplugin_discovered_format_count={}\nplugin_capability_coverage_summary={}\nplugin_capability_coverage_multi_format_catalog={}\nplugin_capability_coverage_max_audio_bus_count={}\nplugin_capability_coverage_max_parameter_count={}{}{}{}{}",
        snapshot.scan_count,
        snapshot.format_filtered_scan_count,
        snapshot.discovered_type_count,
        snapshot.discovered_format_count,
        snapshot.capability_coverage.summary,
        snapshot.capability_coverage.multi_format_catalog,
        snapshot.capability_coverage.max_audio_bus_count,
        snapshot.capability_coverage.max_parameter_count,
        last_scan,
        format_coverage_lines,
        parity_coverage_lines,
        discovered_type_lines,
    )
}

pub(crate) fn format_runtime_lv2_extension_snapshot_compact(
    snapshot: &RuntimeLv2ExtensionSnapshot,
) -> String {
    format!(
        " lv2_extensions=types={}/sandboxes={} worker_required={} worker_guarded={} urid_negotiated={} patch_supported={} negotiated={} guarded={} unavailable={}",
        snapshot.plugin_type_count,
        snapshot.sandbox_count,
        snapshot.worker_required_type_count,
        snapshot.worker_guarded_type_count,
        snapshot.urid_negotiated_type_count,
        snapshot.patch_supported_type_count,
        snapshot.negotiated_type_count,
        snapshot.guarded_type_count,
        snapshot.unavailable_type_count,
    )
}

pub(crate) fn format_runtime_lv2_extension_snapshot_multiline(
    snapshot: &RuntimeLv2ExtensionSnapshot,
) -> String {
    let record_lines = snapshot
        .records
        .iter()
        .enumerate()
        .map(|(index, record)| {
            format!(
                "\nlv2_extension_record_{}={}/plugin_id={}/worker={:?}/urid={:?}/patch={:?}/negotiation={:?}/sandboxes={}/active={}/faulted={}/lifecycle={:?}",
                index,
                record.plugin_type_id,
                record.plugin_id,
                record.worker_posture,
                record.urid_negotiation_posture,
                record.patch_exchange_posture,
                record.extension_negotiation_state,
                record.sandbox_count,
                record.active_sandbox_count,
                record.faulted_sandbox_count,
                record.strongest_lifecycle_state,
            )
        })
        .collect::<String>();
    format!(
        "\nlv2_extension_plugin_type_count={}\nlv2_extension_sandbox_count={}\nlv2_extension_worker_required_type_count={}\nlv2_extension_worker_guarded_type_count={}\nlv2_extension_urid_negotiated_type_count={}\nlv2_extension_patch_supported_type_count={}\nlv2_extension_negotiated_type_count={}\nlv2_extension_guarded_type_count={}\nlv2_extension_unavailable_type_count={}\nlv2_extension_summary={}{}",
        snapshot.plugin_type_count,
        snapshot.sandbox_count,
        snapshot.worker_required_type_count,
        snapshot.worker_guarded_type_count,
        snapshot.urid_negotiated_type_count,
        snapshot.patch_supported_type_count,
        snapshot.negotiated_type_count,
        snapshot.guarded_type_count,
        snapshot.unavailable_type_count,
        snapshot.summary,
        record_lines,
    )
}
