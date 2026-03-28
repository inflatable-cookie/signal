use super::super::*;

#[test]
fn local_host_rolls_leases_forward_after_timeout() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
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
    assert_eq!(
        summary.execution.last_block_sequence,
        supervisor
            .observation
            .timeline_snapshot
            .block_sequence_continuity
            .last_block_sequence()
            .expect("last block sequence")
    );
    assert_eq!(
        summary.execution.last_engine_graph_id.as_deref(),
        Some("signal.host.local.demo")
    );
    assert!(
        summary
            .execution
            .last_engine_output_peak
            .unwrap_or_default()
            <= 0.8
    );
    assert!(summary.execution.last_engine_output_rms.is_some());
    assert!(summary.audio_pump.last_callback_output_peak.is_some());
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .last_execution_context
            .as_ref()
            .map(|context| context.projection_epoch),
        Some(2)
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
    assert_eq!(supervisor.observation.engine_block_snapshot.node_count, 4);
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .stateful_node_count,
        4
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
        supervisor
            .observation
            .engine_block_snapshot
            .anticipative_planning_enabled
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .inline_realtime_node_count,
        0
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .stateful_realtime_node_count,
        3
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .anticipative_eligible_node_count,
        1
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_service_semantic_policy,
        signal_runtime::RuntimePreworkServiceSemanticPolicy::PluginConstrained
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
    assert!(
        !supervisor
            .observation
            .engine_block_snapshot
            .prework_service_plugin_gate_active
    );
    assert!(supervisor
        .observation
        .engine_block_snapshot
        .planned_nodes
        .iter()
        .any(|node| node.node_id == "plugin-insert"
            && node.plugin_sandbox_id.as_deref() == Some("local-default-sandbox")));
    assert_eq!(supervisor.observation.engine_block_snapshot.phase_count, 2);
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .anticipative_phase_count,
        1
    );
    assert_eq!(supervisor.observation.engine_block_snapshot.lane_count, 2);
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .anticipative_lane_count,
        1
    );
    assert_eq!(
        supervisor.observation.engine_block_snapshot.dispatch_count,
        2
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .dispatch_boundary_count,
        1
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prepared_dispatch_count,
        1
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
        1
    );
    assert!(
        supervisor
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
        signal_runtime::RuntimePreworkForecastMode::RuntimeRoleDefault
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_forecast_profile,
        Some(signal_runtime::RuntimePreworkForecastProfile::Local)
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_forecast_profile_source,
        Some(signal_runtime::RuntimePreworkForecastProfileSource::RuntimeRoleDefault)
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_forecast_policy_target_window_blocks,
        Some(2)
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_service_pressure,
        signal_runtime::RuntimePreworkServicePressure::Elevated
    );
    assert!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_service_yield_count
            >= 1
    );
    assert!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_service_throttle_count
            >= 1
    );
    assert!(matches!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_cache_state,
        signal_runtime::RuntimePreworkCacheState::Consumed
            | signal_runtime::RuntimePreworkCacheState::Admitted
    ));
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_cache_freshness_state,
        signal_runtime::RuntimePreworkFreshnessState::Fresh
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_cache_queue_capacity,
        3
    );
    assert!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_cache_queue_depth
            > 0
    );
    assert!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_cache_queue_depth
            <= 3
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_cache_peak_queue_depth,
        3
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_cache_window_target_count,
        3
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_cache_window_target_block_sequences,
        vec![
            summary.execution.last_block_sequence,
            summary.execution.last_block_sequence + 1,
            summary.execution.last_block_sequence + 2,
        ]
    );
    let engine_snapshot = &supervisor.observation.engine_block_snapshot;
    assert!(engine_snapshot.prework_cache_admissions >= engine_snapshot.prework_cache_consumptions);
    assert!(
        engine_snapshot.prework_cache_queued_admissions
            >= engine_snapshot.prework_cache_window_target_count as u64
    );
    assert!(
        engine_snapshot.prework_cache_queued_consumptions
            <= engine_snapshot.prework_cache_consumptions
    );
    assert_eq!(
        engine_snapshot.prework_cache_retirement_count,
        engine_snapshot.prework_cache_unconsumed_retirement_count
            + engine_snapshot.prework_cache_consumed_retirement_count
    );
    assert!(engine_snapshot.prework_cache_retirement_count > 0);
    assert_eq!(
        engine_snapshot.prework_cache_hits + engine_snapshot.prework_cache_misses,
        engine_snapshot.prework_cache_consumptions
    );
    assert!(supervisor
        .observation
        .engine_block_snapshot
        .last_prework_output_peak
        .is_some());
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .last_prework_output_peak,
        supervisor
            .observation
            .engine_block_snapshot
            .last_realtime_input_peak
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .last_prework_admission_processing_epoch,
        Some(2)
    );
    assert!(supervisor
        .observation
        .engine_block_snapshot
        .last_prework_admission_block_sequence
        .is_some_and(|sequence| sequence >= summary.execution.last_block_sequence));
    assert!(supervisor
        .observation
        .engine_block_snapshot
        .last_prework_admitted_from_block_sequence
        .is_some_and(|sequence| sequence <= summary.execution.last_block_sequence));
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .last_prework_consumption_processing_epoch,
        Some(2)
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .last_prework_consumption_block_sequence,
        Some(summary.execution.last_block_sequence)
    );
    assert!(supervisor
        .observation
        .engine_block_snapshot
        .last_prework_consumed_from_block_sequence
        .is_some_and(|sequence| sequence <= summary.execution.last_block_sequence));
    assert!(
        matches!(
            supervisor
                .observation
                .engine_block_snapshot
                .last_prework_retirement_reason,
            Some(signal_runtime::RuntimePreworkRetirementReason::PlanningWindowRevised)
                | Some(signal_runtime::RuntimePreworkRetirementReason::TransportStarted)
                | Some(signal_runtime::RuntimePreworkRetirementReason::TransportStopped)
                | Some(signal_runtime::RuntimePreworkRetirementReason::TransportSeeked)
                | Some(signal_runtime::RuntimePreworkRetirementReason::TransportTempoChanged)
                | Some(signal_runtime::RuntimePreworkRetirementReason::TransportLoopStateChanged)
                | Some(signal_runtime::RuntimePreworkRetirementReason::TransportLoopWrapped)
                | Some(signal_runtime::RuntimePreworkRetirementReason::ParameterBatchApplied)
                | Some(signal_runtime::RuntimePreworkRetirementReason::InputSignatureChanged)
                | Some(signal_runtime::RuntimePreworkRetirementReason::ProcessingEpochExpired)
                | Some(signal_runtime::RuntimePreworkRetirementReason::BlockSequenceExpired)
                | Some(signal_runtime::RuntimePreworkRetirementReason::SupersededByAdmission)
                | Some(signal_runtime::RuntimePreworkRetirementReason::QueueCapacityExceeded)
        ),
        "unexpected prework retirement reason: {:?}",
        supervisor
            .observation
            .engine_block_snapshot
            .last_prework_retirement_reason
    );
    assert!(supervisor
        .observation
        .engine_block_snapshot
        .last_prework_retired_unconsumed
        .is_some());
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .prework_cache_valid_until_processing_epoch,
        Some(3)
    );
    assert!(supervisor
        .observation
        .engine_block_snapshot
        .prework_cache_valid_until_block_sequence
        .is_some_and(|sequence| sequence >= summary.execution.last_block_sequence));
    assert!(supervisor
        .observation
        .engine_block_snapshot
        .prework_cache_remaining_valid_blocks
        .is_some_and(|remaining| remaining > 0));
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .total_latency_samples,
        24
    );
    assert_eq!(summary.last_payload.event_count, 11);
    assert_eq!(summary.last_payload.parameter_event_count, 2);
    assert_eq!(summary.last_payload.parameter_gesture_event_count, 2);
    assert_eq!(summary.last_payload.parameter_modulation_event_count, 2);
    assert_eq!(summary.last_payload.note_event_count, 1);
    assert_eq!(summary.last_payload.note_expression_event_count, 3);
    assert_eq!(summary.last_payload.midi_event_count, 1);
    assert_eq!(summary.last_payload.generated_event_bytes, 268);
    assert_eq!(
        summary.last_payload.first_output_sample,
        Some(summary.execution.last_block_sequence as f32)
    );
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
    assert!(summary
        .transport
        .shared_memory_region_id
        .starts_with("region-"));
    assert!(
        summary
            .transport
            .shared_memory_path
            .ends_with(".signal-shm")
    );
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
        0
    );
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .last_admitted_sandbox_id
            .as_deref(),
        Some("local-default-sandbox")
    );
    let automation = &supervisor.observation.automation_snapshot;
    assert_eq!(automation.parameter_id, 4096);
    assert_eq!(automation.value_events, 8);
    assert_eq!(automation.modulation_events, 8);
    assert_eq!(automation.gesture_begin_events, 2);
    assert_eq!(automation.gesture_end_events, 6);
    assert!(automation.first_value.is_some());
    assert!(automation.last_value.is_some(), "{automation:?}");
    assert!(automation.last_modulation.is_some());
    assert_runtime_automation_continuity(&supervisor, 1, 2, &[1, 2], 1);
    assert_runtime_plugin_event_snapshot(&supervisor, 2, 2, &[2], 0);
    let timeline = &supervisor
        .observation
        .timeline_snapshot
        .block_sequence_continuity;
    assert!(timeline.segment_count() >= 2);
    assert!(timeline.first_block_sequence().is_some());
    assert!(timeline
        .last_block_sequence()
        .is_some_and(|last| last >= summary.execution.last_block_sequence));
    assert!(timeline.sequence_gaps <= 1, "{timeline:?}");
    assert_eq!(timeline.lease_rollovers, 1);
    assert_local_plugin_topology(&summary);
    assert_plugin_dispatch_summary(&summary, &supervisor, 2);
}
