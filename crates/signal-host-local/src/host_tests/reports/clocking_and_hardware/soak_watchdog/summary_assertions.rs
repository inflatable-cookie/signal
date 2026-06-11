use super::super::super::super::*;
use crate::LocalRuntimeHostSummary;
use signal_runtime::RuntimeSupervisorReport;

pub(super) fn assert_mixed_watchdog_summary(
    host: &LocalRuntimeHost,
    summary: &LocalRuntimeHostSummary,
    supervisor: &RuntimeSupervisorReport,
) {
    assert_eq!(summary.execution.processing_epoch, 4);
    assert_eq!(summary.execution.restart_count, 3);
    assert_eq!(summary.execution.teardown_count, 3);
    assert_eq!(
        summary.execution.last_recovery_intent,
        Some(RecoveryRestartIntent::WatchdogRecovery)
    );
    assert_eq!(
        summary.execution.last_stop_reason,
        Some(StopReason::DegradedModeRecovery)
    );
    assert_eq!(summary.execution.processed_blocks, 14);
    assert_eq!(
        summary.execution.last_block_sequence, 13,
        "unexpected mixed watchdog soak summary: {summary:?}"
    );
    assert_eq!(summary.faults.deadline_misses, 2);
    assert_eq!(summary.faults.heartbeat_misses, 4);
    assert_eq!(
        supervisor.observation.supervision_snapshot.watchdog_restart_count,
        3
    );
    assert!(supervisor.observation.supervision_snapshot.safe_mode_enabled);
    assert_eq!(supervisor.observation.control_snapshot.start_count, 4);
    assert_eq!(supervisor.observation.control_snapshot.stop_count, 3);
    assert_eq!(
        supervisor.observation.control_snapshot.last_stop_reason,
        Some(StopReason::DegradedModeRecovery)
    );
    assert_runtime_automation_values(
        supervisor,
        RuntimeAutomationExpectations {
            value_events: 14,
            modulation_events: 14,
            gesture_begin_events: 3,
            gesture_end_events: 11,
            first_value: 2.0 / 7.0,
            last_value: 5.0 / 7.0,
            last_modulation: 0.18,
        },
    );
    assert_runtime_automation_continuity(supervisor, 2, 4, &[2, 3, 4], 2);
    assert_runtime_sequence_continuity(supervisor, &[2, 2, 3, 4], 2, 13, 1, 2);
    assert_plugin_dispatch_summary(summary, supervisor, 2);
    assert!(supervisor.event_count() > 24);
    assert_eq!(supervisor.supervision_update_count(), 3);
    assert_eq!(supervisor.plugin_fault_count(), 3);
    assert_eq!(
        supervisor
            .observation
            .observation
            .fault_detail_count_containing("heartbeat watchdog"),
        2
    );
    assert_eq!(
        supervisor
            .observation
            .observation
            .fault_detail_count_containing("block deadline"),
        1
    );
    assert_eq!(
        host.runtime().get_supervision_snapshot().last_watchdog_trigger,
        Some(signal_runtime::RuntimeWatchdogTrigger::HeartbeatMisses)
    );
    assert_eq!(
        supervisor.last_watchdog_trigger(),
        Some(signal_runtime::RuntimeWatchdogTrigger::HeartbeatMisses)
    );
    assert!(summary.transport.shared_memory_lease_id.contains("epoch-4"));
}
