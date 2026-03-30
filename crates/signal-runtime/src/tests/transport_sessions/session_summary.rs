use super::super::*;

#[test]
fn transport_session_summary_tracks_concurrent_active_sessions() {
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

    let diagnostics = recorder.diagnostics();
    let summary = crate::interfaces::TransportSessionSummary::from_diagnostics(&diagnostics);
    assert_eq!(summary.current_attached_session_count, 2);
    assert_eq!(summary.max_concurrent_attached_sessions, 2);
    assert_eq!(
        summary.current_state,
        crate::interfaces::TransportSessionState::DetachRequested
    );
    assert!(summary.currently_attached);
    assert_eq!(summary.active_sessions.len(), 2);
    assert_eq!(summary.active_sandbox_id.as_deref(), Some("sandbox-a"));
    assert_eq!(summary.active_lease_id.as_deref(), Some("lease-a"));
    assert_eq!(summary.active_region_id.as_deref(), Some("region-a"));
    assert_eq!(summary.active_block_sequence, Some(12));
    assert_eq!(summary.active_sessions[0].sandbox_id.as_str(), "sandbox-a");
    assert_eq!(
        summary.active_sessions[0].state,
        crate::interfaces::TransportSessionState::DetachRequested
    );
    assert!(summary.active_sessions[0].currently_attached);
    assert_eq!(
        summary.active_sessions[0].heartbeat_freshness,
        crate::interfaces::TransportHeartbeatFreshness::Missed
    );
    assert_eq!(
        summary.active_sessions[0].dispatch_state,
        crate::interfaces::TransportDispatchState::TimedOut
    );
    assert_eq!(summary.active_sessions[0].processing_epoch, Some(4));
    assert_eq!(summary.active_sessions[0].active_block_sequence, Some(11));
    assert_eq!(summary.active_sessions[0].transport_fault_count, 2);
    assert_eq!(
        summary.active_sessions[0].last_transport_fault_source,
        Some(crate::interfaces::TransportFaultSource::RuntimeDispatch)
    );
    assert_eq!(
        summary.active_sessions[0].last_transport_fault_stage,
        Some(crate::interfaces::TransportFaultStage::CompletionSlotTimedOut)
    );
    assert_eq!(
        summary.active_sessions[0].last_transport_fault_phase,
        Some(crate::interfaces::TransportFaultPhase::Dispatch)
    );
    assert_eq!(
        summary.active_sessions[0].last_transport_fault_processing_epoch,
        Some(4)
    );
    assert_eq!(
        summary.active_sessions[0].last_transport_fault_block_sequence,
        Some(11)
    );
    assert_eq!(summary.active_sessions[1].sandbox_id.as_str(), "sandbox-b");
    assert_eq!(
        summary.active_sessions[1].state,
        crate::interfaces::TransportSessionState::AttachActive
    );
    assert!(summary.active_sessions[1].currently_attached);
    assert_eq!(
        summary.active_sessions[1].heartbeat_freshness,
        crate::interfaces::TransportHeartbeatFreshness::Fresh
    );
    assert_eq!(
        summary.active_sessions[1].dispatch_state,
        crate::interfaces::TransportDispatchState::Completed
    );
    assert_eq!(summary.active_sessions[1].processing_epoch, Some(5));
    assert_eq!(summary.active_sessions[1].active_block_sequence, Some(12));
    assert_eq!(summary.active_sessions[1].transport_fault_count, 1);
    assert_eq!(
        summary.active_sessions[1].last_transport_fault_source,
        Some(crate::interfaces::TransportFaultSource::HostBroker)
    );
    assert_eq!(
        summary.active_sessions[1].last_transport_fault_stage,
        Some(crate::interfaces::TransportFaultStage::PayloadRead)
    );
    assert_eq!(
        summary.active_sessions[1].last_transport_fault_phase,
        Some(crate::interfaces::TransportFaultPhase::Dispatch)
    );
    assert_eq!(
        summary.active_sessions[1].last_transport_fault_processing_epoch,
        Some(5)
    );
    assert_eq!(
        summary.active_sessions[1].last_transport_fault_block_sequence,
        Some(12)
    );
}
