use super::*;

#[test]
fn handshake_requires_client_version() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let error = runtime
        .handshake(HandshakeRequest {
            client_version: String::new(),
            anticipative_preferred: true,
            max_sample_rate_hint: None,
        })
        .unwrap_err();

    assert_eq!(
        error.kind,
        crate::interfaces::RuntimeErrorKind::InvalidRequest
    );
}

#[test]
fn schedule_projection_advances_epoch() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let receipt = runtime
        .apply_schedule_projection(ScheduleProjection {
            schedule_id: "sched-1".into(),
            stream_count: 2,
        })
        .unwrap();

    assert_eq!(receipt.accepted_epoch, 1);
    assert!(receipt.applied_at_block_boundary);
}

#[test]
fn schedule_projection_refreshes_running_prework_window_with_widened_scope() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    runtime
        .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
            target_window_blocks: 8,
            prepare_budget_per_cycle: 1,
            buffer_seed_offset: 0,
            transport_playing: true,
            transport_tempo_bpm: 126.0,
            transport_loop_length_blocks: 16,
            parameter_target: "engine.local.drive".into(),
            parameter_cycle_length: 8,
        })
        .expect("set widened refresh policy");
    apply_latency_runtime_graph(&mut runtime, "graph:runtime:schedule-refresh");
    runtime.start().expect("start runtime");

    let before = runtime.get_engine_block_snapshot();
    assert_eq!(before.prework_cache_queue_depth, 2);
    assert!(before.prework_pending_target_count > 0);

    runtime
        .apply_schedule_projection(ScheduleProjection {
            schedule_id: "sched:runtime:refresh-widened".into(),
            stream_count: 3,
        })
        .expect("apply widened schedule projection");

    let snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(snapshot.scheduler_topology.schedule_stream_count, Some(3));
    assert!(snapshot.scheduler_topology.compatible);
    assert_eq!(snapshot.last_prework_service_requested_cycles, 3);
    assert_eq!(snapshot.last_prework_service_effective_cycles, 3);
    assert_eq!(snapshot.last_prework_service_cycle_count, 3);
    assert_eq!(snapshot.last_prework_service_budget_per_cycle, Some(1));
    assert_eq!(
        snapshot.last_prework_service_effective_budget_per_cycle,
        Some(3)
    );
    assert!(snapshot.prework_cache_queue_depth > before.prework_cache_queue_depth);
    assert_eq!(snapshot.prework_pending_target_count, 0);
}

#[test]
fn runtime_scheduler_topology_summary_validates_track_bus_console_groups() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    install_scheduler_topology_runtime_graph(
        &mut runtime,
        "graph:runtime:scheduler-topology",
        &["track:drums", "track:bass"],
        false,
    );

    let missing_schedule = runtime.get_engine_block_snapshot();
    let scheduler_topology = runtime.get_scheduler_topology_summary();
    assert_eq!(missing_schedule.scheduler_topology.track_lane_node_count, 2);
    assert_eq!(scheduler_topology.track_lane_node_count, 2);
    assert_eq!(
        missing_schedule.scheduler_topology.track_lane_group_count,
        2
    );
    assert_eq!(missing_schedule.scheduler_topology.bus_node_count, 1);
    assert_eq!(missing_schedule.scheduler_topology.bus_group_count, 2);
    assert_eq!(missing_schedule.scheduler_topology.console_node_count, 1);
    assert_eq!(missing_schedule.scheduler_topology.console_group_count, 1);
    assert_eq!(
        missing_schedule.scheduler_topology.schedule_stream_count,
        None
    );
    assert!(!missing_schedule.scheduler_topology.compatible);
    assert!(
        missing_schedule
            .scheduler_topology
            .requires_host_reinterpretation
    );
    assert!(matches!(
        missing_schedule.scheduler_topology.issues.as_slice(),
        [
            RuntimeSchedulerTopologyIssue::MissingScheduleProjectionForTrackLanes {
                required_streams: 2
            }
        ]
    ));

    runtime
        .apply_schedule_projection(ScheduleProjection {
            schedule_id: "sched-topology".into(),
            stream_count: 2,
        })
        .expect("apply matching schedule projection");

    let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 1);
    let result = runtime
        .process_engine_block(1, 1, block)
        .expect("process topology-aware block");
    let execution_topology = runtime.get_execution_topology_summary();

    assert_eq!(result.snapshot.lane_order.len(), 2);
    assert_eq!(
        result.snapshot.lane_order,
        vec![
            signal_graph::GraphExecutionLane::Anticipative,
            signal_graph::GraphExecutionLane::Realtime,
        ]
    );
    assert_eq!(
        result.snapshot.dispatch_order.last().copied(),
        Some(signal_graph::GraphExecutionLane::Realtime)
    );
    assert!(result.snapshot.scheduler_topology.compatible);
    assert!(
        !result
            .snapshot
            .scheduler_topology
            .requires_host_reinterpretation
    );
    assert!(result.snapshot.scheduler_topology.issues.is_empty());
    assert_eq!(
        result.snapshot.scheduler_topology.schedule_stream_count,
        Some(2)
    );
    assert_eq!(execution_topology.node_count, result.snapshot.node_count);
    assert_eq!(execution_topology.track_lane_group_count, 2);
    assert_eq!(execution_topology.bus_group_count, 2);
    assert_eq!(execution_topology.console_group_count, 1);
}

