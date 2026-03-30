use super::super::*;

#[test]
fn runtime_performance_trace_receipt_summarizes_playback_recording_and_deferred_work_window() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    apply_latency_runtime_graph(&mut runtime, "graph:runtime:performance-trace");
    runtime.set_cpu_load_percent(13.5);
    runtime.set_graph_latency_ms(5.25);

    let capture_path = temp_capture_path("performance-trace");
    let mut reports = Vec::new();
    reports.push(RuntimeSupervisorReport::capture(
        &runtime,
        &RuntimeEventRecorder::default(),
    ));

    runtime
        .apply_transport_projection(TransportProjection {
            playing: true,
            timeline_position_samples: 4_096,
            tempo_bpm: 120.0,
            loop_state: None,
        })
        .unwrap();
    runtime
        .start_recording_capture(RuntimeRecordingCaptureStartRequest {
            capture_kind: RuntimeRecordingCaptureKind::Audio,
            take_id: "take:test:performance-trace".to_string(),
            track_id: "track:test:performance-trace".to_string(),
            start_samples: 4_096,
            capture_path: capture_path.display().to_string(),
        })
        .unwrap();
    runtime
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(16), 19),
        )
        .unwrap();
    reports.push(RuntimeSupervisorReport::capture(
        &runtime,
        &RuntimeEventRecorder::default(),
    ));

    runtime
        .set_safe_mode(SafeModeRequest { enabled: true })
        .expect("enable safe mode");
    let deferred = runtime
        .render_offline_queue(vec![RuntimeOfflineRenderRequest {
            request_id: "render:queue:performance-trace:0001".into(),
            timeline_start_samples: 0,
            duration_samples: 128,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        }])
        .expect("safe mode should defer offline render queue");
    assert_eq!(
        deferred.orchestration.decision,
        RuntimeDeferredServiceDecision::Defer
    );
    reports.push(RuntimeSupervisorReport::capture(
        &runtime,
        &RuntimeEventRecorder::default(),
    ));

    runtime
        .set_safe_mode(SafeModeRequest { enabled: false })
        .expect("disable safe mode");
    runtime
        .process_engine_block(
            2,
            2,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(12), 20),
        )
        .unwrap();
    reports.push(RuntimeSupervisorReport::capture(
        &runtime,
        &RuntimeEventRecorder::default(),
    ));

    let trace = RuntimeSupervisorReport::build_performance_trace_receipt(&reports);
    let performance_snapshots = reports
        .iter()
        .map(|report| report.performance_snapshot())
        .collect::<Vec<_>>();
    let expected_peak_cpu = reports
        .iter()
        .map(|report| report.performance_snapshot().cpu_load_percent)
        .fold(0.0f32, f32::max);
    let expected_peak_graph_latency = reports
        .iter()
        .map(|report| report.performance_snapshot().graph_latency_ms)
        .fold(0.0f32, f32::max);
    assert_eq!(trace.observation_count, reports.len());
    assert_eq!(trace.first_block_sequence, None);
    assert_eq!(trace.last_block_sequence, Some(2));
    assert_eq!(trace.processed_block_span, 2);
    assert_eq!(trace.peak_cpu_load_percent, expected_peak_cpu);
    assert_eq!(trace.peak_graph_latency_ms, expected_peak_graph_latency);
    assert!(trace.peak_block_execution_time_ns > 0);
    assert!(trace.playback_active_observation_count >= 3);
    assert!(trace.recording_active_observation_count >= 3);
    assert!(trace.background_service_defer_count >= 1);
    assert!(trace.background_service_while_playing_count >= 1);
    assert!(trace.background_service_while_recording_count >= 1);
    assert!(trace.background_starvation_observation_count >= 1);
    assert_eq!(trace.peak_background_starved_work_item_count, 1);
    assert_eq!(trace.background_cancellation_observation_count, 0);
    assert_eq!(trace.peak_background_cancelled_work_item_count, 0);
    assert_eq!(trace.background_realtime_backpressure_observation_count, 0);
    assert!(trace.background_recovery_backpressure_observation_count >= 1);
    assert_eq!(trace.peak_background_queued_work_item_count, 1);
    assert_eq!(trace.peak_background_deferred_work_item_count, 1);
    assert_eq!(trace.peak_hot_latency_node_id.as_deref(), Some("latency"));
    assert_eq!(trace.peak_hot_latency_node_samples, 24);
    let expected_peak_lane = performance_snapshots
        .iter()
        .max_by_key(|snapshot| snapshot.critical_path_lane_total_latency_samples)
        .expect("trace should have at least one performance snapshot");
    assert_eq!(
        trace.peak_hot_latency_group_node_count,
        expected_peak_lane.hot_latency_group_node_count
    );
    assert_eq!(
        trace.peak_critical_path_lane.as_deref(),
        expected_peak_lane.critical_path_lane.as_deref()
    );
    assert_eq!(
        trace.peak_critical_path_lane_node_count,
        expected_peak_lane.critical_path_lane_node_count
    );
    assert_eq!(
        trace.peak_critical_path_lane_plugin_backed_node_count,
        expected_peak_lane.critical_path_lane_plugin_backed_node_count
    );
    assert_eq!(
        trace.peak_critical_path_lane_total_latency_samples,
        expected_peak_lane.critical_path_lane_total_latency_samples
    );
    assert!(trace.summary.contains("recording_active="));
    assert!(trace.summary.contains("deadline="));
    assert!(trace.summary.contains("background="));
    assert!(trace.summary.contains("backpressure="));
    assert!(trace.summary.contains("starvation="));
    assert!(trace.summary.contains("critical_lane="));
    assert!(trace
        .render_json()
        .contains("\"peak_hot_latency_node_id\":\"latency\""));
    assert!(trace.render_json().contains("\"peak_critical_path_lane\":"));
    assert!(trace
        .render_json()
        .contains("\"peak_block_execution_time_ns\":"));
    assert!(trace
        .render_json()
        .contains("\"peak_background_starved_work_item_count\":1"));

    runtime.cancel_recording_capture().unwrap();
    let _ = fs::remove_file(capture_path);
}
