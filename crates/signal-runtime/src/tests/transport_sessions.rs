use super::*;

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

#[test]
fn runtime_owns_transport_session_admission_policy() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    handshake_and_configure(&mut runtime);

    let first = runtime
        .begin_transport_session(
            "sandbox-a",
            "lease-a",
            "region-a",
            TransportAttachIntent::SteadyState,
        )
        .unwrap();
    assert_eq!(first.current_attached_sessions, 1);
    assert_eq!(first.peak_attached_sessions, 1);
    assert_eq!(first.current_recovery_overlap_sessions, 0);
    assert_eq!(first.current_lingering_sessions, 0);
    assert_eq!(
        first.active_sessions[0].state,
        crate::interfaces::TransportSessionState::AttachActive
    );

    let steady_reject = runtime
        .begin_transport_session(
            "sandbox-b",
            "lease-b",
            "region-b",
            TransportAttachIntent::SteadyState,
        )
        .unwrap_err();
    assert_eq!(steady_reject.kind, RuntimeErrorKind::ResourceUnavailable);

    let overlap = runtime
        .begin_transport_session(
            "sandbox-b",
            "lease-b",
            "region-b",
            TransportAttachIntent::RecoveryOverlap,
        )
        .unwrap();
    assert_eq!(overlap.current_attached_sessions, 2);
    assert_eq!(overlap.peak_attached_sessions, 2);
    assert_eq!(overlap.current_recovery_overlap_sessions, 1);
    assert_eq!(overlap.peak_recovery_overlap_sessions, 1);
    assert_eq!(overlap.current_lingering_sessions, 0);

    let overlap_reject = runtime
        .begin_transport_session(
            "sandbox-c",
            "lease-c",
            "region-c",
            TransportAttachIntent::RecoveryOverlap,
        )
        .unwrap_err();
    assert_eq!(overlap_reject.kind, RuntimeErrorKind::ResourceUnavailable);
    assert!(overlap_reject
        .message
        .contains("recovery overlap session limit 1"));

    let snapshot = runtime.get_transport_concurrency_snapshot();
    assert_eq!(snapshot.current_attached_sessions, 2);
    assert_eq!(snapshot.peak_attached_sessions, 2);
    assert_eq!(snapshot.current_lingering_sessions, 0);
    assert_eq!(
        snapshot.last_admitted_sandbox_id.as_deref(),
        Some("sandbox-b")
    );
    assert_eq!(
        snapshot.last_rejected_sandbox_id.as_deref(),
        Some("sandbox-c")
    );
    assert!(snapshot
        .last_rejection_reason
        .as_deref()
        .is_some_and(|reason| reason.contains("recovery overlap session limit 1")));

    let after_end = runtime.end_transport_session("sandbox-a", "lease-a", "region-a");
    assert_eq!(after_end.current_attached_sessions, 1);
    assert_eq!(after_end.current_recovery_overlap_sessions, 1);
    assert_eq!(after_end.current_lingering_sessions, 0);

    let promoted =
        runtime.promote_transport_session_to_steady_state("sandbox-b", "lease-b", "region-b");
    assert_eq!(promoted.current_attached_sessions, 1);
    assert_eq!(promoted.current_recovery_overlap_sessions, 0);
    assert_eq!(promoted.current_lingering_sessions, 0);
    assert_eq!(
        promoted.active_sessions[0].provenance,
        TransportSessionProvenance::RecoveryReplacement
    );

    let re_admit = runtime
        .begin_transport_session(
            "sandbox-c",
            "lease-c",
            "region-c",
            TransportAttachIntent::RecoveryOverlap,
        )
        .unwrap();
    assert_eq!(re_admit.current_attached_sessions, 2);
    assert_eq!(re_admit.current_recovery_overlap_sessions, 1);

    let after_overlap_end = runtime.end_transport_session("sandbox-b", "lease-b", "region-b");
    assert_eq!(after_overlap_end.current_attached_sessions, 1);
    assert_eq!(after_overlap_end.current_recovery_overlap_sessions, 1);
    assert_eq!(after_overlap_end.current_lingering_sessions, 0);

    let after_final_end = runtime.end_transport_session("sandbox-c", "lease-c", "region-c");
    assert_eq!(after_final_end.current_attached_sessions, 0);
    assert_eq!(after_final_end.current_recovery_overlap_sessions, 0);
    assert_eq!(after_final_end.current_lingering_sessions, 0);

    runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .unwrap();
    let reset = runtime.get_transport_concurrency_snapshot();
    assert_eq!(reset.current_attached_sessions, 0);
    assert_eq!(reset.current_lingering_sessions, 0);
    assert!(reset.active_sessions.is_empty());
    assert_eq!(reset.peak_attached_sessions, 0);
    assert_eq!(reset.peak_lingering_sessions, 0);
}

