#![allow(dead_code)]

use signal_plugin::{
    PluginFeature, PluginFormat, PluginIoLayout, PluginLifecycleContract, PluginProcessingContract,
    PluginStateContract,
};
use signal_runtime::{RuntimePluginComplexIoSummary, RuntimePluginDiscoveredTypeRecord};

pub fn sample_discovered_type_record() -> RuntimePluginDiscoveredTypeRecord {
    RuntimePluginDiscoveredTypeRecord {
        plugin_type_id: "plugin:clap:public-boundary".into(),
        plugin_id: "com.signal.public-boundary".into(),
        vendor: "Signal".into(),
        name: "Signal Public Boundary".into(),
        format: PluginFormat::Clap,
        version: Some("1.0.0".into()),
        features: vec![PluginFeature::AudioEffect, PluginFeature::Utility],
        default_io_layout: PluginIoLayout {
            audio_inputs: 2,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 1,
        },
        default_multichannel_io: signal_runtime::RuntimeMultichannelIoSummary::for_plugin_io(
            PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 1,
            },
        ),
        complex_io_summary: RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
            &[PluginFeature::AudioEffect, PluginFeature::Utility],
            PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 1,
            },
        ),
        audio_bus_count: 2,
        parameter_count: 8,
        state_contract: PluginStateContract {
            supports_snapshot: true,
            supports_reset: true,
            supports_bypass: true,
            exposes_latency: true,
            exposes_tail: true,
        },
        processing_contract: PluginProcessingContract {
            max_block_frames: 4096,
            sample_accurate_automation: true,
            accepts_midi: true,
            accepts_note_events: true,
            supports_note_expression: true,
            produces_midi: true,
            silence_aware: true,
        },
        lifecycle_contract: PluginLifecycleContract {
            requires_main_thread_for_state: false,
            supports_prepare: true,
            supports_activate: true,
            supports_reset_while_active: true,
        },
        lv2_extension_capabilities: None,
    }
}

pub fn sample_backend_breadth_record() -> RuntimePluginDiscoveredTypeRecord {
    RuntimePluginDiscoveredTypeRecord {
        plugin_type_id: "plugin:vst3:public-instrument".into(),
        plugin_id: "com.signal.public-instrument".into(),
        vendor: "Signal".into(),
        name: "Signal Public Instrument".into(),
        format: PluginFormat::Vst3,
        version: Some("2.0.0".into()),
        features: vec![PluginFeature::Instrument, PluginFeature::Analyzer],
        default_io_layout: PluginIoLayout {
            audio_inputs: 0,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 0,
        },
        default_multichannel_io: signal_runtime::RuntimeMultichannelIoSummary::for_plugin_io(
            PluginIoLayout {
                audio_inputs: 0,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 0,
            },
        ),
        complex_io_summary: RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
            &[PluginFeature::Instrument, PluginFeature::Analyzer],
            PluginIoLayout {
                audio_inputs: 0,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 0,
            },
        ),
        audio_bus_count: 1,
        parameter_count: 12,
        state_contract: PluginStateContract {
            supports_snapshot: false,
            supports_reset: true,
            supports_bypass: false,
            exposes_latency: false,
            exposes_tail: true,
        },
        processing_contract: PluginProcessingContract {
            max_block_frames: 2048,
            sample_accurate_automation: false,
            accepts_midi: true,
            accepts_note_events: true,
            supports_note_expression: true,
            produces_midi: false,
            silence_aware: false,
        },
        lifecycle_contract: PluginLifecycleContract {
            requires_main_thread_for_state: true,
            supports_prepare: true,
            supports_activate: false,
            supports_reset_while_active: false,
        },
        lv2_extension_capabilities: None,
    }
}

pub fn sample_au_breadth_record() -> RuntimePluginDiscoveredTypeRecord {
    RuntimePluginDiscoveredTypeRecord {
        plugin_type_id: "plugin:au:public-instrument".into(),
        plugin_id: "com.signal.public-au-instrument".into(),
        vendor: "Signal".into(),
        name: "Signal Public AU Instrument".into(),
        format: PluginFormat::Au,
        version: Some("1.0.0".into()),
        features: vec![PluginFeature::Instrument, PluginFeature::Analyzer],
        default_io_layout: PluginIoLayout {
            audio_inputs: 0,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 0,
        },
        default_multichannel_io: signal_runtime::RuntimeMultichannelIoSummary::for_plugin_io(
            PluginIoLayout {
                audio_inputs: 0,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 0,
            },
        ),
        complex_io_summary: RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
            &[PluginFeature::Instrument, PluginFeature::Analyzer],
            PluginIoLayout {
                audio_inputs: 0,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 0,
            },
        ),
        audio_bus_count: 1,
        parameter_count: 10,
        state_contract: PluginStateContract {
            supports_snapshot: true,
            supports_reset: true,
            supports_bypass: true,
            exposes_latency: false,
            exposes_tail: true,
        },
        processing_contract: PluginProcessingContract {
            max_block_frames: 2048,
            sample_accurate_automation: false,
            accepts_midi: true,
            accepts_note_events: true,
            supports_note_expression: true,
            produces_midi: false,
            silence_aware: false,
        },
        lifecycle_contract: PluginLifecycleContract {
            requires_main_thread_for_state: true,
            supports_prepare: true,
            supports_activate: true,
            supports_reset_while_active: false,
        },
        lv2_extension_capabilities: None,
    }
}
