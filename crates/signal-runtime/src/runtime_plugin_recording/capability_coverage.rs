use super::*;

pub(crate) fn runtime_plugin_capability_coverage(
    discovered_types: &[RuntimePluginDiscoveredTypeRecord],
) -> RuntimePluginCapabilityCoverageSummary {
    let mut discovered_formats = discovered_types
        .iter()
        .map(|record| record.format)
        .collect::<Vec<_>>();
    discovered_formats.sort_by_key(|format| plugin_format_sort_key(*format));
    discovered_formats.dedup();
    let discovered_format_count = discovered_formats.len();
    let feature_count = |feature: PluginFeature| -> usize {
        discovered_types
            .iter()
            .filter(|record| record.features.contains(&feature))
            .count()
    };
    RuntimePluginCapabilityCoverageSummary {
        discovered_format_count,
        multi_format_catalog: discovered_format_count > 1,
        complex_io_type_count: discovered_types.iter().filter(|record| record.complex_io_summary.has_complex_topology).count(),
        multi_output_instrument_count: discovered_types.iter().filter(|record| record.complex_io_summary.multi_output_instrument).count(),
        bus_capable_fx_count: discovered_types.iter().filter(|record| record.complex_io_summary.bus_capable_fx_class.is_some()).count(),
        sidechain_capable_fx_count: discovered_types
            .iter()
            .filter(|record| {
                record.complex_io_summary.bus_capable_fx_class
                    == Some(RuntimePluginBusCapableFxClass::SidechainCapableFx)
                    || record.complex_io_summary.bus_capable_fx_class
                        == Some(RuntimePluginBusCapableFxClass::SendReturnCapableFx)
            })
            .count(),
        instrument_count: feature_count(PluginFeature::Instrument),
        audio_effect_count: feature_count(PluginFeature::AudioEffect),
        analyzer_count: feature_count(PluginFeature::Analyzer),
        utility_count: feature_count(PluginFeature::Utility),
        note_effect_count: feature_count(PluginFeature::NoteEffect),
        supports_snapshot_count: discovered_types.iter().filter(|record| record.state_contract.supports_snapshot).count(),
        supports_reset_count: discovered_types.iter().filter(|record| record.state_contract.supports_reset).count(),
        supports_bypass_count: discovered_types.iter().filter(|record| record.state_contract.supports_bypass).count(),
        exposes_latency_count: discovered_types.iter().filter(|record| record.state_contract.exposes_latency).count(),
        exposes_tail_count: discovered_types.iter().filter(|record| record.state_contract.exposes_tail).count(),
        sample_accurate_automation_count: discovered_types.iter().filter(|record| record.processing_contract.sample_accurate_automation).count(),
        accepts_midi_count: discovered_types.iter().filter(|record| record.processing_contract.accepts_midi).count(),
        accepts_note_events_count: discovered_types.iter().filter(|record| record.processing_contract.accepts_note_events).count(),
        supports_note_expression_count: discovered_types.iter().filter(|record| record.processing_contract.supports_note_expression).count(),
        produces_midi_count: discovered_types.iter().filter(|record| record.processing_contract.produces_midi).count(),
        silence_aware_count: discovered_types.iter().filter(|record| record.processing_contract.silence_aware).count(),
        requires_main_thread_for_state_count: discovered_types.iter().filter(|record| record.lifecycle_contract.requires_main_thread_for_state).count(),
        supports_prepare_count: discovered_types.iter().filter(|record| record.lifecycle_contract.supports_prepare).count(),
        supports_activate_count: discovered_types.iter().filter(|record| record.lifecycle_contract.supports_activate).count(),
        supports_reset_while_active_count: discovered_types.iter().filter(|record| record.lifecycle_contract.supports_reset_while_active).count(),
        max_complex_io_port_group_count: discovered_types.iter().map(|record| record.complex_io_summary.port_group_count).max().unwrap_or(0),
        max_audio_bus_count: discovered_types.iter().map(|record| record.audio_bus_count).max().unwrap_or(0),
        max_parameter_count: discovered_types.iter().map(|record| record.parameter_count).max().unwrap_or(0),
    }
}
