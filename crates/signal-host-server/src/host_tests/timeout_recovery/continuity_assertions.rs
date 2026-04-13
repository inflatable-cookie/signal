use signal_plugin::WatchdogTriggerReason;
use signal_runtime::RuntimeSupervisorReport;

use crate::host::host_support::ServerRuntimeHostSummary;

pub(super) fn assert_timeout_recovery_continuity(
    summary: &ServerRuntimeHostSummary,
    supervisor: &RuntimeSupervisorReport,
) {
    assert_eq!(
        supervisor
            .observation
            .supervision_snapshot
            .watchdog_restart_count,
        1
    );
    assert!(!supervisor
        .observation
        .supervision_snapshot
        .safe_mode_enabled);
    assert_eq!(
        summary.faults.watchdog_trigger_reason,
        Some(WatchdogTriggerReason::DeadlineMisses)
    );
    assert!(summary.transport.shared_memory_lease_id.contains("epoch-2"));

    let transport = &supervisor.observation.transport_concurrency_snapshot;
    assert_eq!(transport.current_attached_sessions, 1);
    assert_eq!(transport.peak_attached_sessions, 2);
    assert_eq!(transport.current_recovery_overlap_sessions, 0);
    assert_eq!(
        transport.last_admitted_sandbox_id.as_deref(),
        Some("server-default-sandbox")
    );
}
