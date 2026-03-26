use super::*;

pub(crate) fn json_plugin_format_vec(values: &[PluginFormat]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| json_escape_string(&format!("{value:?}")))
            .collect::<Vec<_>>()
            .join(",")
    )
}

pub(crate) fn json_plugin_feature_vec(values: &[PluginFeature]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| json_escape_string(&format!("{value:?}")))
            .collect::<Vec<_>>()
            .join(",")
    )
}

pub(crate) fn json_plugin_io_layout(layout: PluginIoLayout) -> String {
    format!(
        concat!(
            "{{",
            "\"audio_inputs\":{},",
            "\"audio_outputs\":{},",
            "\"midi_inputs\":{},",
            "\"midi_outputs\":{}",
            "}}"
        ),
        layout.audio_inputs, layout.audio_outputs, layout.midi_inputs, layout.midi_outputs,
    )
}

pub(crate) fn json_plugin_state_contract(contract: PluginStateContract) -> String {
    format!(
        concat!(
            "{{",
            "\"supports_snapshot\":{},",
            "\"supports_reset\":{},",
            "\"supports_bypass\":{},",
            "\"exposes_latency\":{},",
            "\"exposes_tail\":{}",
            "}}"
        ),
        contract.supports_snapshot,
        contract.supports_reset,
        contract.supports_bypass,
        contract.exposes_latency,
        contract.exposes_tail,
    )
}

pub(crate) fn json_plugin_processing_contract(contract: PluginProcessingContract) -> String {
    format!(
        concat!(
            "{{",
            "\"max_block_frames\":{},",
            "\"sample_accurate_automation\":{},",
            "\"accepts_midi\":{},",
            "\"accepts_note_events\":{},",
            "\"produces_midi\":{},",
            "\"silence_aware\":{}",
            "}}"
        ),
        contract.max_block_frames,
        contract.sample_accurate_automation,
        contract.accepts_midi,
        contract.accepts_note_events,
        contract.produces_midi,
        contract.silence_aware,
    )
}

pub(crate) fn json_plugin_lifecycle_contract(contract: PluginLifecycleContract) -> String {
    format!(
        concat!(
            "{{",
            "\"requires_main_thread_for_state\":{},",
            "\"supports_prepare\":{},",
            "\"supports_activate\":{},",
            "\"supports_reset_while_active\":{}",
            "}}"
        ),
        contract.requires_main_thread_for_state,
        contract.supports_prepare,
        contract.supports_activate,
        contract.supports_reset_while_active,
    )
}
