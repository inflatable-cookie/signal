use super::*;

#[test]
fn restart_reconfigures_runtime() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "runtime-test".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .unwrap();
    runtime
        .restart(RestartRequest {
            reconfigure: Some(RuntimeConfigRequest::new(44_100, 128)),
        })
        .unwrap();

    assert_eq!(runtime.get_effective_config().sample_rate.0, 44_100);
    assert_eq!(runtime.get_readiness(), RuntimeReadiness::Ready);
}

#[test]
fn transport_projection_rejects_non_positive_tempo() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let error = runtime
        .apply_transport_projection(TransportProjection {
            playing: true,
            timeline_position_samples: 0,
            tempo_bpm: 0.0,
            loop_state: None,
        })
        .unwrap_err();

    assert_eq!(
        error.kind,
        crate::interfaces::RuntimeErrorKind::InvalidRequest
    );
}

#[test]
fn runtime_classifies_transport_invalidation_boundaries() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    apply_latency_runtime_graph(&mut runtime, "graph:runtime:transport-boundaries");

    let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 31);
    runtime.process_engine_block(1, 1, block.clone()).unwrap();

    runtime
        .apply_transport_projection(TransportProjection {
            playing: true,
            timeline_position_samples: 64,
            tempo_bpm: 120.0,
            loop_state: None,
        })
        .unwrap();
    let started = runtime.get_engine_block_snapshot();
    assert_eq!(
        started.last_prework_invalidation_reason,
        Some(RuntimePreworkInvalidationReason::TransportStarted)
    );
    assert_eq!(
        runtime.get_timeline_snapshot().last_transport_transition,
        Some(crate::interfaces::RuntimeTransportTransitionKind::Started)
    );

    runtime.process_engine_block(2, 2, block.clone()).unwrap();
    runtime
        .apply_transport_projection(TransportProjection {
            playing: true,
            timeline_position_samples: 512,
            tempo_bpm: 120.0,
            loop_state: None,
        })
        .unwrap();
    let seeked = runtime.get_engine_block_snapshot();
    assert_eq!(
        seeked.last_prework_invalidation_reason,
        Some(RuntimePreworkInvalidationReason::TransportSeeked)
    );
    assert_eq!(
        runtime.get_timeline_snapshot().last_transport_transition,
        Some(crate::interfaces::RuntimeTransportTransitionKind::Seeked)
    );

    runtime.process_engine_block(3, 3, block.clone()).unwrap();
    runtime
        .apply_transport_projection(TransportProjection {
            playing: true,
            timeline_position_samples: 520,
            tempo_bpm: 130.0,
            loop_state: None,
        })
        .unwrap();
    let tempo_changed = runtime.get_engine_block_snapshot();
    assert_eq!(
        tempo_changed.last_prework_invalidation_reason,
        Some(RuntimePreworkInvalidationReason::TransportTempoChanged)
    );
    assert_eq!(
        runtime.get_timeline_snapshot().last_transport_transition,
        Some(crate::interfaces::RuntimeTransportTransitionKind::TempoChanged)
    );

    runtime.process_engine_block(4, 4, block.clone()).unwrap();
    runtime
        .apply_transport_projection(TransportProjection {
            playing: true,
            timeline_position_samples: 528,
            tempo_bpm: 130.0,
            loop_state: Some(crate::interfaces::LoopRegion {
                start_samples: 256,
                end_samples: 1024,
            }),
        })
        .unwrap();
    let loop_state_changed = runtime.get_engine_block_snapshot();
    assert_eq!(
        loop_state_changed.last_prework_invalidation_reason,
        Some(RuntimePreworkInvalidationReason::TransportLoopStateChanged)
    );
    assert_eq!(
        runtime.get_timeline_snapshot().last_transport_transition,
        Some(crate::interfaces::RuntimeTransportTransitionKind::LoopStateChanged)
    );

    runtime.process_engine_block(5, 5, block).unwrap();
    runtime
        .apply_transport_projection(TransportProjection {
            playing: false,
            timeline_position_samples: 536,
            tempo_bpm: 130.0,
            loop_state: Some(crate::interfaces::LoopRegion {
                start_samples: 256,
                end_samples: 1024,
            }),
        })
        .unwrap();
    let stopped = runtime.get_engine_block_snapshot();
    assert_eq!(
        stopped.last_prework_invalidation_reason,
        Some(RuntimePreworkInvalidationReason::TransportStopped)
    );
    assert_eq!(
        runtime.get_timeline_snapshot().last_transport_transition,
        Some(crate::interfaces::RuntimeTransportTransitionKind::Stopped)
    );
}

