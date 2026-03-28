use super::super::host_test_support::{
    assert_runtime_automation_continuity, assert_runtime_automation_values,
    assert_runtime_plugin_event_snapshot, assert_runtime_sequence_continuity,
    RuntimeAutomationExpectations,
};
use super::super::ServerRuntimeHost;
use signal_plugin::{CompletionState, WatchdogTriggerReason};
use signal_runtime::{
    RecoveryRestartIntent, RuntimeConfig, SignalRuntime, StopReason,
};

#[test]
fn server_host_rolls_leases_forward_after_timeout() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);
    let summary = host
        .boot_with_timeout_recovery()
        .expect("timeout recovery boot");
    let supervisor = host.supervisor_report();

    assert_eq!(summary.execution.processing_epoch, 2);
    assert_eq!(summary.execution.restart_count, 1);
    assert_eq!(summary.execution.teardown_count, 1);
    assert_eq!(
        summary.execution.last_recovery_intent,
        Some(RecoveryRestartIntent::WatchdogRecovery)
    );
    assert_eq!(
        summary.execution.last_stop_reason,
        Some(StopReason::DegradedModeRecovery)
    );
    assert_eq!(
        summary.execution.last_completion_state,
        CompletionState::Completed
    );
    assert_eq!(summary.execution.processed_blocks, 10);
    assert_eq!(summary.execution.engine_processed_blocks, 10);
    assert_eq!(summary.execution.last_block_sequence, 9);
    assert_eq!(
        summary.execution.last_engine_graph_id.as_deref(),
        Some("signal.host.server.demo")
    );
    let plugin_state = summary
        .execution
        .last_plugin_state
        .as_ref()
        .expect("plugin instance state should be projected into server summary");
    assert_eq!(plugin_state.plugin_type_id, "plugin:clap:server");
    assert_eq!(plugin_state.instance_id, "instance:server:default");
    assert_eq!(plugin_state.lifecycle_state, "Active");
    assert_eq!(plugin_state.readiness_state, "Ready");
    assert!(plugin_state.active);
    assert_eq!(plugin_state.processing_sample_rate_hz, Some(48_000));
    assert_eq!(plugin_state.processing_max_block_frames, Some(512));
    assert!(plugin_state.last_fault.is_none());
    let observed_plugin_state = supervisor
        .observation
        .observation
        .last_plugin_instance_state()
        .expect("runtime observation should retain typed plugin state");
    assert_eq!(observed_plugin_state.instance_id, "instance:server:default");
    assert_eq!(observed_plugin_state.lifecycle_state, "Active");
    assert_eq!(observed_plugin_state.readiness_state, "Ready");
    assert!(supervisor
        .render_json()
        .contains("\"plugin_instance_state_events\":"));
    assert!(
        summary
            .execution
            .last_engine_output_peak
            .unwrap_or_default()
            <= 0.7
    );
    assert!(summary.execution.last_engine_output_rms.unwrap_or_default() > 0.0);
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .last_execution_context
            .as_ref()
            .map(|context| context.projection_epoch),
        Some(1)
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .last_execution_context
            .as_ref()
            .map(|context| context.transport_playing),
        Some(true)
    );
    assert!(
        supervisor
            .observation
            .engine_block_snapshot
            .last_execution_context
            .as_ref()
            .map(|context| context.timeline_position_samples)
            .unwrap_or_default()
            > 0
    );
    assert_eq!(supervisor.observation.engine_block_snapshot.node_count, 3);
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .stateful_node_count,
        2
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .latency_node_count,
        1
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .plugin_backed_node_count,
        1
    );
    assert!(
        !supervisor
            .observation
            .engine_block_snapshot
            .anticipative_planning_enabled
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .inline_realtime_node_count,
        1
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .stateful_realtime_node_count,
        2
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .anticipative_eligible_node_count,
        0
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_service_semantic_policy,
        signal_runtime::RuntimePreworkServiceSemanticPolicy::Balanced
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_service_active_plugin_sandboxes,
        1
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_service_bound_plugin_sandboxes,
        1
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_service_active_bound_plugin_sandboxes,
        1
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_service_degraded_bound_plugin_sandboxes,
        0
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_service_missing_bound_plugin_sandboxes,
        0
    );
    assert!(supervisor
        .observation
        .engine_block_snapshot
        .planned_nodes
        .iter()
        .any(|node| node.node_id == "drive"
            && node.plugin_sandbox_id.as_deref() == Some("server-default-sandbox")));
    assert_eq!(supervisor.observation.engine_block_snapshot.phase_count, 2);
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .anticipative_phase_count,
        0
    );
    assert_eq!(supervisor.observation.engine_block_snapshot.lane_count, 1);
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .anticipative_lane_count,
        0
    );
    assert_eq!(
        supervisor.observation.engine_block_snapshot.dispatch_count,
        1
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .dispatch_boundary_count,
        0
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prepared_dispatch_count,
        0
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .realtime_dispatch_count,
        1
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .dispatch_handoff_count,
        0
    );
    assert!(
        !supervisor
            .observation
            .engine_block_snapshot
            .prework_cache_enabled
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_forecast_requested_mode,
        signal_runtime::RuntimePreworkForecastMode::RuntimeRoleDefault
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_forecast_mode,
        signal_runtime::RuntimePreworkForecastMode::Disabled
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_cache_state,
        signal_runtime::RuntimePreworkCacheState::Disabled
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_cache_freshness_state,
        signal_runtime::RuntimePreworkFreshnessState::Disabled
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_cache_admissions,
        0
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_cache_consumptions,
        0
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_cache_retirement_count,
        0
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_cache_hits,
        0
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_cache_misses,
        0
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .last_prework_output_peak,
        None
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .last_prework_admission_processing_epoch,
        None
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .last_prework_admission_block_sequence,
        None
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .last_prework_consumption_processing_epoch,
        None
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .last_prework_consumption_block_sequence,
        None
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .last_prework_retirement_reason,
        None
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .last_prework_retired_unconsumed,
        None
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_cache_valid_until_block_sequence,
        None
    );
    assert!(supervisor
        .observation
        .engine_block_snapshot
        .last_realtime_input_peak
        .is_some());
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .total_latency_samples,
        32
    );
    assert_eq!(summary.last_payload.event_count, 11);
    assert_eq!(summary.last_payload.parameter_event_count, 2);
    assert_eq!(summary.last_payload.parameter_gesture_event_count, 2);
    assert_eq!(summary.last_payload.parameter_modulation_event_count, 2);
    assert_eq!(summary.last_payload.note_event_count, 1);
    assert_eq!(summary.last_payload.note_expression_event_count, 3);
    assert_eq!(summary.last_payload.midi_event_count, 1);
    assert_eq!(summary.last_payload.generated_event_bytes, 268);
    assert_eq!(summary.last_payload.first_output_sample, Some(9.0));
    assert_eq!(summary.faults.deadline_misses, 2);
    assert_eq!(summary.faults.heartbeat_misses, 0);
    assert!(summary.faults.watchdog_triggered);
    assert_eq!(
        summary.faults.watchdog_trigger_reason,
        Some(WatchdogTriggerReason::DeadlineMisses)
    );
    assert_eq!(
        supervisor
            .observation
            .supervision_snapshot
            .watchdog_restart_count,
        1
    );
    assert!(
        !supervisor
            .observation
            .supervision_snapshot
            .safe_mode_enabled
    );
    assert!(summary.transport.shared_memory_lease_id.contains("epoch-2"));
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .current_attached_sessions,
        1
    );
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .peak_attached_sessions,
        2
    );
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .current_recovery_overlap_sessions,
        1
    );
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .last_admitted_sandbox_id
            .as_deref(),
        Some("server-default-sandbox")
    );
    assert_runtime_automation_values(
        &supervisor,
        RuntimeAutomationExpectations {
            value_events: 8,
            modulation_events: 8,
            gesture_begin_events: 2,
            gesture_end_events: 6,
            first_value: 0.2,
            last_value: 0.55,
            last_modulation: 0.10,
        },
    );
    assert_runtime_automation_continuity(&supervisor, 1, 2, &[1, 2], 1);
    assert_runtime_plugin_event_snapshot(&supervisor, 2, 2, &[2], 0);
    assert_runtime_sequence_continuity(&supervisor, &[1, 2], 0, 9, 0, 1);
}
