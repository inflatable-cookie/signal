#[path = "support/public_host_edge_plugins.rs"]
mod public_host_edge_plugins_support;

use public_host_edge_plugins_support::temp_public_local_vst3_scan_root;
use signal_host_local::LocalRuntimeHost;
use signal_plugin::{EventPacketSummary, PluginFormat};
use signal_runtime::{RuntimeConfig, RuntimeSupervisorApi, SignalRuntime};

#[test]
fn local_shared_host_edge_exports_runtime_generic_event_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime.record_plugin_event_summary(
        9,
        "lease:public-local-events",
        14,
        196,
        EventPacketSummary {
            total_events: 8,
            parameter_value_events: 1,
            parameter_modulation_events: 1,
            parameter_gesture_events: 1,
            note_events: 1,
            note_expression_events: 3,
            note_expression_pressure_events: 1,
            note_expression_timbre_events: 1,
            note_expression_tuning_events: 1,
            midi_events: 1,
        },
    );
    let mut host = LocalRuntimeHost::new(runtime);
    let vst3_root = temp_public_local_vst3_scan_root();

    host.start_plugin_scan(signal_runtime::PluginScanRequest {
        roots: vec!["scan:clap:local-events".into(), vst3_root.root()],
        formats: vec![PluginFormat::Clap, PluginFormat::Vst3],
    })
    .expect("public local generic event scan should succeed");

    let report = host.supervisor_report();
    let snapshot = &report.observation.plugin_event_snapshot;
    assert_eq!(snapshot.last_processing_epoch, Some(9));
    assert_eq!(snapshot.last_block_sequence, Some(14));
    assert_eq!(snapshot.last_generated_event_bytes, 196);
    assert_eq!(snapshot.total_events, 8);
    assert_eq!(snapshot.note_expression_events, 3);
    assert_eq!(snapshot.midi_events, 1);
    assert_eq!(snapshot.segment_epochs, vec![9]);
    assert_eq!(
        report
            .observation
            .plugin_discovery_snapshot
            .capability_coverage
            .supports_note_expression_count,
        2
    );

}

#[test]
fn local_shared_host_edge_exports_runtime_controller_expression_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime.record_plugin_event_summary(
        15,
        "lease:public-local-controller-expression",
        24,
        224,
        EventPacketSummary {
            total_events: 9,
            parameter_value_events: 1,
            parameter_modulation_events: 1,
            parameter_gesture_events: 1,
            note_events: 1,
            note_expression_events: 4,
            note_expression_pressure_events: 1,
            note_expression_timbre_events: 1,
            note_expression_tuning_events: 2,
            midi_events: 1,
        },
    );
    let mut host = LocalRuntimeHost::new(runtime);
    let vst3_root = temp_public_local_vst3_scan_root();

    host.start_plugin_scan(signal_runtime::PluginScanRequest {
        roots: vec![
            "scan:clap:local-controller-expression".into(),
            vst3_root.root(),
        ],
        formats: vec![PluginFormat::Clap, PluginFormat::Vst3],
    })
    .expect("public local controller-expression scan should succeed");

    let report = host.supervisor_report();
    let snapshot = &report.observation.plugin_event_snapshot;
    assert_eq!(snapshot.note_expression_pressure_events, 1);
    assert_eq!(snapshot.note_expression_timbre_events, 1);
    assert_eq!(snapshot.note_expression_tuning_events, 2);
    assert_eq!(
        snapshot.mpe_posture,
        signal_runtime::RuntimeControllerExpressionMpePosture::Guarded
    );
    assert_eq!(
        snapshot.midi2_posture,
        signal_runtime::RuntimeControllerExpressionMidi2Posture::Guarded
    );

}
