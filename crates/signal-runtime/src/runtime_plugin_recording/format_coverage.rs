use super::*;

pub(crate) fn runtime_plugin_format_coverage(
    discovered_types: &[RuntimePluginDiscoveredTypeRecord],
) -> Vec<RuntimePluginFormatCoverageRecord> {
    let mut grouped = BTreeMap::new();
    for record in discovered_types {
        grouped
            .entry(plugin_format_sort_key(record.format))
            .or_insert_with(Vec::new)
            .push(record);
    }
    grouped
        .into_values()
        .map(|records| {
            let format = records[0].format;
            let feature_count = |feature: PluginFeature| -> usize {
                records
                    .iter()
                    .filter(|record| record.features.contains(&feature))
                    .count()
            };
            let supports_snapshot_count = records
                .iter()
                .filter(|record| record.state_contract.supports_snapshot)
                .count();
            let supports_prepare_count = records
                .iter()
                .filter(|record| record.lifecycle_contract.supports_prepare)
                .count();
            let supports_activate_count = records
                .iter()
                .filter(|record| record.lifecycle_contract.supports_activate)
                .count();
            let accepts_midi_count = records
                .iter()
                .filter(|record| record.processing_contract.accepts_midi)
                .count();
            let supports_note_expression_count = records
                .iter()
                .filter(|record| record.processing_contract.supports_note_expression)
                .count();
            let produces_midi_count = records
                .iter()
                .filter(|record| record.processing_contract.produces_midi)
                .count();
            let max_audio_bus_count = records.iter().map(|record| record.audio_bus_count).max().unwrap_or(0);
            let max_complex_io_port_group_count = records
                .iter()
                .map(|record| record.complex_io_summary.port_group_count)
                .max()
                .unwrap_or(0);
            let max_parameter_count = records.iter().map(|record| record.parameter_count).max().unwrap_or(0);
            RuntimePluginFormatCoverageRecord {
                format,
                discovered_type_count: records.len(),
                complex_io_type_count: records.iter().filter(|record| record.complex_io_summary.has_complex_topology).count(),
                multi_output_instrument_count: records.iter().filter(|record| record.complex_io_summary.multi_output_instrument).count(),
                bus_capable_fx_count: records.iter().filter(|record| record.complex_io_summary.bus_capable_fx_class.is_some()).count(),
                sidechain_capable_fx_count: records
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
                supports_snapshot_count,
                supports_prepare_count,
                supports_activate_count,
                accepts_midi_count,
                supports_note_expression_count,
                produces_midi_count,
                max_complex_io_port_group_count,
                max_audio_bus_count,
                max_parameter_count,
            }
        })
        .collect()
}
