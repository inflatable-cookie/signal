mod continuity_assertions;
mod execution_assertions;

use super::super::host_test_support::{
    assert_runtime_automation_continuity, assert_runtime_automation_values,
    assert_runtime_plugin_event_snapshot, assert_runtime_sequence_continuity,
    RuntimeAutomationExpectations,
};
use super::super::ServerRuntimeHost;
use continuity_assertions::assert_timeout_recovery_continuity;
use execution_assertions::assert_timeout_recovery_execution;
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

    assert_timeout_recovery_execution(&summary, &supervisor);
    assert_timeout_recovery_continuity(&summary, &supervisor);

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
