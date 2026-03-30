use super::super::super::*;

pub(super) fn json_runtime_plugin_capability_coverage_summary(
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