#[test]
fn runtime_scheduler_topology_summary_flags_insufficient_schedule_streams() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    install_scheduler_topology_runtime_graph(
        &mut runtime,
        "graph:runtime:scheduler-topology-insufficient",
        &["track:drums", "track:bass"],
        false,
    );
    runtime
        .apply_schedule_projection(ScheduleProjection {
            schedule_id: "sched-too-small".into(),
            stream_count: 1,
        })
        .expect("apply undersized schedule projection");

    let snapshot = runtime.get_engine_block_snapshot();
    assert!(!snapshot.scheduler_topology.compatible);
    assert!(snapshot.scheduler_topology.requires_host_reinterpretation);
    assert!(snapshot.scheduler_topology.issues.iter().any(|issue| {
        matches!(
            issue,
            RuntimeSchedulerTopologyIssue::InsufficientScheduleStreams {
                required_streams: 2,
                actual_streams: 1
            }
        )
    }));
}

#[test]
fn runtime_scheduler_topology_summary_flags_missing_track_lane_metadata() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    install_scheduler_topology_runtime_graph(
        &mut runtime,
        "graph:runtime:scheduler-topology-missing-metadata",
        &["track:drums"],
        true,
    );
    runtime
        .apply_schedule_projection(ScheduleProjection {
            schedule_id: "sched-metadata".into(),
            stream_count: 2,
        })
        .expect("apply schedule projection");

    let snapshot = runtime.get_engine_block_snapshot();
    assert!(!snapshot.scheduler_topology.compatible);
    assert!(snapshot.scheduler_topology.requires_host_reinterpretation);
    assert!(snapshot.scheduler_topology.issues.iter().any(|issue| {
        matches!(
            issue,
            RuntimeSchedulerTopologyIssue::MissingTrackLaneIds { node_count: 1 }
        )
    }));
}

