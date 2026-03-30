use super::super::*;

#[test]
fn runtime_linux_plugin_parity_coverage_tracks_policy_render_failure_and_restart_receipts() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 256));
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
            linux_preferred_sandbox_outcome: Some(
                RuntimePluginIsolationOutcome::IsolatedSandbox,
            ),
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
            linux_preferred_sandbox_outcome: Some(
                RuntimePluginIsolationOutcome::IsolatedSandbox,
            ),
            linux_strict_sandbox_default: true,
            summary:
                "platforms=MacOs/Linux/Windows linux=Portable linux_policy=IsolatedSandbox unsupported=none"
                    .into(),
        },
        RuntimePluginFormatPlatformCoverageRecord {
            format: PluginFormat::Lv2,
            supported_platforms: vec![RuntimePluginHostPlatform::Linux],
            unsupported_platforms: vec![
                RuntimePluginHostPlatform::MacOs,
                RuntimePluginHostPlatform::Windows,
            ],
            linux_parity_band: RuntimePluginParityBand::Portable,
            linux_preferred_sandbox_outcome: Some(
                RuntimePluginIsolationOutcome::IsolatedSandbox,
            ),
            linux_strict_sandbox_default: true,
            summary:
                "platforms=Linux linux=Portable linux_policy=IsolatedSandbox unsupported=MacOs/Windows"
                    .into(),
        },
    ]);
    runtime
        .apply_plugin_placement_policy(RuntimePluginPlacementPolicy {
            default_outcome: RuntimePluginIsolationOutcome::IsolatedSandbox,
            rules: vec![
                RuntimePluginPlacementRule {
                    rule_id: "share-clap-linux".into(),
                    matcher: RuntimePluginPlacementRuleMatcher::PluginFormat(PluginFormat::Clap),
                    outcome: RuntimePluginIsolationOutcome::SharedSandbox,
                    sandbox_group_key: Some("linux:clap".into()),
                },
                RuntimePluginPlacementRule {
                    rule_id: "inline-vst3-linux".into(),
                    matcher: RuntimePluginPlacementRuleMatcher::PluginFormat(PluginFormat::Vst3),
                    outcome: RuntimePluginIsolationOutcome::InProcess,
                    sandbox_group_key: None,
                },
            ],
        })
        .expect("apply linux placement policy");

    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/.clap".into(), "~/.vst3".into(), "~/.lv2".into()],
        formats: vec![PluginFormat::Clap, PluginFormat::Vst3, PluginFormat::Lv2],
    });
    let sample_record = |plugin_type_id: &str,
                         format: PluginFormat,
                         features: Vec<PluginFeature>,
                         io: PluginIoLayout,
                         supports_prepare: bool,
                         supports_activate: bool| {
        crate::RuntimePluginDiscoveredTypeRecord {
            plugin_type_id: plugin_type_id.into(),
            plugin_id: format!("com.signal.{}", plugin_type_id.replace(':', ".")),
            vendor: "Signal".into(),
            name: plugin_type_id.into(),
            format,
            version: Some("1.0.0".into()),
            features: features.clone(),
            default_io_layout: io,
            default_multichannel_io: crate::RuntimeMultichannelIoSummary::for_plugin_io(io),
            complex_io_summary:
                crate::RuntimePluginComplexIoSummary::from_plugin_features_and_layout(&features, io),
            audio_bus_count: 1,
            parameter_count: 8,
            state_contract: PluginStateContract {
                supports_snapshot: true,
                supports_reset: true,
                supports_bypass: true,
                exposes_latency: false,
                exposes_tail: false,
            },
            processing_contract: PluginProcessingContract {
                max_block_frames: 2048,
                sample_accurate_automation: true,
                accepts_midi: io.midi_inputs > 0,
                accepts_note_events: io.midi_inputs > 0,
                supports_note_expression: io.midi_inputs > 0,
                produces_midi: io.midi_outputs > 0,
                silence_aware: true,
            },
            lifecycle_contract: PluginLifecycleContract {
                requires_main_thread_for_state: false,
                supports_prepare,
                supports_activate,
                supports_reset_while_active: supports_activate,
            },
            lv2_extension_capabilities: (format == PluginFormat::Lv2).then(|| {
                crate::RuntimeLv2ExtensionCapabilitySummary::from_lv2_feature_uris(
                    &["http://lv2plug.in/ns/ext/urid#map".into()],
                    &["http://lv2plug.in/ns/ext/patch#Message".into()],
                )
            }),
            summary: format!("plugin_type={plugin_type_id} format={format:?}"),
        }
    };
    runtime.record_plugin_scan_results(
        scan_handle,
        vec![
            sample_record(
                "plugin:clap:linux-parity",
                PluginFormat::Clap,
                vec![PluginFeature::AudioEffect],
                PluginIoLayout {
                    audio_inputs: 2,
                    audio_outputs: 2,
                    midi_inputs: 0,
                    midi_outputs: 0,
                },
                true,
                true,
            ),
            sample_record(
                "plugin:vst3:linux-parity",
                PluginFormat::Vst3,
                vec![PluginFeature::Instrument],
                PluginIoLayout {
                    audio_inputs: 0,
                    audio_outputs: 2,
                    midi_inputs: 1,
                    midi_outputs: 0,
                },
                true,
                true,
            ),
            sample_record(
                "plugin:lv2:linux-parity",
                PluginFormat::Lv2,
                vec![PluginFeature::Utility],
                PluginIoLayout {
                    audio_inputs: 2,
                    audio_outputs: 2,
                    midi_inputs: 0,
                    midi_outputs: 0,
                },
                true,
                true,
            ),
        ],
    );

    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "linux-clap-sandbox".into(),
        plugin_format: PluginFormat::Clap,
        plugin_type_id: Some("plugin:clap:linux-parity".into()),
    });
    runtime.record_plugin_sandbox_lifecycle(
        "linux-clap-sandbox",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_plugin_sandbox_transport(
        "linux-clap-sandbox",
        "lease-clap",
        "region-clap",
        PluginSandboxTransportStage::Attached,
        Some(1),
        None,
    );

    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "linux-vst3-sandbox".into(),
        plugin_format: PluginFormat::Vst3,
        plugin_type_id: Some("plugin:vst3:linux-parity".into()),
    });
    runtime.record_recovery_cycle(
        "linux-vst3-sandbox",
        RecoveryRestartIntent::CrashRecovery,
        StopReason::DegradedModeRecovery,
        Some(2),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "linux-vst3-sandbox",
        PluginSandboxLifecycleStage::SandboxRestarted,
        Some(2),
    );

    runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
        sandbox_id: "linux-lv2-sandbox".into(),
        plugin_format: PluginFormat::Lv2,
        plugin_type_id: Some("plugin:lv2:linux-parity".into()),
    });
    runtime.record_plugin_sandbox_fault(
        "linux-lv2-sandbox",
        PluginFaultKind::Crash,
        "linux lv2 sandbox fault",
        Some(3),
    );

    let lifecycle = runtime.get_plugin_lifecycle_snapshot();
    let clap = lifecycle
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Clap)
        .expect("clap linux parity should be present");
    assert_eq!(clap.linux_parity_band, RuntimePluginParityBand::Portable);
    assert!(clap.linux_supported);
    assert_eq!(
        clap.linux_preferred_sandbox_outcome,
        Some(RuntimePluginIsolationOutcome::IsolatedSandbox)
    );
    assert!(clap.linux_strict_sandbox_default);
    assert_eq!(clap.prepare_capable_type_count, 1);
    assert_eq!(clap.activate_capable_type_count, 1);
    assert_eq!(clap.shared_sandbox_count, 1);
    assert_eq!(clap.active_transport_count, 1);

    let vst3 = lifecycle
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Vst3)
        .expect("vst3 linux parity should be present");
    assert_eq!(vst3.linux_parity_band, RuntimePluginParityBand::Portable);
    assert!(vst3.linux_supported);
    assert_eq!(vst3.in_process_sandbox_count, 1);
    assert_eq!(vst3.restarting_sandbox_count, 1);
    assert_eq!(vst3.rebindable_sandbox_count, 1);
    assert_eq!(vst3.prepare_capable_type_count, 1);

    let lv2 = lifecycle
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Lv2)
        .expect("lv2 linux parity should be present");
    assert_eq!(lv2.parity_band, RuntimePluginParityBand::Guarded);
    assert_eq!(lv2.linux_parity_band, RuntimePluginParityBand::Portable);
    assert!(lv2.linux_supported);
    assert_eq!(lv2.faulted_sandbox_count, 1);
    assert_eq!(
        lv2.linux_preferred_sandbox_outcome,
        Some(RuntimePluginIsolationOutcome::IsolatedSandbox)
    );

    let observation = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
    assert_eq!(observation.lv2_extension_snapshot.plugin_type_count, 1);
    assert_eq!(
        observation
            .lv2_extension_snapshot
            .worker_required_type_count,
        0
    );
    assert_eq!(
        observation
            .lv2_extension_snapshot
            .patch_supported_type_count,
        0
    );
    assert_eq!(observation.lv2_extension_snapshot.unavailable_type_count, 1);
    let lv2_extension = observation
        .lv2_extension_snapshot
        .records
        .iter()
        .find(|record| record.plugin_type_id == "plugin:lv2:linux-parity")
        .expect("lv2 extension snapshot should be present");
    assert_eq!(
        lv2_extension.worker_posture,
        crate::RuntimeLv2WorkerPosture::WorkerAbsent
    );
    assert_eq!(
        lv2_extension.urid_negotiation_posture,
        crate::RuntimeLv2UridNegotiationPosture::Unavailable
    );
    assert_eq!(
        lv2_extension.patch_exchange_posture,
        crate::RuntimeLv2PatchExchangePosture::Unavailable
    );
    assert_eq!(
        lv2_extension.extension_negotiation_state,
        crate::RuntimeLv2ExtensionNegotiationState::Unavailable
    );

    let rendered = observation.render_json();
    assert!(rendered.contains("\"linux_parity_band\":\"Portable\""));
    assert!(rendered.contains("\"linux_preferred_sandbox_outcome\":\"IsolatedSandbox\""));
    assert!(rendered.contains("\"linux_strict_sandbox_default\":true"));
    assert!(rendered.contains("\"restarting_sandbox_count\":1"));
    assert!(rendered.contains("\"faulted_sandbox_count\":1"));
    assert!(rendered.contains("\"lv2_extension_snapshot\":{"));
    assert!(rendered.contains("\"urid_negotiation_posture\":\"Unavailable\""));
}
