use super::super::*;

#[test]
fn runtime_starts_and_reports_ready() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    handshake_and_configure(&mut runtime);
    runtime.start().unwrap();

    assert_eq!(runtime.get_readiness(), RuntimeReadiness::Ready);
    assert_eq!(runtime.config().profile, RuntimeProfile::Local);
}

#[test]
fn configure_updates_effective_config() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "runtime-test".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .unwrap();
    runtime
        .configure(RuntimeConfigRequest::new(96_000, 256))
        .unwrap();

    let config = runtime.get_effective_config();
    assert_eq!(config.sample_rate.0, 96_000);
    assert_eq!(config.block_size, 256);
}

#[test]
fn configure_resets_runtime_block_timeline() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "runtime-test".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .unwrap();
    let first_sequence = runtime.allocate_block_sequence();
    runtime.record_block_sequence("sandbox-a", 1, "lease-a", first_sequence);

    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .unwrap();

    let timeline = runtime.get_timeline_snapshot();
    assert_eq!(timeline.next_block_sequence, 0);
    assert_eq!(timeline.block_sequence_continuity.segment_count(), 0);
}

#[test]
fn runtime_timeline_tracks_sequences_across_leases() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let first = runtime.allocate_block_sequence();
    runtime.record_block_sequence("sandbox-a", 1, "lease-a", first);
    let second = runtime.allocate_block_sequence();
    runtime.record_block_sequence("sandbox-a", 1, "lease-a", second);
    let third = runtime.allocate_block_sequence();
    runtime.record_block_sequence("sandbox-a", 2, "lease-b", third);

    let timeline = runtime.get_timeline_snapshot();
    assert_eq!(timeline.next_block_sequence, 3);
    assert_eq!(timeline.block_sequence_continuity.segment_count(), 2);
    assert_eq!(timeline.block_sequence_continuity.lease_rollovers, 1);
    assert_eq!(
        timeline.block_sequence_continuity.first_block_sequence(),
        Some(0)
    );
    assert_eq!(
        timeline.block_sequence_continuity.last_block_sequence(),
        Some(2)
    );
}

#[test]
fn configure_resets_runtime_automation_tracking() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "runtime-test".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .unwrap();
    runtime.record_automation_summary(
        1,
        "lease-a",
        ParameterAutomationSummary {
            parameter_id: 4096,
            value_events: 2,
            modulation_events: 2,
            gesture_begin_events: 1,
            gesture_end_events: 1,
            first_value: Some(0.2),
            last_value: Some(0.4),
            last_modulation: Some(0.08),
        },
    );

    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .unwrap();

    let automation = runtime.get_automation_snapshot();
    assert_eq!(automation.parameter_id, 0);
    assert_eq!(automation.segment_count, 0);
    assert_eq!(automation.first_epoch, None);
}

#[test]
fn runtime_automation_tracking_rolls_across_leases() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime.record_automation_summary(
        1,
        "lease-a",
        ParameterAutomationSummary {
            parameter_id: 4096,
            value_events: 2,
            modulation_events: 2,
            gesture_begin_events: 1,
            gesture_end_events: 1,
            first_value: Some(0.2),
            last_value: Some(0.4),
            last_modulation: Some(0.08),
        },
    );
    runtime.record_automation_summary(
        2,
        "lease-b",
        ParameterAutomationSummary {
            parameter_id: 4096,
            value_events: 2,
            modulation_events: 2,
            gesture_begin_events: 0,
            gesture_end_events: 1,
            first_value: Some(0.5),
            last_value: Some(0.7),
            last_modulation: Some(0.12),
        },
    );

    let automation = runtime.get_automation_snapshot();
    assert_eq!(automation.parameter_id, 4096);
    assert_eq!(automation.value_events, 4);
    assert_eq!(automation.segment_count, 2);
    assert_eq!(automation.segment_epochs, vec![1, 2]);
    assert_eq!(automation.lease_rollovers, 1);
    assert_eq!(automation.first_epoch, Some(1));
    assert_eq!(automation.last_epoch, Some(2));
}

