use super::super::super::*;

pub(super) fn json_runtime_plugin_format_coverage_vec(
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
