use super::*;

#[test]
fn server_host_soak_path_rolls_across_multiple_lease_generations() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);
    let summary = host.boot_with_watchdog_soak().expect("watchdog soak boot");
    let supervisor = host.supervisor_report();

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
    assert_eq!(summary.execution.processed_blocks, 12);
    assert_eq!(summary.execution.last_block_sequence, 17);
    assert_eq!(summary.faults.heartbeat_misses, 6);
    assert_eq!(
        supervisor
            .observation
            .supervision_snapshot
            .watchdog_restart_count,
        3
    );
    assert!(
        supervisor
            .observation
            .supervision_snapshot
            .safe_mode_enabled
    );
    assert!(summary.transport.shared_memory_lease_id.contains("epoch-4"));
    assert_eq!(summary.last_payload.first_output_sample, Some(17.0));
    assert!(matches!(
        supervisor.observation.readiness,
        signal_runtime::RuntimeReadiness::Degraded { .. }
    ));
    assert_eq!(supervisor.observation.control_snapshot.start_count, 4);
    assert_eq!(supervisor.observation.control_snapshot.stop_count, 3);
    assert_eq!(
        supervisor.observation.control_snapshot.last_stop_reason,
        Some(StopReason::DegradedModeRecovery)
    );
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
                    stage: PluginSandboxLifecycleStage::InstanceDeactivated,
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
                    stage: PluginSandboxLifecycleStage::InstanceReset,
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
                    stage: PluginSandboxLifecycleStage::InstanceDestroyed,
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
        6
    );
    assert_eq!(supervisor.block_dispatch_event_count(), 24);
    assert_eq!(supervisor.lease_rollover_event_count(), 2);
    assert_eq!(supervisor.invalidation_event_count(), 6);
    assert_eq!(supervisor.completion_slot_event_count(), 39);
    assert_eq!(supervisor.broker_failure_event_count(), 0);
    assert_eq!(supervisor.sandbox_operation_failure_event_count(), 0);
    assert_eq!(supervisor.transport_fault_event_count(), 0);
    assert_eq!(
        supervisor
            .events
            .iter()
            .filter(|event| matches!(
                event,
                signal_runtime::RuntimeEvent::BlockDispatch {
                    stage: BlockDispatchStage::Requested,
                    ..
                }
            ))
            .count(),
        12
    );
    assert_eq!(
        supervisor
            .events
            .iter()
            .filter(|event| matches!(
                event,
                signal_runtime::RuntimeEvent::BlockDispatch {
                    stage: BlockDispatchStage::Completed,
                    ..
                }
            ))
            .count(),
        12
    );
    assert_eq!(
        supervisor
            .events
            .iter()
            .filter(|event| matches!(
                event,
                signal_runtime::RuntimeEvent::BrokerInvalidation {
                    stage: BrokerInvalidationStage::CompletionRegionInvalidated,
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
                signal_runtime::RuntimeEvent::BrokerInvalidation {
                    stage: BrokerInvalidationStage::LeaseEpochInvalidated,
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
                signal_runtime::RuntimeEvent::CompletionSlotTransition {
                    stage: CompletionSlotStage::ReadyForProcessing,
                    ..
                }
            ))
            .count(),
        12
    );
    assert_eq!(
        supervisor
            .events
            .iter()
            .filter(|event| matches!(
                event,
                signal_runtime::RuntimeEvent::CompletionSlotTransition {
                    stage: CompletionSlotStage::Processing,
                    ..
                }
            ))
            .count(),
        12
    );
    assert_eq!(
        supervisor
            .events
            .iter()
            .filter(|event| matches!(
                event,
                signal_runtime::RuntimeEvent::CompletionSlotTransition {
                    stage: CompletionSlotStage::Completed,
                    ..
                }
            ))
            .count(),
        12
    );
    assert_eq!(
        supervisor
            .events
            .iter()
            .filter(|event| matches!(
                event,
                signal_runtime::RuntimeEvent::CompletionSlotTransition {
                    stage: CompletionSlotStage::Invalidated,
                    ..
                }
            ))
            .count(),
        3
    );
    assert_runtime_automation_values(
        &supervisor,
        RuntimeAutomationExpectations {
            value_events: 12,
            modulation_events: 12,
            gesture_begin_events: 2,
            gesture_end_events: 10,
            first_value: 0.2,
            last_value: 0.95,
            last_modulation: 0.26,
        },
    );
    assert_runtime_automation_continuity(&supervisor, 2, 4, &[2, 3, 4], 2);
    assert_runtime_sequence_continuity(&supervisor, &[2, 3, 4], 2, 17, 0, 2);
}

