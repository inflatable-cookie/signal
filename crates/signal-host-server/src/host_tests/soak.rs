use super::super::host_test_support::{
    assert_runtime_automation_continuity, assert_runtime_automation_values,
    assert_runtime_plugin_event_snapshot, assert_runtime_sequence_continuity,
    prepare_server_host_with_lifecycle, prepare_server_host_without_lifecycle,
    temp_media_fixture_path,
};
use super::super::ServerRuntimeHost;
use signal_graph::{GraphNodeExecutionClass, GraphNodeTopologyRole, GraphStageSpec};
use signal_plugin::{CompletionState, PluginFormat, WatchdogTriggerReason};
use signal_plugin_clap::ClapSandboxLifecycleHarness;
use signal_primitives::{ChannelCount, ChannelLayout};
use signal_runtime::{
    BlockDispatchStage, BrokerFailureStage, BrokerInvalidationStage, CompletionSlotStage,
    GraphContractProjection, GraphNodeBufferContractProjection, GraphNodeBusEndpointProjection,
    GraphNodeContractProjection, GraphNodeProjection, GraphNodeTopologyProjection,
    GraphProjection, HandshakeRequest, HeartbeatCycleStage, LingeringCleanupMode,
    PluginBackedNodeBinding, PluginBackedNodeBindingProjection, PluginSandboxLifecycleStage,
    PluginSandboxSpec, PluginSandboxTransportStage, PluginScanRequest, RecoveryRestartIntent,
    RuntimeConfig, RuntimeConfigRequest, RuntimeErrorKind, RuntimeExternalIoDeviceChangeState,
    RuntimeExternalIoHealthState, RuntimeExternalIoLoopbackState,
    RuntimeExternalIoMonitoringState, RuntimeExternalIoMonitoringTapPoint,
    RuntimeExternalIoPrimaryRole, RuntimeLifecycleApi, RuntimeMediaAssetRegistration,
    RuntimeMediaPreviewState, RuntimeObservationApi, RuntimePluginHostPlatform,
    RuntimePluginIsolationOutcome, RuntimePluginParityBand, RuntimeProjectionApi,
    RuntimeReadiness, RuntimeSupervisorApi, SandboxOperationFailureStage, SignalRuntime,
    StopReason, TransportAttachIntent,
};
use std::{fs, path::Path};

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
    assert_runtime_automation_values(&supervisor, 12, 12, 2, 10, 0.2, 0.95, 0.26);
    assert_runtime_automation_continuity(&supervisor, 2, 4, &[2, 3, 4], 2);
    assert_runtime_sequence_continuity(&supervisor, &[2, 3, 4], 2, 17, 0, 2);
}

#[test]
fn server_host_mixed_watchdog_soak_tracks_deadlines_and_heartbeats() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);
    let summary = host
        .boot_with_mixed_watchdog_soak()
        .expect("mixed watchdog soak boot");
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
    assert_eq!(summary.execution.processed_blocks, 14);
    assert_eq!(summary.execution.last_block_sequence, 17);
    assert_eq!(summary.faults.deadline_misses, 2);
    assert_eq!(summary.faults.heartbeat_misses, 4);
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
    assert_eq!(supervisor.block_dispatch_event_count(), 28);
    assert_eq!(supervisor.lease_rollover_event_count(), 2);
    assert_eq!(supervisor.invalidation_event_count(), 6);
    assert_eq!(supervisor.completion_slot_event_count(), 45);
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
        14
    );
    assert_eq!(
        supervisor
            .events
            .iter()
            .filter(|event| matches!(
                event,
                signal_runtime::RuntimeEvent::BlockDispatch {
                    stage: BlockDispatchStage::TimedOut,
                    ..
                }
            ))
            .count(),
        2
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
        14
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
                    stage: CompletionSlotStage::TimedOut,
                    ..
                }
            ))
            .count(),
        2
    );
    assert_eq!(
        supervisor
            .events
            .iter()
            .filter(|event| matches!(
                event,
                signal_runtime::RuntimeEvent::CompletionSlotTransition {
                    stage: CompletionSlotStage::FallbackApplied,
                    ..
                }
            ))
            .count(),
        2
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
    assert_runtime_automation_values(&supervisor, 14, 14, 2, 12, 0.2, 0.95, 0.26);
    assert_runtime_automation_continuity(&supervisor, 2, 4, &[2, 3, 4], 2);
    assert_runtime_sequence_continuity(&supervisor, &[2, 3, 4], 2, 17, 0, 2);
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
        host.runtime()
            .get_supervision_snapshot()
            .last_watchdog_trigger,
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
