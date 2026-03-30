use super::*;

pub(crate) fn sample_complex_multi_output_record() -> RuntimePluginDiscoveredTypeRecord {
    RuntimePluginDiscoveredTypeRecord {
        plugin_type_id: "plugin:vst3:host-server-multiout".into(),
        plugin_id: "com.signal.host-server-multiout".into(),
        vendor: "Signal".into(),
        name: "Signal Host Server Multi Output".into(),
        format: PluginFormat::Vst3,
        version: Some("1.0.0".into()),
        features: vec![PluginFeature::Instrument, PluginFeature::Analyzer],
        default_io_layout: PluginIoLayout {
            audio_inputs: 0,
            audio_outputs: 6,
            midi_inputs: 1,
            midi_outputs: 0,
        },
        default_multichannel_io: signal_runtime::RuntimeMultichannelIoSummary::for_plugin_io(
            PluginIoLayout {
                audio_inputs: 0,
                audio_outputs: 6,
                midi_inputs: 1,
                midi_outputs: 0,
            },
        ),
        complex_io_summary: RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
            &[PluginFeature::Instrument, PluginFeature::Analyzer],
            PluginIoLayout {
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
        summary: "server complex multi-output instrument".into(),
    }
}

pub(crate) fn sample_complex_bus_fx_record() -> RuntimePluginDiscoveredTypeRecord {
    RuntimePluginDiscoveredTypeRecord {
        plugin_type_id: "plugin:vst3:host-server-bus-fx".into(),
        plugin_id: "com.signal.host-server-bus-fx".into(),
        vendor: "Signal".into(),
        name: "Signal Host Server Bus FX".into(),
        format: PluginFormat::Vst3,
        version: Some("1.0.0".into()),
        features: vec![PluginFeature::AudioEffect, PluginFeature::Utility],
        default_io_layout: PluginIoLayout {
            audio_inputs: 4,
            audio_outputs: 4,
            midi_inputs: 0,
            midi_outputs: 0,
        },
        default_multichannel_io: signal_runtime::RuntimeMultichannelIoSummary::for_plugin_io(
            PluginIoLayout {
                audio_inputs: 4,
                audio_outputs: 4,
                midi_inputs: 0,
                midi_outputs: 0,
            },
        ),
        complex_io_summary: RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
            &[PluginFeature::AudioEffect, PluginFeature::Utility],
            PluginIoLayout {
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
        summary: "server bus-capable fx".into(),
    }
}

pub(crate) fn public_server_media_fixture_path(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic enough for test files")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "signal-host-server-public-media-{label}-{}-{unique}.wav",
        std::process::id()
    ))
}

pub(crate) fn write_public_test_wav(path: &Path) {
    let channels = 1u16;
    let sample_rate = 48_000u32;
    let bits_per_sample = 16u16;
    let frame_count = 128u32;
    let block_align = channels * (bits_per_sample / 8);
    let byte_rate = sample_rate * block_align as u32;
    let data_size = frame_count * block_align as u32;
    let riff_size = 36 + data_size;
    let mut bytes = Vec::with_capacity((44 + data_size) as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&riff_size.to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&channels.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&byte_rate.to_le_bytes());
    bytes.extend_from_slice(&block_align.to_le_bytes());
    bytes.extend_from_slice(&bits_per_sample.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_size.to_le_bytes());
    for index in 0..frame_count {
        let sample =
            (((index as f32 / (frame_count - 1) as f32) * 2.0) - 1.0) * i16::MAX as f32 * 0.5;
        bytes.extend_from_slice(&(sample as i16).to_le_bytes());
    }
    fs::write(path, bytes).expect("public server media fixture should be written");
}

pub(crate) fn write_public_transient_test_wav(path: &Path) {
    let channels = 1u16;
    let sample_rate = 48_000u32;
    let bits_per_sample = 16u16;
    let frame_count = 48_000u32;
    let block_align = channels * (bits_per_sample / 8);
    let byte_rate = sample_rate * block_align as u32;
    let data_size = frame_count * block_align as u32;
    let riff_size = 36 + data_size;
    let mut bytes = Vec::with_capacity((44 + data_size) as usize);
    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&riff_size.to_le_bytes());
    bytes.extend_from_slice(b"WAVE");
    bytes.extend_from_slice(b"fmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&channels.to_le_bytes());
    bytes.extend_from_slice(&sample_rate.to_le_bytes());
    bytes.extend_from_slice(&byte_rate.to_le_bytes());
    bytes.extend_from_slice(&block_align.to_le_bytes());
    bytes.extend_from_slice(&bits_per_sample.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data_size.to_le_bytes());
    for index in 0..frame_count {
        let sample = if index % 6_000 == 0 { i16::MAX } else { 0 };
        bytes.extend_from_slice(&sample.to_le_bytes());
    }
    fs::write(path, bytes).expect("public server transient media fixture should be written");
}

pub(crate) fn record_public_plugin_sandbox_ready(
    runtime: &mut SignalRuntime,
    sandbox_id: &str,
    plugin_format: PluginFormat,
    plugin_type_id: &str,
    epoch: u64,
) {
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: sandbox_id.into(),
        plugin_format,
        plugin_type_id: Some(plugin_type_id.into()),
    });
    runtime.record_plugin_sandbox_lifecycle(
        sandbox_id,
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(epoch),
    );
    runtime.record_plugin_sandbox_transport(
        sandbox_id,
        format!("lease-{sandbox_id}"),
        format!("region-{sandbox_id}"),
        PluginSandboxTransportStage::Attached,
        Some(epoch),
        None,
    );
}

pub(crate) fn sample_server_ara_context() -> RuntimePluginAraContextSnapshot {
    RuntimePluginAraContextSnapshot {
        portability_class: RuntimePluginRecallPortabilityClass::ContextOnly,
        document_context: Some(RuntimePluginAraDocumentContext {
            document_id: "doc:host-server".into(),
            display_label: Some("Server Session".into()),
            summary: "server host ara document".into(),
        }),
        source_context: Some(RuntimePluginAraSourceContext {
            source_id: "source:stem-bus".into(),
            display_label: Some("Stem Bus".into()),
            summary: "server host ara source".into(),
        }),
        region_context: Some(RuntimePluginAraRegionContext {
            region_id: "region:bridge".into(),
            display_label: Some("Bridge".into()),
            timeline_start_samples: Some(16_384),
            duration_samples: Some(4_096),
            summary: "server host ara region".into(),
        }),
        summary: "server host ara context".into(),
    }
}
