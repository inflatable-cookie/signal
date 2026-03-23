use super::*;

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
        json_runtime_plugin_format_coverage_vec(&receipt.format_coverage),
        json_runtime_plugin_parity_coverage_vec(&receipt.parity_coverage),
        json_runtime_plugin_capability_coverage_summary(&receipt.capability_coverage),
        json_option_string(Some(receipt.summary.as_str())),
    )
}

fn json_runtime_plugin_format_coverage_vec(
    records: &[RuntimePluginFormatCoverageRecord],
) -> String {
    format!(
        "[{}]",
        records
            .iter()
            .map(json_runtime_plugin_format_coverage_record)
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn json_runtime_plugin_format_coverage_record(
    record: &RuntimePluginFormatCoverageRecord,
) -> String {
    format!(
        concat!(
            "{{",
            "\"format\":{},",
            "\"discovered_type_count\":{},",
            "\"complex_io_type_count\":{},",
            "\"multi_output_instrument_count\":{},",
            "\"bus_capable_fx_count\":{},",
            "\"sidechain_capable_fx_count\":{},",
            "\"instrument_count\":{},",
            "\"audio_effect_count\":{},",
            "\"analyzer_count\":{},",
            "\"utility_count\":{},",
            "\"note_effect_count\":{},",
            "\"supports_snapshot_count\":{},",
            "\"supports_prepare_count\":{},",
            "\"supports_activate_count\":{},",
            "\"accepts_midi_count\":{},",
            "\"supports_note_expression_count\":{},",
            "\"produces_midi_count\":{},",
            "\"max_complex_io_port_group_count\":{},",
            "\"max_audio_bus_count\":{},",
            "\"max_parameter_count\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_escape_string(&format!("{:?}", record.format)),
        record.discovered_type_count,
        record.complex_io_type_count,
        record.multi_output_instrument_count,
        record.bus_capable_fx_count,
        record.sidechain_capable_fx_count,
        record.instrument_count,
        record.audio_effect_count,
        record.analyzer_count,
        record.utility_count,
        record.note_effect_count,
        record.supports_snapshot_count,
        record.supports_prepare_count,
        record.supports_activate_count,
        record.accepts_midi_count,
        record.supports_note_expression_count,
        record.produces_midi_count,
        record.max_complex_io_port_group_count,
        record.max_audio_bus_count,
        record.max_parameter_count,
        json_option_string(Some(record.summary.as_str())),
    )
}

pub(super) fn json_runtime_plugin_parity_coverage_vec(
    records: &[RuntimePluginFormatParityRecord],
) -> String {
    format!(
        "[{}]",
        records
            .iter()
            .map(json_runtime_plugin_parity_coverage_record)
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn json_runtime_plugin_host_platform_vec(platforms: &[RuntimePluginHostPlatform]) -> String {
    format!(
        "[{}]",
        platforms
            .iter()
            .map(|platform| json_escape_string(&format!("{platform:?}")))
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn json_runtime_plugin_parity_coverage_record(record: &RuntimePluginFormatParityRecord) -> String {
    format!(
        concat!(
            "{{",
            "\"format\":{},",
            "\"parity_band\":{},",
            "\"linux_parity_band\":{},",
            "\"supported_platforms\":{},",
            "\"unsupported_platforms\":{},",
            "\"linux_supported\":{},",
            "\"linux_preferred_sandbox_outcome\":{},",
            "\"linux_strict_sandbox_default\":{},",
            "\"discovered_type_count\":{},",
            "\"prepare_capable_type_count\":{},",
            "\"activate_capable_type_count\":{},",
            "\"sandbox_count\":{},",
            "\"in_process_sandbox_count\":{},",
            "\"shared_sandbox_count\":{},",
            "\"isolated_sandbox_count\":{},",
            "\"ready_sandbox_count\":{},",
            "\"restarting_sandbox_count\":{},",
            "\"rebindable_sandbox_count\":{},",
            "\"degraded_sandbox_count\":{},",
            "\"faulted_sandbox_count\":{},",
            "\"quarantined_sandbox_count\":{},",
            "\"terminal_sandbox_count\":{},",
            "\"active_transport_count\":{},",
            "\"explicit_placement_rule_count\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_escape_string(&format!("{:?}", record.format)),
        json_escape_string(&format!("{:?}", record.parity_band)),
        json_escape_string(&format!("{:?}", record.linux_parity_band)),
        json_runtime_plugin_host_platform_vec(&record.supported_platforms),
        json_runtime_plugin_host_platform_vec(&record.unsupported_platforms),
        record.linux_supported,
        json_option_string(
            record
                .linux_preferred_sandbox_outcome
                .as_ref()
                .map(|value| format!("{value:?}"))
                .as_deref(),
        ),
        record.linux_strict_sandbox_default,
        record.discovered_type_count,
        record.prepare_capable_type_count,
        record.activate_capable_type_count,
        record.sandbox_count,
        record.in_process_sandbox_count,
        record.shared_sandbox_count,
        record.isolated_sandbox_count,
        record.ready_sandbox_count,
        record.restarting_sandbox_count,
        record.rebindable_sandbox_count,
        record.degraded_sandbox_count,
        record.faulted_sandbox_count,
        record.quarantined_sandbox_count,
        record.terminal_sandbox_count,
        record.active_transport_count,
        record.explicit_placement_rule_count,
        json_option_string(Some(record.summary.as_str())),
    )
}

fn json_runtime_plugin_capability_coverage_summary(
    summary: &RuntimePluginCapabilityCoverageSummary,
) -> String {
    format!(
        concat!(
            "{{",
            "\"discovered_format_count\":{},",
            "\"multi_format_catalog\":{},",
            "\"complex_io_type_count\":{},",
            "\"multi_output_instrument_count\":{},",
            "\"bus_capable_fx_count\":{},",
            "\"sidechain_capable_fx_count\":{},",
            "\"instrument_count\":{},",
            "\"audio_effect_count\":{},",
            "\"analyzer_count\":{},",
            "\"utility_count\":{},",
            "\"note_effect_count\":{},",
            "\"supports_snapshot_count\":{},",
            "\"supports_reset_count\":{},",
            "\"supports_bypass_count\":{},",
            "\"exposes_latency_count\":{},",
            "\"exposes_tail_count\":{},",
            "\"sample_accurate_automation_count\":{},",
            "\"accepts_midi_count\":{},",
            "\"accepts_note_events_count\":{},",
            "\"supports_note_expression_count\":{},",
            "\"produces_midi_count\":{},",
            "\"silence_aware_count\":{},",
            "\"requires_main_thread_for_state_count\":{},",
            "\"supports_prepare_count\":{},",
            "\"supports_activate_count\":{},",
            "\"supports_reset_while_active_count\":{},",
            "\"max_complex_io_port_group_count\":{},",
            "\"max_audio_bus_count\":{},",
            "\"max_parameter_count\":{},",
            "\"summary\":{}",
            "}}"
        ),
        summary.discovered_format_count,
        summary.multi_format_catalog,
        summary.complex_io_type_count,
        summary.multi_output_instrument_count,
        summary.bus_capable_fx_count,
        summary.sidechain_capable_fx_count,
        summary.instrument_count,
        summary.audio_effect_count,
        summary.analyzer_count,
        summary.utility_count,
        summary.note_effect_count,
        summary.supports_snapshot_count,
        summary.supports_reset_count,
        summary.supports_bypass_count,
        summary.exposes_latency_count,
        summary.exposes_tail_count,
        summary.sample_accurate_automation_count,
        summary.accepts_midi_count,
        summary.accepts_note_events_count,
        summary.supports_note_expression_count,
        summary.produces_midi_count,
        summary.silence_aware_count,
        summary.requires_main_thread_for_state_count,
        summary.supports_prepare_count,
        summary.supports_activate_count,
        summary.supports_reset_while_active_count,
        summary.max_complex_io_port_group_count,
        summary.max_audio_bus_count,
        summary.max_parameter_count,
        json_option_string(Some(summary.summary.as_str())),
    )
}

fn json_runtime_plugin_port_class_vec(values: &[RuntimePluginPortClass]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| json_escape_string(&format!("{value:?}")))
            .collect::<Vec<_>>()
            .join(",")
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

pub(super) fn json_runtime_plugin_complex_io_summary(
    summary: &RuntimePluginComplexIoSummary,
) -> String {
    let bus_capable_fx_class = summary
        .bus_capable_fx_class
        .map(|value| format!("{value:?}"));
    format!(
        concat!(
            "{{",
            "\"has_complex_topology\":{},",
            "\"declared_port_classes\":{},",
            "\"port_group_count\":{},",
            "\"main_input_group_count\":{},",
            "\"main_output_group_count\":{},",
            "\"secondary_input_group_count\":{},",
            "\"aux_input_group_count\":{},",
            "\"aux_output_group_count\":{},",
            "\"instrument_output_group_count\":{},",
            "\"analysis_output_group_count\":{},",
            "\"multi_output_instrument\":{},",
            "\"bus_capable_fx_class\":{},",
            "\"attachment_policy\":{},",
            "\"fallback_outcome\":{},",
            "\"summary\":{}",
            "}}"
        ),
        summary.has_complex_topology,
        json_runtime_plugin_port_class_vec(&summary.declared_port_classes),
        summary.port_group_count,
        summary.main_input_group_count,
        summary.main_output_group_count,
        summary.secondary_input_group_count,
        summary.aux_input_group_count,
        summary.aux_output_group_count,
        summary.instrument_output_group_count,
        summary.analysis_output_group_count,
        summary.multi_output_instrument,
        json_option_string(bus_capable_fx_class.as_deref()),
        json_option_string(Some(&format!("{:?}", summary.attachment_policy))),
        json_option_string(Some(&format!("{:?}", summary.fallback_outcome))),
        json_option_string(Some(summary.summary.as_str())),
    )
}
