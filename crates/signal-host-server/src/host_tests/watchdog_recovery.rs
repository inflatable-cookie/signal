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
    assert_runtime_automation_values(&supervisor, 9, 9, 3, 6, 0.1, 0.5, 0.08);
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
    assert_runtime_automation_values(&supervisor, 8, 8, 2, 6, 0.2, 0.55, 0.10);
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
    assert_runtime_automation_values(&supervisor, 10, 10, 2, 8, 0.2, 0.75, 0.18);
    assert_runtime_automation_continuity(&supervisor, 2, 3, &[2, 3], 1);
    assert_runtime_sequence_continuity(&supervisor, &[2, 3], 2, 13, 0, 1);
}

