use crate::{render_supervisor_export_json, HostProfile, Scenario};
use signal_host_local::RecoveryRestartIntent;
use signal_plugin::CompletionState;
use signal_runtime::{
    BlockDispatchStage, BrokerFailureStage, BrokerInvalidationStage, CompletionSlotStage,
    HeartbeatCycleStage, PluginSandboxLifecycleStage, PluginSandboxTransportStage, RuntimeConfig,
    RuntimeEvent, RuntimeEventRecorder, RuntimeEventSink, RuntimeSupervisorReport,
    SandboxOperationFailureStage, SignalRuntime, StopReason,
};

use super::assert_transport_fault_export;

pub(crate) fn verify_export_json_carries_runtime_recovery_sequence() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut recorder = RuntimeEventRecorder::default();
    RuntimeEventSink::push(
        &mut recorder,
        RuntimeEvent::RecoveryCycle {
            sandbox_id: "sandbox-1".into(),
            intent: RecoveryRestartIntent::WatchdogRecovery,
            stop_reason: StopReason::DegradedModeRecovery,
            processing_epoch: Some(4),
        },
    );
    RuntimeEventSink::push(
        &mut recorder,
        RuntimeEvent::PluginSandboxLifecycle {
            sandbox_id: "sandbox-1".into(),
            stage: PluginSandboxLifecycleStage::TransportAttached,
            processing_epoch: Some(4),
        },
    );
    RuntimeEventSink::push(
        &mut recorder,
        RuntimeEvent::PluginSandboxTransport {
            sandbox_id: "sandbox-1".into(),
            lease_id: "lease-4".into(),
            region_id: "region-4".into(),
            stage: PluginSandboxTransportStage::Attached,
            processing_epoch: Some(4),
            detail: None,
        },
    );
    RuntimeEventSink::push(
        &mut recorder,
        RuntimeEvent::HeartbeatCycle {
            sandbox_id: "sandbox-1".into(),
            stage: HeartbeatCycleStage::Responded,
            processing_epoch: Some(4),
            block_sequence: Some(9),
        },
    );
    RuntimeEventSink::push(
        &mut recorder,
        RuntimeEvent::BlockDispatch {
            sandbox_id: "sandbox-1".into(),
            lease_id: "lease-4".into(),
            processing_epoch: 4,
            block_sequence: 9,
            frame_count: 512,
            stage: BlockDispatchStage::Completed,
            completion_state: Some(CompletionState::Completed),
        },
    );
    RuntimeEventSink::push(
        &mut recorder,
        RuntimeEvent::LeaseRollover {
            sandbox_id: "sandbox-1".into(),
            previous_lease_id: "lease-3".into(),
            lease_id: "lease-4".into(),
            processing_epoch: 4,
            first_block_sequence: 9,
        },
    );
    RuntimeEventSink::push(
        &mut recorder,
        RuntimeEvent::BrokerInvalidation {
            sandbox_id: "sandbox-1".into(),
            lease_id: "lease-4".into(),
            processing_epoch: 4,
            block_sequence: Some(9),
            stage: BrokerInvalidationStage::CompletionRegionInvalidated,
            reason: "watchdog recovery teardown".into(),
        },
    );
    RuntimeEventSink::push(
        &mut recorder,
        RuntimeEvent::CompletionSlotTransition {
            sandbox_id: "sandbox-1".into(),
            lease_id: "lease-4".into(),
            processing_epoch: 4,
            block_sequence: 9,
            stage: CompletionSlotStage::TimedOut,
        },
    );
    RuntimeEventSink::push(
        &mut recorder,
        RuntimeEvent::CompletionSlotTransition {
            sandbox_id: "sandbox-1".into(),
            lease_id: "lease-4".into(),
            processing_epoch: 4,
            block_sequence: 9,
            stage: CompletionSlotStage::FallbackApplied,
        },
    );
    RuntimeEventSink::push(
        &mut recorder,
        RuntimeEvent::BrokerFailure {
            sandbox_id: "sandbox-1".into(),
            lease_id: Some("lease-4".into()),
            processing_epoch: Some(4),
            block_sequence: Some(9),
            stage: BrokerFailureStage::PayloadRead,
            detail: "failed to attach shared-memory region: stale mapping".into(),
        },
    );
    RuntimeEventSink::push(
        &mut recorder,
        RuntimeEvent::PluginSandboxTransport {
            sandbox_id: "sandbox-1".into(),
            lease_id: "lease-4".into(),
            region_id: "region-4".into(),
            stage: PluginSandboxTransportStage::DetachRequested,
            processing_epoch: Some(4),
            detail: None,
        },
    );
    RuntimeEventSink::push(
        &mut recorder,
        RuntimeEvent::PluginSandboxTransport {
            sandbox_id: "sandbox-1".into(),
            lease_id: "lease-4".into(),
            region_id: "region-4".into(),
            stage: PluginSandboxTransportStage::Detached,
            processing_epoch: Some(4),
            detail: None,
        },
    );
    RuntimeEventSink::push(
        &mut recorder,
        RuntimeEvent::PluginSandboxTransport {
            sandbox_id: "sandbox-1".into(),
            lease_id: "lease-4".into(),
            region_id: "region-4".into(),
            stage: PluginSandboxTransportStage::DetachFault,
            processing_epoch: Some(4),
            detail: Some("broker detach fault: stale region mapping".into()),
        },
    );
    RuntimeEventSink::push(
        &mut recorder,
        RuntimeEvent::SandboxOperationFailure {
            sandbox_id: "sandbox-1".into(),
            lease_id: Some("lease-4".into()),
            processing_epoch: Some(4),
            operation: "processBlock".into(),
            error_kind: "resourceUnavailable".into(),
            stage: SandboxOperationFailureStage::ProcessAttach,
            detail: "failed to attach shared-memory region: stale mapping".into(),
        },
    );

    let report = RuntimeSupervisorReport::capture(&runtime, &recorder);
    let export = render_supervisor_export_json(
        HostProfile::Local,
        Scenario::Soak,
        "{}".into(),
        &report.profiling_receipt(),
        &report.soak_receipt(),
        &report,
    );
    assert_transport_fault_export(&export);
}
