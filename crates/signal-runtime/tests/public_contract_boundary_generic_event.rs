use signal_plugin::{EventPacketSummary, PluginFormat};
use signal_runtime::{
    PluginScanRequest, RuntimeConfig, RuntimeEventRecorder, RuntimeObservationReport,
    RuntimeSupervisorReport, SignalRuntime,
};

#[path = "support/public_contract_boundary_plugin_records_core.rs"]
mod public_contract_boundary_plugin_records_core_support;

use public_contract_boundary_plugin_records_core_support::{
    sample_backend_breadth_record, sample_discovered_type_record,
};

#[test]
fn public_runtime_generic_event_boundary_reports_runtime_owned_event_and_capability_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let recorder = RuntimeEventRecorder::default();

    let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
        roots: vec!["~/.clap".into(), "~/.vst3".into()],
        formats: vec![PluginFormat::Clap, PluginFormat::Vst3],
    });
    runtime.record_plugin_scan_results(
        scan_handle,
        vec![
            sample_discovered_type_record(),
            sample_backend_breadth_record(),
        ],
    );
    runtime.record_plugin_event_summary(
        7,
        "lease-public-events",
        12,
        144,
        EventPacketSummary {
            total_events: 7,
            parameter_value_events: 1,
            parameter_modulation_events: 1,
            parameter_gesture_events: 1,
            note_events: 1,
            note_expression_events: 2,
            note_expression_pressure_events: 1,
            note_expression_timbre_events: 0,
            note_expression_tuning_events: 1,
            midi_events: 1,
        },
    );

    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);

    let snapshot = &observation.plugin_event_snapshot;
    assert_eq!(snapshot.last_processing_epoch, Some(7));
    assert_eq!(snapshot.last_block_sequence, Some(12));
    assert_eq!(snapshot.last_generated_event_bytes, 144);
    assert_eq!(snapshot.total_events, 7);
    assert_eq!(snapshot.note_expression_events, 2);
    assert_eq!(snapshot.note_expression_pressure_events, 1);
    assert_eq!(snapshot.note_expression_timbre_events, 0);
    assert_eq!(snapshot.note_expression_tuning_events, 1);
    assert_eq!(snapshot.midi_events, 1);
    assert_eq!(
        snapshot.mpe_posture,
        signal_runtime::RuntimeControllerExpressionMpePosture::Guarded
    );
    assert_eq!(
        snapshot.midi2_posture,
        signal_runtime::RuntimeControllerExpressionMidi2Posture::Guarded
    );
    assert_eq!(snapshot.segment_count, 1);
    assert_eq!(snapshot.segment_epochs, vec![7]);
    assert_eq!(
        observation
            .plugin_discovery_snapshot
            .capability_coverage
            .supports_note_expression_count,
        2
    );

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"plugin_events\":{"));
    assert!(observation_json.contains("\"note_expression_events\":2"));
    assert!(observation_json.contains("\"supports_note_expression_count\":2"));

    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"plugin_events\":{"));
    assert!(supervisor_json.contains("\"last_generated_event_bytes\":144"));
    assert!(supervisor_json.contains("\"supports_note_expression_count\":2"));
}
