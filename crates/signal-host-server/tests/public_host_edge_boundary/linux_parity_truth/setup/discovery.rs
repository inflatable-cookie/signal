use super::super::super::*;

pub(super) fn record_server_linux_parity_discovery(runtime: &mut SignalRuntime) {
    runtime.record_plugin_format_platform_coverage(vec![
        signal_runtime::RuntimePluginFormatPlatformCoverageRecord {
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
        signal_runtime::RuntimePluginFormatPlatformCoverageRecord {
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
        signal_runtime::RuntimePluginFormatPlatformCoverageRecord {
            format: PluginFormat::Lv2,
            supported_platforms: vec![RuntimePluginHostPlatform::Linux],
            unsupported_platforms: vec![
                RuntimePluginHostPlatform::MacOs,
                RuntimePluginHostPlatform::Windows,
            ],
            linux_parity_band: RuntimePluginParityBand::Portable,
            linux_preferred_sandbox_outcome: Some(RuntimePluginIsolationOutcome::IsolatedSandbox),
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
                    rule_id: "server-linux-share-clap".into(),
                    matcher: RuntimePluginPlacementRuleMatcher::PluginFormat(PluginFormat::Clap),
                    outcome: RuntimePluginIsolationOutcome::SharedSandbox,
                    sandbox_group_key: Some("linux:clap".into()),
                },
                RuntimePluginPlacementRule {
                    rule_id: "server-linux-inline-vst3".into(),
                    matcher: RuntimePluginPlacementRuleMatcher::PluginFormat(PluginFormat::Vst3),
                    outcome: RuntimePluginIsolationOutcome::InProcess,
                    sandbox_group_key: None,
                },
            ],
        })
        .expect("public server linux parity placement policy should apply");

    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec![
            "~/.clap".into(),
            "/usr/lib/vst3".into(),
            "/usr/lib/lv2".into(),
        ],
        formats: vec![PluginFormat::Clap, PluginFormat::Vst3, PluginFormat::Lv2],
    });
    runtime.record_plugin_scan_results(
        scan_handle,
        vec![
            signal_runtime::RuntimePluginDiscoveredTypeRecord {
                plugin_type_id: "plugin:clap:server-linux-parity".into(),
                plugin_id: "com.signal.server-linux-parity-clap".into(),
                vendor: "Signal".into(),
                name: "Server Linux Parity CLAP".into(),
                format: PluginFormat::Clap,
                version: Some("1.0.0".into()),
                features: vec![PluginFeature::AudioEffect],
                default_io_layout: PluginIoLayout {
                    audio_inputs: 2,
                    audio_outputs: 2,
                    midi_inputs: 0,
                    midi_outputs: 0,
                },
                default_multichannel_io:
                    signal_runtime::RuntimeMultichannelIoSummary::for_plugin_io(PluginIoLayout {
                        audio_inputs: 2,
                        audio_outputs: 2,
                        midi_inputs: 0,
                        midi_outputs: 0,
                    }),
                complex_io_summary: RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
                    &[PluginFeature::AudioEffect],
                    PluginIoLayout {
                        audio_inputs: 2,
                        audio_outputs: 2,
                        midi_inputs: 0,
                        midi_outputs: 0,
                    },
                ),
                audio_bus_count: 1,
                parameter_count: 8,
                state_contract: signal_plugin::PluginStateContract {
                    supports_snapshot: true,
                    supports_reset: true,
                    supports_bypass: true,
                    exposes_latency: false,
                    exposes_tail: false,
                },
                processing_contract: signal_plugin::PluginProcessingContract {
                    max_block_frames: 2048,
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
                summary: "server linux parity clap".into(),
            },
            RuntimePluginDiscoveredTypeRecord {
                plugin_type_id: "plugin:vst3:server-linux-parity".into(),
                plugin_id: "com.signal.server-linux-parity-vst3".into(),
                vendor: "Signal".into(),
                name: "Server Linux Parity VST3".into(),
                format: PluginFormat::Vst3,
                version: Some("1.0.0".into()),
                features: vec![PluginFeature::Instrument],
                default_io_layout: PluginIoLayout {
                    audio_inputs: 0,
                    audio_outputs: 2,
                    midi_inputs: 1,
                    midi_outputs: 0,
                },
                default_multichannel_io:
                    signal_runtime::RuntimeMultichannelIoSummary::for_plugin_io(PluginIoLayout {
                        audio_inputs: 0,
                        audio_outputs: 2,
                        midi_inputs: 1,
                        midi_outputs: 0,
                    }),
                complex_io_summary: RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
                    &[PluginFeature::Instrument],
                    PluginIoLayout {
                        audio_inputs: 0,
                        audio_outputs: 2,
                        midi_inputs: 1,
                        midi_outputs: 0,
                    },
                ),
                audio_bus_count: 1,
                parameter_count: 12,
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
                    requires_main_thread_for_state: false,
                    supports_prepare: true,
                    supports_activate: true,
                    supports_reset_while_active: true,
                },
                lv2_extension_capabilities: None,
                summary: "server linux parity vst3".into(),
            },
            RuntimePluginDiscoveredTypeRecord {
                plugin_type_id: "plugin:lv2:server-linux-parity".into(),
                plugin_id: "com.signal.server-linux-parity-lv2".into(),
                vendor: "Signal".into(),
                name: "Server Linux Parity LV2".into(),
                format: PluginFormat::Lv2,
                version: Some("1.0.0".into()),
                features: vec![PluginFeature::Utility],
                default_io_layout: PluginIoLayout {
                    audio_inputs: 2,
                    audio_outputs: 2,
                    midi_inputs: 0,
                    midi_outputs: 0,
                },
                default_multichannel_io:
                    signal_runtime::RuntimeMultichannelIoSummary::for_plugin_io(PluginIoLayout {
                        audio_inputs: 2,
                        audio_outputs: 2,
                        midi_inputs: 0,
                        midi_outputs: 0,
                    }),
                complex_io_summary: RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
                    &[PluginFeature::Utility],
                    PluginIoLayout {
                        audio_inputs: 2,
                        audio_outputs: 2,
                        midi_inputs: 0,
                        midi_outputs: 0,
                    },
                ),
                audio_bus_count: 1,
                parameter_count: 6,
                state_contract: signal_plugin::PluginStateContract {
                    supports_snapshot: true,
                    supports_reset: true,
                    supports_bypass: true,
                    exposes_latency: false,
                    exposes_tail: false,
                },
                processing_contract: signal_plugin::PluginProcessingContract {
                    max_block_frames: 2048,
                    sample_accurate_automation: false,
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
                summary: "server linux parity lv2".into(),
            },
        ],
    );
}
