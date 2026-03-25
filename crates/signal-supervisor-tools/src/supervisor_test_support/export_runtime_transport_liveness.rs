use crate::{render_supervisor_export_json, HostProfile, Scenario};
use signal_plugin::CompletionState;
use signal_runtime::{
    BlockDispatchStage, BrokerFailureStage, CompletionSlotStage, HeartbeatCycleStage,
    PluginSandboxTransportStage, RuntimeConfig, RuntimeEvent, RuntimeEventRecorder,
    RuntimeEventSink, RuntimeSupervisorReport, SignalRuntime,
};

use super::assert_transport_liveness_export;

pub(crate) fn verify_export_json_serializes_per_session_transport_liveness() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut recorder = RuntimeEventRecorder::default();
    RuntimeEventSink::push(
        &mut recorder,
        RuntimeEvent::PluginSandboxTransport {
            sandbox_id: "sandbox-a".into(),
            lease_id: "lease-a".into(),
            region_id: "region-a".into(),
            stage: PluginSandboxTransportStage::Attached,
            processing_epoch: Some(2),
            detail: None,
        },
    );
    RuntimeEventSink::push(
        &mut recorder,
        RuntimeEvent::PluginSandboxTransport {
            sandbox_id: "sandbox-b".into(),
            lease_id: "lease-b".into(),
            region_id: "region-b".into(),
            stage: PluginSandboxTransportStage::Attached,
            processing_epoch: Some(3),
            detail: None,
        },
    );
    RuntimeEventSink::push(
        &mut recorder,
        RuntimeEvent::PluginSandboxTransport {
            sandbox_id: "sandbox-a".into(),
            lease_id: "lease-a".into(),
            region_id: "region-a".into(),
            stage: PluginSandboxTransportStage::DetachRequested,
            processing_epoch: Some(4),
            detail: None,
        },
    );
    RuntimeEventSink::push(
        &mut recorder,
        RuntimeEvent::HeartbeatCycle {
            sandbox_id: "sandbox-a".into(),
            stage: HeartbeatCycleStage::Missed,
            processing_epoch: Some(4),
            block_sequence: Some(11),
        },
    );
    RuntimeEventSink::push(
        &mut recorder,
        RuntimeEvent::HeartbeatCycle {
            sandbox_id: "sandbox-b".into(),
            stage: HeartbeatCycleStage::Responded,
            processing_epoch: Some(5),
            block_sequence: Some(12),
        },
    );
    RuntimeEventSink::push(
        &mut recorder,
        RuntimeEvent::BlockDispatch {
            sandbox_id: "sandbox-a".into(),
            lease_id: "lease-a".into(),
            processing_epoch: 4,
            block_sequence: 11,
            frame_count: 512,
            stage: BlockDispatchStage::TimedOut,
            completion_state: Some(CompletionState::TimedOut),
        },
    );
    RuntimeEventSink::push(
        &mut recorder,
        RuntimeEvent::BlockDispatch {
            sandbox_id: "sandbox-b".into(),
            lease_id: "lease-b".into(),
            processing_epoch: 5,
            block_sequence: 12,
            frame_count: 512,
            stage: BlockDispatchStage::Completed,
            completion_state: Some(CompletionState::Completed),
        },
    );
    RuntimeEventSink::push(
        &mut recorder,
        RuntimeEvent::CompletionSlotTransition {
            sandbox_id: "sandbox-a".into(),
            lease_id: "lease-a".into(),
            processing_epoch: 4,
            block_sequence: 11,
            stage: CompletionSlotStage::TimedOut,
        },
    );
    RuntimeEventSink::push(
        &mut recorder,
        RuntimeEvent::BrokerFailure {
            sandbox_id: "sandbox-b".into(),
            lease_id: Some("lease-b".into()),
            processing_epoch: Some(5),
            block_sequence: Some(12),
            stage: BrokerFailureStage::PayloadRead,
            detail: "stale shared-memory mapping".into(),
        },
    );

    let report = RuntimeSupervisorReport::capture(&runtime, &recorder);
    assert_eq!(
        report.observation.transport_session_summary.active_sessions[0].state,
        signal_runtime::TransportSessionState::DetachRequested
    );
    assert_eq!(
        report.observation.transport_session_summary.active_sessions[0].heartbeat_freshness,
        signal_runtime::TransportHeartbeatFreshness::Missed
    );
    assert_eq!(
        report.observation.transport_session_summary.active_sessions[0].dispatch_state,
        signal_runtime::TransportDispatchState::TimedOut
    );
    assert_eq!(
        report.observation.transport_session_summary.active_sessions[1].heartbeat_freshness,
        signal_runtime::TransportHeartbeatFreshness::Fresh
    );
    assert!(
        report.observation.transport_session_summary.active_sessions[0].transport_fault_count >= 1
    );
    assert!(
        report.observation.transport_session_summary.active_sessions[1].transport_fault_count >= 1
    );

    let export = render_supervisor_export_json(
        HostProfile::Local,
        Scenario::Mixed,
        "{}".into(),
        &report.profiling_receipt(),
        &report.soak_receipt(),
        &report,
    );
    assert_transport_liveness_export(&export);
}