#[test]
fn runtime_transport_session_limits_can_be_widened_for_multiple_steady_sessions() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    handshake_and_configure(&mut runtime);

    let widened = runtime
        .set_transport_session_limits(4, 6)
        .expect("set widened transport session policy");
    assert_eq!(widened.steady_session_limit, 4);
    assert_eq!(widened.recovery_session_limit, 6);

    let first = runtime
        .begin_transport_session(
            "sandbox-a",
            "lease-a",
            "region-a",
            TransportAttachIntent::SteadyState,
        )
        .expect("begin first steady session");
    assert_eq!(first.current_attached_sessions, 1);

    let second = runtime
        .begin_transport_session(
            "sandbox-a",
            "lease-b",
            "region-b",
            TransportAttachIntent::SteadyState,
        )
        .expect("begin second steady session");
    assert_eq!(second.current_attached_sessions, 2);
    assert_eq!(second.steady_session_limit, 4);
    assert_eq!(second.recovery_session_limit, 6);
}

#[test]
fn runtime_tracks_lingering_transport_sessions_as_first_class_admission_state() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    handshake_and_configure(&mut runtime);

    runtime
        .begin_transport_session(
            "sandbox-a",
            "lease-a",
            "region-a",
            TransportAttachIntent::SteadyState,
        )
        .unwrap();
    runtime.record_plugin_sandbox_transport(
        "sandbox-a",
        "lease-a",
        "region-a",
        PluginSandboxTransportStage::DetachRequested,
        Some(2),
        None,
    );

    let requested = runtime.get_transport_concurrency_snapshot();
    assert_eq!(requested.current_attached_sessions, 1);
    assert_eq!(requested.current_lingering_sessions, 1);
    assert_eq!(requested.peak_lingering_sessions, 1);
    assert_eq!(requested.current_detach_requested_sessions, 1);
    assert_eq!(requested.current_detach_faulted_sessions, 0);
    assert_eq!(
        requested.active_sessions[0].state,
        crate::interfaces::TransportSessionState::DetachRequested
    );

    let steady_reject = runtime
        .begin_transport_session(
            "sandbox-b",
            "lease-b",
            "region-b",
            TransportAttachIntent::SteadyState,
        )
        .unwrap_err();
    assert_eq!(steady_reject.kind, RuntimeErrorKind::ResourceUnavailable);
    assert!(steady_reject.message.contains("lingering session"));

    runtime.record_plugin_sandbox_transport(
        "sandbox-a",
        "lease-a",
        "region-a",
        PluginSandboxTransportStage::DetachFault,
        Some(2),
        Some("teardown fault".into()),
    );

    let faulted = runtime.get_transport_concurrency_snapshot();
    assert_eq!(faulted.current_attached_sessions, 1);
    assert_eq!(faulted.current_lingering_sessions, 1);
    assert_eq!(faulted.current_detach_requested_sessions, 0);
    assert_eq!(faulted.current_detach_faulted_sessions, 1);
    assert_eq!(
        faulted.active_sessions[0].state,
        crate::interfaces::TransportSessionState::DetachFaulted
    );

    let overlap = runtime
        .begin_transport_session(
            "sandbox-b",
            "lease-b",
            "region-b",
            TransportAttachIntent::RecoveryOverlap,
        )
        .unwrap();
    assert_eq!(overlap.current_attached_sessions, 2);
    assert_eq!(overlap.current_recovery_overlap_sessions, 1);
    assert_eq!(overlap.current_lingering_sessions, 1);
    assert_eq!(overlap.current_detach_faulted_sessions, 1);
    assert_eq!(overlap.peak_lingering_sessions, 1);

    runtime.end_transport_session("sandbox-b", "lease-b", "region-b");
    runtime.end_transport_session("sandbox-a", "lease-a", "region-a");

    let cleared = runtime.get_transport_concurrency_snapshot();
    assert_eq!(cleared.current_attached_sessions, 0);
    assert_eq!(cleared.current_lingering_sessions, 0);
    assert_eq!(cleared.current_detach_requested_sessions, 0);
    assert_eq!(cleared.current_detach_faulted_sessions, 0);
}
