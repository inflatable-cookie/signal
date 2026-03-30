use super::super::super::*;

pub(super) fn assert_compensation_recall_and_supervisor_receipts(runtime: &SignalRuntime) {
    let snapshot = runtime.get_plugin_chain_snapshot();
    assert_eq!(
        snapshot.chains[0].stages[0].compensation_state,
        RuntimePluginCompensationState::Compensated
    );
    assert_eq!(
        snapshot.chains[0].stages[0].recall_state,
        RuntimePluginRecallState::Warm
    );
    assert_eq!(
        snapshot.chains[0].stages[0].recall.state,
        RuntimePluginRecallState::Warm
    );
    assert_eq!(
        snapshot.chains[0].stages[0]
            .recall
            .payload
            .sandbox_id
            .as_deref(),
        Some("sandbox-a")
    );
    assert_eq!(
        snapshot.chains[0].stages[0].recall.payload.lifecycle_state,
        Some(RuntimePluginLifecycleState::Ready)
    );
    assert_eq!(
        snapshot.chains[0].stages[0].recall.payload.transport_stage,
        Some(PluginSandboxTransportStage::Attached)
    );
    assert_eq!(
        snapshot.chains[0].stages[1].compensation_state,
        RuntimePluginCompensationState::Bypassed
    );
    assert_eq!(
        snapshot.chains[0].stages[1].recall_state,
        RuntimePluginRecallState::Recovered
    );
    assert_eq!(
        snapshot.chains[0].stages[1].recall.state,
        RuntimePluginRecallState::Recovered
    );
    assert_eq!(
        snapshot.chains[0].stages[1].recall.payload.recovery_count,
        1
    );
    assert_eq!(snapshot.chains[0].stages[1].recall.payload.restart_count, 1);
    assert_eq!(
        snapshot.chains[0].stages[1]
            .recall
            .payload
            .last_restart_intent,
        Some(RecoveryRestartIntent::CrashRecovery)
    );
    assert_eq!(
        snapshot.chains[0].stages[1].recall.payload.last_stop_reason,
        Some(StopReason::DegradedModeRecovery)
    );

    let handoff = runtime.get_plugin_recall_handoff_snapshot();
    assert_eq!(handoff.stage_count, 2);
    assert_eq!(handoff.warm_stage_count, 1);
    assert_eq!(handoff.recovered_stage_count, 1);
    assert_eq!(handoff.unavailable_stage_count, 0);
    assert_eq!(handoff.stages[1].chain_id, "track:lead");
    assert_eq!(handoff.stages[1].node_id, "plugin-b");
    assert_eq!(
        handoff.stages[1].recall_state,
        RuntimePluginRecallState::Recovered
    );
    assert_eq!(
        handoff.stages[1].recall_payload,
        snapshot.chains[0].stages[1].recall.payload
    );

    let recorder = RuntimeEventRecorder::default();
    let observation = RuntimeObservationReport::capture(runtime, &recorder);
    let compact = observation.render_compact();
    assert!(compact.contains("plugin_chains=1/2"));
    assert!(compact.contains("plugin_chain_compensated=1"));
    assert!(compact.contains("plugin_chain_bypassed=1"));

    let supervisor = RuntimeSupervisorReport::capture(runtime, &recorder);
    let multiline = supervisor.render_multiline();
    assert!(multiline.contains("plugin_chain_count=1"));
    assert!(multiline.contains("plugin_chain_0_stage_0=plugin-a"));
    assert!(multiline.contains("plugin_chain_0_stage_1=plugin-b"));
    assert!(multiline.contains("recall=Recovered/sandbox=Some(\"sandbox-b\")"));
    let json = supervisor.render_json();
    assert!(json.contains("\"plugin_chain_snapshot\":{\"chain_count\":1"));
    assert!(json.contains("\"recall\":{\"state\":\"Recovered\""));
    assert!(json.contains("\"payload\":{\"sandbox_id\":\"sandbox-b\""));
    assert!(json.contains("\"last_restart_intent\":\"CrashRecovery\""));
    assert!(json.contains("\"compensation_state\":\"Bypassed\""));
}