#[test]
fn runtime_scheduler_topology_projects_into_runtime_reports() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    install_scheduler_topology_runtime_graph(
        &mut runtime,
        "graph:runtime:scheduler-topology-report",
        &["track:drums", "track:bass"],
        false,
    );
    runtime
        .apply_schedule_projection(ScheduleProjection {
            schedule_id: "sched-topology-report".into(),
            stream_count: 2,
        })
        .expect("apply matching schedule projection");

    let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 1);
    runtime
        .process_engine_block(1, 1, block)
        .expect("process topology report block");

    let metering = runtime.get_metering_snapshot();
    assert!(metering.meter_count > 0);
    assert!(metering.main_output_peak_level.is_some());
    assert!(metering.main_output_rms_level.is_some());
    assert!(metering
        .meters
        .iter()
        .any(|meter| meter.bus_id == "main:out"));
    assert_eq!(metering.track_lanes.len(), 2);
    assert_eq!(metering.bus_groups.len(), 2);
    assert_eq!(metering.console_groups.len(), 1);
    assert!(metering.send_returns.is_empty());
    assert!(metering
        .track_lanes
        .iter()
        .any(|track_lane| track_lane.track_lane_id == "track:drums"));
    assert!(metering
        .bus_groups
        .iter()
        .any(|bus_group| bus_group.bus_group_id == "mix:master"));
    assert!(metering.console_groups.iter().any(|console_group| {
        console_group.console_group_id == "console:main" && console_group.aggregate.meter_count > 0
    }));

    let diagnostics = runtime.get_diagnostics_snapshot();
    assert!(diagnostics.topology_compatible);
    assert_eq!(
        diagnostics.last_output_peak,
        metering.main_output_peak_level
    );
    assert_eq!(diagnostics.last_output_rms, metering.main_output_rms_level);
    assert_eq!(
        diagnostics.momentary_loudness_lufs,
        metering.momentary_loudness_lufs
    );
    assert_eq!(
        diagnostics.integrated_loudness_lufs,
        metering.integrated_loudness_lufs
    );

    let observation = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
    assert_eq!(observation.execution_topology_summary.node_count, 5);
    assert_eq!(
        observation.execution_topology_summary.track_lane_node_count,
        2
    );
    assert_eq!(observation.execution_topology_summary.bus_node_count, 1);
    assert_eq!(observation.execution_topology_summary.console_node_count, 1);
    assert_eq!(
        observation
            .execution_topology_summary
            .track_lane_group_count,
        2
    );
    assert_eq!(observation.execution_topology_summary.bus_group_count, 2);
    assert_eq!(
        observation.execution_topology_summary.console_group_count,
        1
    );
    assert_eq!(observation.execution_topology_summary.track_lanes.len(), 2);
    assert_eq!(observation.execution_topology_summary.bus_groups.len(), 2);
    assert_eq!(
        observation.execution_topology_summary.console_groups.len(),
        1
    );
    assert_eq!(observation.execution_topology_summary.lanes.len(), 2);
    assert_eq!(observation.metering_snapshot.track_lanes.len(), 2);
    assert_eq!(observation.metering_snapshot.bus_groups.len(), 2);
    assert_eq!(observation.metering_snapshot.console_groups.len(), 1);
    assert!(observation.metering_snapshot.send_returns.is_empty());
    assert!(observation
        .render_compact()
        .contains("engine_scheduler_topology_compatible=true"));
    assert!(observation
        .render_compact()
        .contains("engine_scheduler_topology_track_lanes=2/2"));
    assert!(observation
        .render_compact()
        .contains("execution_topology_summary_roles=1/2/1/0/1"));
    assert!(observation
        .render_compact()
        .contains("execution_topology_summary_lane_shapes=Anticipative:1|Realtime:4"));
    assert!(observation
        .render_compact()
        .contains("metering_snapshot_routes=2/2/0/1"));

    let supervisor = RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
    assert!(supervisor
        .render_multiline()
        .contains("engine_scheduler_topology_buses=1/2"));
    assert!(supervisor
        .render_multiline()
        .contains("engine_scheduler_topology_consoles=1/1"));
    assert!(supervisor
        .render_multiline()
        .contains("engine_scheduler_topology_issue_count=0"));
    assert!(supervisor
        .render_multiline()
        .contains("execution_topology_summary_lane_0=Anticipative"));
    assert!(supervisor
        .render_multiline()
        .contains("execution_topology_summary_lane_1=Realtime"));
    assert!(supervisor
        .render_multiline()
        .contains("metering_snapshot_meter_count="));
    assert!(supervisor
        .render_multiline()
        .contains("metering_snapshot_track_lane_count=2"));
    assert!(supervisor
        .render_multiline()
        .contains("metering_snapshot_console_group_0=console:main"));
    assert!(supervisor
            .render_multiline()
            .contains("execution_topology_summary_node_2=track-1/Realtime/StatefulRealtime/TrackLane/track_lane_id=Some(\"track:bass\")"));
    assert!(supervisor.render_multiline().contains(
        "execution_topology_summary_node_4=console-main/Realtime/InlineRealtime/ConsoleNode"
    ));

    let json = supervisor.render_json();
    assert!(json.contains("\"scheduler_topology\":{\"track_lane_node_count\":2"));
    assert!(json.contains("\"track_lane_group_count\":2"));
    assert!(json.contains("\"schedule_stream_count\":2"));
    assert!(json.contains("\"compatible\":true"));
    assert!(json.contains("\"metering_snapshot\":{\"meter_count\":"));
    assert!(json.contains("\"track_lanes\":["));
    assert!(json.contains("\"console_groups\":["));
    assert!(json.contains("\"execution_topology_summary\":{\"node_count\":5"));
    assert!(json.contains("\"track_lane_node_count\":2"));
    assert!(json.contains("\"lane\":\"Anticipative\""));
    assert!(json.contains("\"lane\":\"Realtime\""));
    assert!(json.contains("\"node_id\":\"track-0\""));
    assert!(json.contains("\"track_lane_id\":\"track:drums\""));
    assert!(json.contains("\"bus_group_id\":\"mix:master\""));
    assert!(json.contains("\"console_group_id\":\"console:main\""));
    assert!(json.contains("\"track_lanes\":["));
    assert!(json.contains("\"bus_groups\":["));
    assert!(json.contains("\"console_groups\":["));
    assert!(json.contains("\"node_id\":\"console-main\""));
    assert!(json.contains("\"output_bus_id\":\"main:out\""));
}
