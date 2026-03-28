use super::super::super::super::host_test_support::{
    assert_runtime_automation_continuity, assert_runtime_automation_values,
    assert_runtime_sequence_continuity, RuntimeAutomationExpectations,
};
use super::super::super::super::host_support::ServerRuntimeHostSummary;
use super::super::super::super::ServerRuntimeHost;
use signal_runtime::{
    RecoveryRestartIntent, RuntimeHostSupervisorReport, RuntimeObservationApi, StopReason,
};

pub(super) fn assert_mixed_watchdog_summary(
    host: &ServerRuntimeHost,
    summary: &ServerRuntimeHostSummary,
    supervisor: &RuntimeHostSupervisorReport,
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
    assert_eq!(summary.execution.last_block_sequence, 17);
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
            gesture_begin_events: 2,
            gesture_end_events: 12,
            first_value: 0.2,
            last_value: 0.95,
            last_modulation: 0.26,
        },
    );
    assert_runtime_automation_continuity(supervisor, 2, 4, &[2, 3, 4], 2);
    assert_runtime_sequence_continuity(supervisor, &[2, 3, 4], 2, 17, 0, 2);
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
    let rendered = supervisor.render_compact();
    assert!(rendered.contains("readiness=Degraded"));
    assert!(rendered.contains("supervision_updates=3"));
    assert!(rendered.contains("plugin_faults=3"));
    assert!(rendered.contains("last_watchdog=HeartbeatMisses"));
    assert!(rendered.contains(&format!("event_stream={}", supervisor.event_count())));
}