#[test]
fn runtime_plugin_event_tracking_rolls_across_leases() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime.record_plugin_event_summary(
        1,
        "lease-a",
        7,
        96,
        EventPacketSummary {
            total_events: 6,
            parameter_value_events: 1,
            parameter_modulation_events: 1,
            parameter_gesture_events: 1,
            note_events: 1,
            note_expression_events: 1,
            note_expression_pressure_events: 1,
            note_expression_timbre_events: 0,
            note_expression_tuning_events: 0,
            midi_events: 1,
        },
    );
    runtime.record_plugin_event_summary(
        2,
        "lease-b",
        8,
        64,
        EventPacketSummary {
            total_events: 5,
            parameter_value_events: 1,
            parameter_modulation_events: 0,
            parameter_gesture_events: 1,
            note_events: 1,
            note_expression_events: 1,
            note_expression_pressure_events: 0,
            note_expression_timbre_events: 0,
            note_expression_tuning_events: 1,
            midi_events: 1,
        },
    );

    let snapshot = runtime.get_plugin_event_snapshot();
    assert_eq!(snapshot.last_processing_epoch, Some(2));
    assert_eq!(snapshot.last_block_sequence, Some(8));
    assert_eq!(snapshot.last_generated_event_bytes, 64);
    assert_eq!(snapshot.last_batch_total_events, 5);
    assert_eq!(snapshot.last_batch_note_expression_events, 1);
    assert_eq!(snapshot.last_batch_note_expression_pressure_events, 0);
    assert_eq!(snapshot.last_batch_note_expression_timbre_events, 0);
    assert_eq!(snapshot.last_batch_note_expression_tuning_events, 1);
    assert_eq!(snapshot.total_events, 11);
    assert_eq!(snapshot.parameter_value_events, 2);
    assert_eq!(snapshot.parameter_modulation_events, 1);
    assert_eq!(snapshot.parameter_gesture_events, 2);
    assert_eq!(snapshot.note_events, 2);
    assert_eq!(snapshot.note_expression_events, 2);
    assert_eq!(snapshot.note_expression_pressure_events, 1);
    assert_eq!(snapshot.note_expression_timbre_events, 0);
    assert_eq!(snapshot.note_expression_tuning_events, 1);
    assert_eq!(snapshot.midi_events, 2);
    assert_eq!(
        snapshot.mpe_posture,
        RuntimeControllerExpressionMpePosture::Guarded
    );
    assert_eq!(
        snapshot.midi2_posture,
        RuntimeControllerExpressionMidi2Posture::Guarded
    );
    assert_eq!(snapshot.first_epoch, Some(1));
    assert_eq!(snapshot.last_epoch, Some(2));
    assert_eq!(snapshot.segment_count, 2);
    assert_eq!(snapshot.segment_epochs, vec![1, 2]);
    assert_eq!(snapshot.lease_rollovers, 1);

    let _observation =
        RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
}

#[test]
fn runtime_plugin_event_tracking_resets_on_reconfigure() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "runtime-test".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .unwrap();
    runtime.record_plugin_event_summary(
        1,
        "lease-a",
        4,
        80,
        EventPacketSummary {
            total_events: 4,
            parameter_value_events: 1,
            parameter_modulation_events: 1,
            parameter_gesture_events: 0,
            note_events: 1,
            note_expression_events: 1,
            note_expression_pressure_events: 1,
            note_expression_timbre_events: 0,
            note_expression_tuning_events: 0,
            midi_events: 0,
        },
    );

    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .unwrap();

    let snapshot = runtime.get_plugin_event_snapshot();
    assert_eq!(snapshot.total_events, 0);
    assert_eq!(snapshot.segment_count, 0);
    assert_eq!(snapshot.first_epoch, None);
    assert_eq!(snapshot.last_processing_epoch, None);
}
