use super::super::host_test_support::{
    assert_runtime_automation_continuity, assert_runtime_automation_values,
    assert_runtime_sequence_continuity, RuntimeAutomationExpectations,
};
use super::super::ServerRuntimeHost;
use signal_plugin::{CompletionState, WatchdogTriggerReason};
use signal_runtime::{
    RecoveryRestartIntent, RuntimeConfig, SignalRuntime, StopReason,
};

#[test]
fn server_host_recovers_after_crash() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);
    let summary = host
        .boot_with_crash_recovery()
        .expect("crash recovery boot");
    let supervisor = host.supervisor_report();

    assert_eq!(summary.execution.processing_epoch, 2);
    assert_eq!(summary.execution.restart_count, 1);
    assert_eq!(summary.execution.teardown_count, 1);
    assert_eq!(
        summary.execution.last_recovery_intent,
        Some(RecoveryRestartIntent::CrashRecovery)
    );
    assert_eq!(
        summary.execution.last_stop_reason,
        Some(StopReason::DegradedModeRecovery)
    );
    assert_eq!(
        summary.execution.last_completion_state,
        CompletionState::Completed
    );
    assert_eq!(summary.execution.processed_blocks, 9);
    assert_eq!(summary.last_payload.event_count, 11);
    assert_eq!(summary.last_payload.parameter_event_count, 2);
    assert_eq!(summary.last_payload.parameter_gesture_event_count, 2);
    assert_eq!(summary.last_payload.parameter_modulation_event_count, 2);
    assert_eq!(summary.last_payload.note_event_count, 1);
    assert_eq!(summary.last_payload.note_expression_event_count, 3);
    assert_eq!(summary.last_payload.midi_event_count, 1);
    assert_eq!(summary.last_payload.first_output_sample, Some(8.0));
    assert_eq!(summary.faults.deadline_misses, 0);
    assert_eq!(summary.faults.heartbeat_misses, 0);
    assert!(!summary.faults.watchdog_triggered);
    assert_eq!(
        supervisor
            .observation
            .supervision_snapshot
            .watchdog_restart_count,
        0
    );
    assert!(
        !supervisor
            .observation
            .supervision_snapshot
            .safe_mode_enabled
    );
    assert!(summary
        .transport
        .shared_memory_region_id
        .starts_with("region-"));
    assert_runtime_automation_values(
        &supervisor,
        RuntimeAutomationExpectations {
            value_events: 9,
            modulation_events: 9,
            gesture_begin_events: 3,
            gesture_end_events: 6,
            first_value: 0.1,
            last_value: 0.5,
            last_modulation: 0.08,
        },
    );
    assert_runtime_automation_continuity(&supervisor, 1, 2, &[1, 2], 1);
    assert_runtime_sequence_continuity(&supervisor, &[1, 2], 0, 8, 0, 1);
}

#[test]
fn server_host_recovers_after_heartbeat_watchdog_trigger() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);
    let summary = host
        .boot_with_heartbeat_miss_recovery()
        .expect("heartbeat recovery boot");
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
    assert_eq!(summary.execution.processed_blocks, 8);
    assert_eq!(summary.execution.last_block_sequence, 9);
    assert_eq!(summary.faults.heartbeat_misses, 2);
    assert_eq!(summary.faults.deadline_misses, 0);
    assert!(summary.faults.watchdog_triggered);
    assert_eq!(
        summary.faults.watchdog_trigger_reason,
        Some(WatchdogTriggerReason::HeartbeatMisses)
    );
    assert_eq!(
        supervisor
            .observation
            .supervision_snapshot
            .watchdog_restart_count,
        1
    );
    assert_eq!(supervisor.observation.control_snapshot.start_count, 2);
    assert_eq!(supervisor.observation.control_snapshot.stop_count, 1);
    assert_eq!(
        supervisor.observation.control_snapshot.last_stop_reason,
        Some(StopReason::DegradedModeRecovery)
    );
    assert!(supervisor.observation.control_snapshot.running);
    assert!(
        !supervisor
            .observation
            .supervision_snapshot
            .safe_mode_enabled
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
    assert_runtime_automation_continuity(&supervisor, 2, 2, &[2], 0);
    assert_runtime_sequence_continuity(&supervisor, &[2], 2, 9, 0, 0);
}

#[test]
fn server_host_enters_safe_mode_after_repeated_watchdog_restarts() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);
    let summary = host
        .boot_with_escalating_heartbeat_failures()
        .expect("escalating heartbeat recovery boot");
    let supervisor = host.supervisor_report();

    assert_eq!(summary.execution.processing_epoch, 3);
    assert_eq!(summary.execution.restart_count, 2);
    assert_eq!(summary.execution.teardown_count, 2);
    assert_eq!(
        summary.execution.last_recovery_intent,
        Some(RecoveryRestartIntent::WatchdogRecovery)
    );
    assert_eq!(
        summary.execution.last_stop_reason,
        Some(StopReason::DegradedModeRecovery)
    );
    assert_eq!(summary.execution.processed_blocks, 10);
    assert_eq!(summary.execution.last_block_sequence, 13);
    assert_eq!(summary.faults.heartbeat_misses, 4);
    assert!(summary.faults.watchdog_triggered);
    assert_eq!(
        supervisor
            .observation
            .supervision_snapshot
            .watchdog_restart_count,
        2
    );
    assert!(
        supervisor
            .observation
            .supervision_snapshot
            .safe_mode_enabled
    );
    assert!(matches!(
        supervisor.observation.readiness,
        signal_runtime::RuntimeReadiness::Degraded { .. }
    ));
    assert_eq!(supervisor.observation.control_snapshot.start_count, 3);
    assert_eq!(supervisor.observation.control_snapshot.stop_count, 2);
    assert_eq!(
        supervisor.observation.control_snapshot.last_stop_reason,
        Some(StopReason::DegradedModeRecovery)
    );
    assert_runtime_automation_values(
        &supervisor,
        RuntimeAutomationExpectations {
            value_events: 10,
            modulation_events: 10,
            gesture_begin_events: 2,
            gesture_end_events: 8,
            first_value: 0.2,
            last_value: 0.75,
            last_modulation: 0.18,
        },
    );
    assert_runtime_automation_continuity(&supervisor, 2, 3, &[2, 3], 1);
    assert_runtime_sequence_continuity(&supervisor, &[2, 3], 2, 13, 0, 1);
}
