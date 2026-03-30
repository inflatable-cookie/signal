use super::super::*;

#[test]
fn runtime_plugin_discovery_snapshot_and_reports_surface_typed_scan_filters() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure(&mut runtime);
    runtime.record_plugin_format_platform_coverage(vec![
        RuntimePluginFormatPlatformCoverageRecord {
            format: PluginFormat::Clap,
            supported_platforms: vec![
                RuntimePluginHostPlatform::MacOs,
                RuntimePluginHostPlatform::Linux,
                RuntimePluginHostPlatform::Windows,
            ],
            unsupported_platforms: Vec::new(),
            linux_parity_band: RuntimePluginParityBand::Portable,
            linux_preferred_sandbox_outcome: Some(RuntimePluginIsolationOutcome::IsolatedSandbox),
            linux_strict_sandbox_default: true,
            summary:
                "platforms=MacOs/Linux/Windows linux=Portable linux_policy=IsolatedSandbox unsupported=none"
                    .into(),
        },
        RuntimePluginFormatPlatformCoverageRecord {
            format: PluginFormat::Vst3,
            supported_platforms: vec![
                RuntimePluginHostPlatform::MacOs,
                RuntimePluginHostPlatform::Linux,
                RuntimePluginHostPlatform::Windows,
            ],
            unsupported_platforms: Vec::new(),
            linux_parity_band: RuntimePluginParityBand::Portable,
            linux_preferred_sandbox_outcome: Some(RuntimePluginIsolationOutcome::IsolatedSandbox),
            linux_strict_sandbox_default: true,
            summary:
                "platforms=MacOs/Linux/Windows linux=Portable linux_policy=IsolatedSandbox unsupported=none"
                    .into(),
        },
        RuntimePluginFormatPlatformCoverageRecord {
            format: PluginFormat::Au,
            supported_platforms: vec![RuntimePluginHostPlatform::MacOs],
            unsupported_platforms: vec![
                RuntimePluginHostPlatform::Linux,
                RuntimePluginHostPlatform::Windows,
            ],
            linux_parity_band: RuntimePluginParityBand::Unsupported,
            linux_preferred_sandbox_outcome: None,
            linux_strict_sandbox_default: false,
            summary: "platforms=MacOs linux=Unsupported unsupported=Linux/Windows".into(),
        },
    ]);

    let first_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/Library/Audio/Plug-Ins/CLAP".into()],
        formats: vec![PluginFormat::Clap],
    });
    let second_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec![
            "~/Library/Audio/Plug-Ins".into(),
            "/Library/Audio/Plug-Ins".into(),
        ],
        formats: vec![PluginFormat::Clap, PluginFormat::Vst3],
    });
    runtime.record_plugin_scan_results(
        second_handle,
        vec![
            crate::RuntimePluginDiscoveredTypeRecord {
                plugin_type_id: "plugin:clap:default".into(),
                plugin_id: "com.signal.default".into(),
                vendor: "Signal".into(),
                name: "Signal Default".into(),
                format: PluginFormat::Clap,
                version: Some("1.0.0".into()),
                features: vec![
                    signal_plugin::PluginFeature::AudioEffect,
                    signal_plugin::PluginFeature::Utility,
                ],
                default_io_layout: signal_plugin::PluginIoLayout {
                    audio_inputs: 2,
                    audio_outputs: 2,
                    midi_inputs: 1,
                    midi_outputs: 1,
                },
                default_multichannel_io: crate::RuntimeMultichannelIoSummary::for_plugin_io(
                    signal_plugin::PluginIoLayout {
                        audio_inputs: 2,
                        audio_outputs: 2,
                        midi_inputs: 1,
                        midi_outputs: 1,
                    },
                ),
                complex_io_summary:
                    crate::RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
                        &[
                            signal_plugin::PluginFeature::AudioEffect,
                            signal_plugin::PluginFeature::Utility,
                        ],
                        signal_plugin::PluginIoLayout {
                            audio_inputs: 2,
                            audio_outputs: 2,
                            midi_inputs: 1,
                            midi_outputs: 1,
                        },
                    ),
                audio_bus_count: 2,
                parameter_count: 16,
                state_contract: signal_plugin::PluginStateContract {
                    supports_snapshot: true,
                    supports_reset: true,
                    supports_bypass: true,
                    exposes_latency: true,
                    exposes_tail: true,
                },
                processing_contract: signal_plugin::PluginProcessingContract {
                    max_block_frames: 4_096,
                    sample_accurate_automation: true,
                    accepts_midi: true,
                    accepts_note_events: true,
                    supports_note_expression: true,
                    produces_midi: true,
                    silence_aware: true,
                },
                lifecycle_contract: signal_plugin::PluginLifecycleContract {
                    requires_main_thread_for_state: false,
                    supports_prepare: true,
                    supports_activate: true,
                    supports_reset_while_active: true,
                },
                lv2_extension_capabilities: None,
                summary: "plugin_type=plugin:clap:default plugin_id=com.signal.default format=Clap features=2 io=PluginIoLayout { audio_inputs: 2, audio_outputs: 2, midi_inputs: 1, midi_outputs: 1 } parameters=16".into(),
            },
            crate::RuntimePluginDiscoveredTypeRecord {
                plugin_type_id: "plugin:vst3:instrument".into(),
                plugin_id: "com.signal.instrument".into(),
                vendor: "Signal".into(),
                name: "Signal Instrument".into(),
                format: PluginFormat::Vst3,
                version: Some("2.0.0".into()),
                features: vec![
                    signal_plugin::PluginFeature::Instrument,
                    signal_plugin::PluginFeature::Analyzer,
                ],
                default_io_layout: signal_plugin::PluginIoLayout {
                    audio_inputs: 0,
                    audio_outputs: 2,
                    midi_inputs: 1,
                    midi_outputs: 0,
                },
                default_multichannel_io: crate::RuntimeMultichannelIoSummary::for_plugin_io(
                    signal_plugin::PluginIoLayout {
                        audio_inputs: 0,
                        audio_outputs: 2,
                        midi_inputs: 1,
                        midi_outputs: 0,
                    },
                ),
                complex_io_summary:
                    crate::RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
                        &[
                            signal_plugin::PluginFeature::Instrument,
                            signal_plugin::PluginFeature::Analyzer,
                        ],
                        signal_plugin::PluginIoLayout {
                            audio_inputs: 0,
                            audio_outputs: 2,
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
                    max_block_frames: 2_048,
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
                    supports_activate: false,
                    supports_reset_while_active: false,
                },
                lv2_extension_capabilities: None,
                summary: "plugin_type=plugin:vst3:instrument plugin_id=com.signal.instrument format=Vst3 features=2 io=PluginIoLayout { audio_inputs: 0, audio_outputs: 2, midi_inputs: 1, midi_outputs: 0 } parameters=24".into(),
            },
        ],
    );
    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "sandbox-a".into(),
        plugin_format: PluginFormat::Clap,
        plugin_type_id: None,
    });
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox-a",
        PluginSandboxLifecycleStage::SandboxEnsured,
        None,
    );

    let discovery = runtime.get_plugin_discovery_snapshot();
    assert_eq!(first_handle.0, 1);
    assert_eq!(second_handle.0, 2);
    assert_eq!(discovery.scan_count, 2);
    assert_eq!(discovery.format_filtered_scan_count, 2);
    let last_scan = discovery.last_scan.expect("last scan receipt should exist");
    assert_eq!(last_scan.scan_handle, second_handle);
    assert_eq!(
        last_scan.formats,
        vec![PluginFormat::Clap, PluginFormat::Vst3]
    );
    assert_eq!(last_scan.targeted_format_count, 2);
    assert_eq!(last_scan.discovered_type_count, 2);
    assert_eq!(last_scan.discovered_format_count, 2);
    assert_eq!(last_scan.format_coverage.len(), 2);
    assert_eq!(last_scan.parity_coverage.len(), 3);
    assert!(last_scan.capability_coverage.multi_format_catalog);
    assert_eq!(last_scan.capability_coverage.supports_snapshot_count, 1);
    assert_eq!(last_scan.capability_coverage.supports_activate_count, 1);
    assert_eq!(discovery.discovered_type_count, 2);
    assert_eq!(discovery.discovered_format_count, 2);
    assert_eq!(discovery.format_coverage.len(), 2);
    assert_eq!(discovery.parity_coverage.len(), 3);
    assert_eq!(discovery.capability_coverage.instrument_count, 1);
    assert_eq!(discovery.capability_coverage.audio_effect_count, 1);
    assert_eq!(
        discovery
            .capability_coverage
            .requires_main_thread_for_state_count,
        1
    );
    assert_eq!(discovery.capability_coverage.max_parameter_count, 24);
    assert_eq!(discovery.discovered_types.len(), 2);
    let discovered_type = &discovery.discovered_types[0];
    assert_eq!(discovered_type.plugin_type_id, "plugin:clap:default");
    assert_eq!(discovered_type.plugin_id, "com.signal.default");
    assert_eq!(discovered_type.format, PluginFormat::Clap);
    assert_eq!(
        discovered_type.features,
        vec![
            signal_plugin::PluginFeature::AudioEffect,
            signal_plugin::PluginFeature::Utility,
        ]
    );
    assert_eq!(discovered_type.audio_bus_count, 2);
    assert_eq!(discovered_type.parameter_count, 16);
    assert_eq!(
        discovered_type
            .default_multichannel_io
            .input_layout
            .canonical_layout,
        Some(crate::RuntimeCanonicalChannelLayout::Stereo)
    );
    assert_eq!(
        discovered_type
            .default_multichannel_io
            .output_layout
            .canonical_layout,
        Some(crate::RuntimeCanonicalChannelLayout::Stereo)
    );
    assert!(discovered_type.state_contract.supports_snapshot);
    assert!(discovered_type.processing_contract.produces_midi);
    assert!(discovered_type.lifecycle_contract.supports_activate);
    let clap_parity = discovery
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Clap)
        .expect("clap parity should be present");
    assert_eq!(clap_parity.parity_band, RuntimePluginParityBand::Portable);
    assert_eq!(
        clap_parity.linux_parity_band,
        RuntimePluginParityBand::Portable
    );
    assert_eq!(
        clap_parity.supported_platforms,
        vec![
            RuntimePluginHostPlatform::MacOs,
            RuntimePluginHostPlatform::Linux,
            RuntimePluginHostPlatform::Windows,
        ]
    );
    assert!(clap_parity.linux_supported);
    assert_eq!(
        clap_parity.linux_preferred_sandbox_outcome,
        Some(RuntimePluginIsolationOutcome::IsolatedSandbox)
    );
    assert!(clap_parity.linux_strict_sandbox_default);
    assert_eq!(clap_parity.discovered_type_count, 1);
    assert_eq!(clap_parity.prepare_capable_type_count, 1);
    assert_eq!(clap_parity.activate_capable_type_count, 1);
    assert_eq!(clap_parity.sandbox_count, 1);
    assert_eq!(clap_parity.in_process_sandbox_count, 0);
    assert_eq!(clap_parity.explicit_placement_rule_count, 0);
    let au_parity = discovery
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Au)
        .expect("au parity should be present even before discovery");
    assert_eq!(au_parity.parity_band, RuntimePluginParityBand::Guarded);
    assert_eq!(
        au_parity.linux_parity_band,
        RuntimePluginParityBand::Unsupported
    );
    assert!(!au_parity.linux_supported);
    assert_eq!(au_parity.linux_preferred_sandbox_outcome, None);
    assert!(!au_parity.linux_strict_sandbox_default);
    assert_eq!(
        au_parity.unsupported_platforms,
        vec![
            RuntimePluginHostPlatform::Linux,
            RuntimePluginHostPlatform::Windows,
        ]
    );
    assert_eq!(au_parity.discovered_type_count, 0);

    let lifecycle = runtime.get_plugin_lifecycle_snapshot();
    assert_eq!(lifecycle.sandbox_count, 1);
    assert_eq!(
        lifecycle.sandboxes[0].plugin_format,
        Some(PluginFormat::Clap)
    );
    assert_eq!(lifecycle.parity_coverage.len(), 3);
    assert_eq!(
        lifecycle
            .parity_coverage
            .iter()
            .find(|record| record.format == PluginFormat::Clap)
            .map(|record| record.active_transport_count),
        Some(0)
    );

    let report = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
    assert_eq!(report.plugin_discovery_snapshot.scan_count, 2);
    assert_eq!(report.plugin_discovery_snapshot.discovered_type_count, 2);
    assert_eq!(report.plugin_discovery_snapshot.discovered_format_count, 2);
    assert!(report
        .render_json()
        .contains("\"plugin_discovery_snapshot\":{"));
    assert!(report
        .render_json()
        .contains("\"formats\":[\"Clap\",\"Vst3\"]"));
    assert!(report.render_json().contains("\"discovered_type_count\":2"));
    assert!(report
        .render_json()
        .contains("\"discovered_format_count\":2"));
    assert!(report
        .render_json()
        .contains("\"plugin_type_id\":\"plugin:clap:default\""));
    assert!(report
        .render_json()
        .contains("\"default_multichannel_io\":{"));
    assert!(report
        .render_json()
        .contains("\"plugin_type_id\":\"plugin:vst3:instrument\""));
    assert!(report
        .render_json()
        .contains("\"multi_format_catalog\":true"));
    assert!(report
        .render_json()
        .contains("\"supports_activate_count\":1"));
    assert!(report.render_json().contains("\"format_coverage\":["));
    assert!(report.render_json().contains("\"parity_coverage\":["));
    assert!(report
        .render_json()
        .contains("\"parity_band\":\"Portable\""));
    assert!(report
        .render_json()
        .contains("\"linux_parity_band\":\"Portable\""));
    assert!(report
        .render_json()
        .contains("\"linux_preferred_sandbox_outcome\":\"IsolatedSandbox\""));
    assert!(report
        .render_json()
        .contains("\"unsupported_platforms\":[\"Linux\",\"Windows\"]"));
    assert!(report.render_json().contains("\"supports_snapshot\":true"));
}
