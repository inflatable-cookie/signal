use super::super::super::super::super::*;

pub(super) fn assert_mixed_watchdog_lifecycle_faults(
    _summary: &LocalRuntimeHostSummary,
    supervisor: &RuntimeHostSupervisorReport,
) {
    assert_eq!(supervisor.recovery_event_count(), 3);
    assert_eq!(
        supervisor
            .events
            .iter()
            .filter(|event| matches!(
                event,
                signal_runtime::RuntimeEvent::RecoveryCycle {
                    intent: RecoveryRestartIntent::WatchdogRecovery,
                    stop_reason: StopReason::DegradedModeRecovery,
                    ..
                }
            ))
            .count(),
        3
    );
    assert_eq!(
        supervisor
            .events
            .iter()
            .filter(|event| matches!(
                event,
                signal_runtime::RuntimeEvent::PluginSandboxLifecycle {
                    stage: PluginSandboxLifecycleStage::TransportTornDown,
                    ..
                }
            ))
            .count(),
        3
    );
    assert_eq!(
        supervisor
            .events
            .iter()
            .filter(|event| matches!(
                event,
                signal_runtime::RuntimeEvent::PluginSandboxLifecycle {
                    stage: PluginSandboxLifecycleStage::SandboxRestarted,
                    ..
                }
            ))
            .count(),
        3
    );
    assert_eq!(
        supervisor
            .events
            .iter()
            .filter(|event| matches!(
                event,
                signal_runtime::RuntimeEvent::PluginSandboxTransport {
                    stage: PluginSandboxTransportStage::DetachRequested,
                    ..
                }
            ))
            .count(),
        3
    );
    assert_eq!(
        supervisor
            .events
            .iter()
            .filter(|event| matches!(
                event,
                signal_runtime::RuntimeEvent::PluginSandboxTransport {
                    stage: PluginSandboxTransportStage::Detached,
                    ..
                }
            ))
            .count(),
        3
    );
    assert_eq!(
        supervisor
            .events
            .iter()
            .filter(|event| matches!(
                event,
                signal_runtime::RuntimeEvent::HeartbeatCycle {
                    stage: HeartbeatCycleStage::Missed,
                    ..
                }
            ))
            .count(),
        4
    );
    assert_eq!(supervisor.broker_failure_event_count(), 0);
    assert_eq!(supervisor.sandbox_operation_failure_event_count(), 0);
    assert_eq!(supervisor.transport_fault_event_count(), 19);
    assert_eq!(
        supervisor
            .events
            .iter()
            .filter(|event| matches!(
                event,
                signal_runtime::RuntimeEvent::SandboxOperationFailure {
                    stage: SandboxOperationFailureStage::ProcessAttach,
                    ..
                }
            ))
            .count(),
        0
    );
    assert_eq!(
        supervisor
            .events
            .iter()
            .filter(|event| matches!(
                event,
                signal_runtime::RuntimeEvent::BrokerFailure {
                    stage: BrokerFailureStage::PayloadRead,
                    ..
                }
            ))
            .count(),
        0
    );
}
