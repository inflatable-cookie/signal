use super::*;

pub(super) fn discovered_types() -> Vec<crate::RuntimePluginDiscoveredTypeRecord> {
    vec![
        crate::RuntimePluginDiscoveredTypeRecord {
            plugin_type_id: "plugin:vst3:multiout-instrument".into(),
            plugin_id: "com.signal.multiout".into(),
            vendor: "Signal".into(),
            name: "Signal Multi Output Instrument".into(),
            format: PluginFormat::Vst3,
            version: Some("1.0.0".into()),
            features: vec![
                signal_plugin::PluginFeature::Instrument,
                signal_plugin::PluginFeature::Analyzer,
            ],
            default_io_layout: signal_plugin::PluginIoLayout {
                audio_inputs: 0,
                audio_outputs: 6,
                midi_inputs: 1,
                midi_outputs: 0,
            },
            default_multichannel_io: crate::RuntimeMultichannelIoSummary::for_plugin_io(
                signal_plugin::PluginIoLayout {
                    audio_inputs: 0,
                    audio_outputs: 6,
                    midi_inputs: 1,
                    midi_outputs: 0,
                },
            ),
            complex_io_summary: crate::RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
                &[
                    signal_plugin::PluginFeature::Instrument,
                    signal_plugin::PluginFeature::Analyzer,
                ],
                signal_plugin::PluginIoLayout {
                    audio_inputs: 0,
                    audio_outputs: 6,
                    midi_inputs: 1,
                    midi_outputs: 0,
                },
            ),
            audio_bus_count: 1,
            parameter_count: 24,
            state_contract: signal_plugin::PluginStateContract {
                supports_snapshot: false,
                supports_reset: true,
                supports_bypass: false,
                exposes_latency: false,
                exposes_tail: true,
            },
            processing_contract: signal_plugin::PluginProcessingContract {
                max_block_frames: 2048,
                sample_accurate_automation: false,
                accepts_midi: true,
                accepts_note_events: true,
                supports_note_expression: true,
                produces_midi: false,
                silence_aware: false,
            },
            lifecycle_contract: signal_plugin::PluginLifecycleContract {
                requires_main_thread_for_state: true,
                supports_prepare: true,
                supports_activate: true,
                supports_reset_while_active: false,
            },
            lv2_extension_capabilities: None,
            summary: "plugin_type=plugin:vst3:multiout-instrument".into(),
        },
        crate::RuntimePluginDiscoveredTypeRecord {
            plugin_type_id: "plugin:vst3:bus-fx".into(),
            plugin_id: "com.signal.bus-fx".into(),
            vendor: "Signal".into(),
            name: "Signal Bus FX".into(),
            format: PluginFormat::Vst3,
            version: Some("1.0.0".into()),
            features: vec![
                signal_plugin::PluginFeature::AudioEffect,
                signal_plugin::PluginFeature::Utility,
            ],
            default_io_layout: signal_plugin::PluginIoLayout {
                audio_inputs: 4,
                audio_outputs: 4,
                midi_inputs: 0,
                midi_outputs: 0,
            },
            default_multichannel_io: crate::RuntimeMultichannelIoSummary::for_plugin_io(
                signal_plugin::PluginIoLayout {
                    audio_inputs: 4,
                    audio_outputs: 4,
                    midi_inputs: 0,
                    midi_outputs: 0,
                },
            ),
            complex_io_summary: crate::RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
                &[
                    signal_plugin::PluginFeature::AudioEffect,
                    signal_plugin::PluginFeature::Utility,
                ],
                signal_plugin::PluginIoLayout {
                    audio_inputs: 4,
                    audio_outputs: 4,
                    midi_inputs: 0,
                    midi_outputs: 0,
                },
            ),
            audio_bus_count: 2,
            parameter_count: 18,
            state_contract: signal_plugin::PluginStateContract {
                supports_snapshot: true,
                supports_reset: true,
                supports_bypass: true,
                exposes_latency: true,
                exposes_tail: true,
            },
            processing_contract: signal_plugin::PluginProcessingContract {
                max_block_frames: 4096,
                sample_accurate_automation: true,
                accepts_midi: false,
                accepts_note_events: false,
                supports_note_expression: false,
                produces_midi: false,
                silence_aware: true,
            },
            lifecycle_contract: signal_plugin::PluginLifecycleContract {
                requires_main_thread_for_state: false,
                supports_prepare: true,
                supports_activate: true,
                supports_reset_while_active: true,
            },
            lv2_extension_capabilities: None,
            summary: "plugin_type=plugin:vst3:bus-fx".into(),
        },
    ]
}