#[test]
fn runtime_records_transport_progression_in_timeline_and_engine_snapshot() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    apply_latency_runtime_graph(&mut runtime, "graph:runtime:transport-progression");

    let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 41);
    runtime.process_engine_block(1, 1, block.clone()).unwrap();
    runtime
        .apply_transport_projection(TransportProjection {
            playing: true,
            timeline_position_samples: 64,
            tempo_bpm: 120.0,
            loop_state: None,
        })
        .unwrap();

    let result = runtime.process_engine_block(2, 2, block).unwrap();
    assert_eq!(result.snapshot.transport_epoch, 1);
    assert_eq!(
        result.snapshot.transport_transition,
        Some(crate::interfaces::RuntimeTransportTransitionKind::Started)
    );
    assert_eq!(result.snapshot.transport_block_start_samples, Some(64));
    assert_eq!(result.snapshot.transport_block_end_samples, Some(72));
    assert!(!result.snapshot.transport_loop_wrapped);

    let timeline = runtime.get_timeline_snapshot();
    assert_eq!(timeline.transport_epoch, 1);
    assert_eq!(
        timeline.last_transport_transition,
        Some(crate::interfaces::RuntimeTransportTransitionKind::Started)
    );
    assert_eq!(timeline.last_transport_transition_block_sequence, Some(2));
    assert_eq!(timeline.last_transport_playing, Some(true));
    assert_eq!(timeline.last_transport_tempo_bpm, Some(120.0));
    assert_eq!(timeline.last_transport_timeline_position_samples, Some(72));
    assert_eq!(timeline.last_engine_block_start_samples, Some(64));
    assert_eq!(timeline.last_engine_block_end_samples, Some(72));
    assert_eq!(timeline.loop_wrap_count, 0);

    let report = crate::interfaces::RuntimeObservationReport::capture(
        &runtime,
        &RuntimeEventRecorder::default(),
    );
    let compact = report.render_compact();
    assert!(compact.contains("transport_epoch=1"));
    assert!(compact.contains("engine_transport_transition=Some(Started)"));
    let json = crate::interfaces::RuntimeSupervisorReport::capture(
        &runtime,
        &RuntimeEventRecorder::default(),
    )
    .render_json();
    assert!(json.contains("\"transport_epoch\":1"));
    assert!(json.contains("\"transport_transition\":\"Started\""));

    let transport = runtime.get_transport_observation_snapshot();
    assert_eq!(transport.transport_epoch, 1);
    assert_eq!(transport.projected_playing, Some(true));
    assert_eq!(transport.projected_tempo_bpm, Some(120.0));
    assert_eq!(transport.projected_timeline_position_samples, Some(72));
    assert_eq!(transport.observed_playing, Some(true));
    assert_eq!(transport.observed_tempo_bpm, Some(120.0));
    assert_eq!(transport.observed_timeline_position_samples, Some(72));
    assert_eq!(
        transport.last_transition,
        Some(crate::interfaces::RuntimeTransportTransitionKind::Started)
    );
    assert_eq!(transport.last_transition_block_sequence, Some(2));
    assert_eq!(transport.last_engine_block_start_samples, Some(64));
    assert_eq!(transport.last_engine_block_end_samples, Some(72));
    assert_eq!(transport.loop_wrap_count, 0);
}

#[test]
fn runtime_seek_invalidation_projects_into_export_summaries_on_real_engine_path() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    apply_latency_runtime_graph(&mut runtime, "graph:runtime:seek-export");

    let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 43);
    runtime.process_engine_block(1, 1, block.clone()).unwrap();
    runtime
        .apply_transport_projection(TransportProjection {
            playing: true,
            timeline_position_samples: 64,
            tempo_bpm: 120.0,
            loop_state: None,
        })
        .unwrap();
    runtime.process_engine_block(2, 2, block.clone()).unwrap();
    runtime
        .apply_transport_projection(TransportProjection {
            playing: true,
            timeline_position_samples: 512,
            tempo_bpm: 120.0,
            loop_state: None,
        })
        .unwrap();
    let boundary_report = crate::interfaces::RuntimeObservationReport::capture(
        &runtime,
        &RuntimeEventRecorder::default(),
    );
    assert_eq!(
        boundary_report
            .block_summary
            .last_prework_invalidation_reason,
        Some(RuntimePreworkInvalidationReason::TransportSeeked)
    );

    let result = runtime.process_engine_block(3, 3, block).unwrap();
    assert_eq!(
        result.snapshot.transport_transition,
        Some(crate::interfaces::RuntimeTransportTransitionKind::Seeked)
    );
    assert_eq!(
        result.snapshot.last_prework_invalidation_reason,
        Some(RuntimePreworkInvalidationReason::ProcessingEpochExpired)
    );

    let report = crate::interfaces::RuntimeObservationReport::capture(
        &runtime,
        &RuntimeEventRecorder::default(),
    );
    assert_eq!(
        report.block_summary.transport_transition,
        Some(crate::interfaces::RuntimeTransportTransitionKind::Seeked)
    );
    assert_eq!(
        report.block_summary.last_prework_invalidation_reason,
        Some(RuntimePreworkInvalidationReason::ProcessingEpochExpired)
    );

    let supervisor = crate::interfaces::RuntimeSupervisorReport::capture(
        &runtime,
        &RuntimeEventRecorder::default(),
    );
    assert!(supervisor
        .render_multiline()
        .contains("block_summary_transport=2/Some(Seeked)/false"));
    let json = supervisor.render_json();
    assert!(json.contains("\"block_summary\":{"));
    assert!(json.contains("\"transport_transition\":\"Seeked\""));
    assert!(json.contains("\"last_prework_invalidation_reason\":\"ProcessingEpochExpired\""));
}
