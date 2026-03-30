use super::*;

pub(crate) fn sample_complex_multi_output_record() -> RuntimePluginDiscoveredTypeRecord {
    RuntimePluginDiscoveredTypeRecord {
        plugin_type_id: "plugin:clap:host-local:multi-output".into(),
        format: PluginFormat::Clap,
        manufacturer: Some("Signal Audio".into()),
        display_name: Some("Host Local Multi Output".into()),
        io_layout: PluginIoLayout::StereoToStereo,
        features: vec![PluginFeature::Instrument, PluginFeature::Spatial],
        complex_io_summary: Some(RuntimePluginComplexIoSummary {
            input_buses: 1,
            output_buses: 3,
            sidechain_buses: 0,
            aux_output_buses: 2,
        }),
        preset_descriptor: Some(sample_host_preset_descriptor()),
        ara_context: Some(sample_host_ara_context()),
        recall_portability: Some(RuntimePluginRecallPortabilityClass::Portable),
        ..RuntimePluginDiscoveredTypeRecord::default()
    }
}

pub(crate) fn sample_complex_bus_fx_record() -> RuntimePluginDiscoveredTypeRecord {
    RuntimePluginDiscoveredTypeRecord {
        plugin_type_id: "plugin:clap:host-local:bus-fx".into(),
        format: PluginFormat::Clap,
        manufacturer: Some("Signal Audio".into()),
        display_name: Some("Host Local Bus FX".into()),
        io_layout: PluginIoLayout::StereoToStereo,
        features: vec![PluginFeature::AudioEffect, PluginFeature::Utility],
        complex_io_summary: Some(RuntimePluginComplexIoSummary {
            input_buses: 2,
            output_buses: 2,
            sidechain_buses: 1,
            aux_output_buses: 0,
        }),
        recall_portability: Some(RuntimePluginRecallPortabilityClass::SessionBound),
        ..RuntimePluginDiscoveredTypeRecord::default()
    }
}

pub(crate) fn public_local_media_fixture_path(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("signal-public-host-local-{label}-{unique}.wav"))
}

pub(crate) fn write_public_test_wav(path: &Path) {
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 48_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec).expect("wav writer");
    for _ in 0..128 {
        writer.write_sample::<i16>(500).expect("left sample");
        writer.write_sample::<i16>(-500).expect("right sample");
    }
    writer.finalize().expect("finalize wav");
}

pub(crate) fn write_public_transient_test_wav(path: &Path) {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 48_000,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec).expect("wav writer");
    for index in 0..4_096 {
        let sample = if index % 512 == 0 { i16::MAX / 3 } else { 0 };
        writer.write_sample::<i16>(sample).expect("mono sample");
    }
    writer.finalize().expect("finalize transient wav");
}

pub(crate) fn record_public_plugin_sandbox_ready(
    runtime: &mut SignalRuntime,
    sandbox_id: &str,
    plugin_format: PluginFormat,
    plugin_type_id: &str,
    processing_epoch: u64,
) {
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: sandbox_id.into(),
        plugin_format,
        plugin_type_id: Some(plugin_type_id.into()),
    });
    runtime.record_plugin_sandbox_lifecycle(
        sandbox_id,
        PluginSandboxLifecycleStage::SandboxEnsured,
        None,
    );
    runtime.record_plugin_sandbox_lifecycle(
        sandbox_id,
        PluginSandboxLifecycleStage::TransportReady,
        Some(plugin_type_id.into()),
    );
    runtime.record_plugin_sandbox_transport(
        sandbox_id,
        PluginSandboxTransportStage::Attached,
        processing_epoch,
    );
}

pub(crate) fn sample_host_preset_descriptor() -> RuntimePluginPresetDescriptor {
    RuntimePluginPresetDescriptor {
        preset_id: "preset:host-local:main".into(),
        display_name: "Host Local Main".into(),
        origin: RuntimePluginPresetOrigin::Factory,
    }
}

pub(crate) fn sample_host_ara_context() -> RuntimePluginAraContextSnapshot {
    RuntimePluginAraContextSnapshot {
        document: Some(RuntimePluginAraDocumentContext {
            persistent_id: "document:host-local".into(),
            summary: "local host ara document".into(),
        }),
        source: Some(RuntimePluginAraSourceContext {
            persistent_id: "source:host-local".into(),
            summary: "local host ara source".into(),
        }),
        region: Some(RuntimePluginAraRegionContext {
            persistent_id: "region:host-local".into(),
            timeline_start_samples: 0,
            duration_samples: Some(8_192),
            summary: "local host ara region".into(),
        }),
        summary: "local host ara context".into(),
    }
}
