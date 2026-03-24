use signal_host_server::ServerRuntimeHost;
use signal_plugin::{EventPacketSummary, PluginFormat};
use signal_runtime::{RuntimeConfig, RuntimeSupervisorApi, SignalRuntime};

#[test]
fn server_shared_host_edge_exports_runtime_generic_event_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    runtime.record_plugin_event_summary(
        11,
        "lease:public-server-events",
        18,
        212,
        EventPacketSummary {
            total_events: 9,
            parameter_value_events: 1,
            parameter_modulation_events: 1,
            parameter_gesture_events: 1,
            note_events: 2,
            note_expression_events: 3,
            note_expression_pressure_events: 1,
            note_expression_timbre_events: 1,
            note_expression_tuning_events: 1,
            midi_events: 1,
        },
    );
    let mut host = ServerRuntimeHost::new(runtime);

    host.start_plugin_scan(signal_runtime::PluginScanRequest {
        roots: vec!["~/.clap".into(), "/usr/lib/vst3".into()],
        formats: vec![PluginFormat::Clap, PluginFormat::Vst3],
    })
    .expect("public server generic event scan should succeed");

    let report = host.supervisor_report();
    let snapshot = &report.observation.plugin_event_snapshot;
    assert_eq!(snapshot.last_processing_epoch, Some(11));
    assert_eq!(snapshot.last_block_sequence, Some(18));
    assert_eq!(snapshot.last_generated_event_bytes, 212);
    assert_eq!(snapshot.total_events, 9);
    assert_eq!(snapshot.note_expression_events, 3);
    assert_eq!(snapshot.midi_events, 1);
    assert_eq!(snapshot.segment_epochs, vec![11]);
    assert!(
        report
            .observation
            .plugin_discovery_snapshot
            .capability_coverage
            .supports_note_expression_count
            >= 2
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"plugin_events\":{"));
    assert!(rendered.contains("\"note_expression_events\":3"));
    assert!(rendered.contains("\"supports_note_expression_count\":"));
}

#[test]
fn server_shared_host_edge_exports_runtime_controller_expression_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    runtime.record_plugin_event_summary(
        17,
        "lease:public-server-controller-expression",
        27,
        240,
        EventPacketSummary {
            total_events: 10,
            parameter_value_events: 1,
            parameter_modulation_events: 1,
            parameter_gesture_events: 1,
            note_events: 2,
            note_expression_events: 4,
            note_expression_pressure_events: 1,
            note_expression_timbre_events: 1,
            note_expression_tuning_events: 2,
            midi_events: 1,
        },
    );
    let mut host = ServerRuntimeHost::new(runtime);

    host.start_plugin_scan(signal_runtime::PluginScanRequest {
        roots: vec!["~/.clap".into(), "/usr/lib/vst3".into()],
        formats: vec![PluginFormat::Clap, PluginFormat::Vst3],
    })
    .expect("public server controller-expression scan should succeed");

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
    assert_eq!(
        report.observation.external_midi_snapshot.graph_state,
        signal_runtime::RuntimeExternalMidiGraphState::Empty
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"note_expression_pressure_events\":1"));
    assert!(rendered.contains("\"note_expression_timbre_events\":1"));
    assert!(rendered.contains("\"note_expression_tuning_events\":2"));
    assert!(rendered.contains("\"mpe_posture\":\"Guarded\""));
    assert!(rendered.contains("\"midi2_posture\":\"Guarded\""));
    assert!(rendered.contains("\"external_midi_snapshot\":{"));
    assert!(rendered.contains("\"graph_state\":\"Empty\""));
}
