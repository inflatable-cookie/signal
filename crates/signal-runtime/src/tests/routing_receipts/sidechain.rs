use super::super::*;

#[test]
fn runtime_execution_topology_summary_carries_sidechain_routing_and_fallback_receipts() {
    let mut runtime = prepare_sidechain_runtime();
    let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(128), 2);
    runtime
        .process_engine_block(5, 8, block)
        .expect("process sidechain routing block");

    let observation = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
    assert_eq!(
        observation.execution_topology_summary.secondary_input_count,
        1
    );
    assert_eq!(
        observation
            .execution_topology_summary
            .required_secondary_input_count,
        1
    );
    assert_eq!(
        observation
            .execution_topology_summary
            .terminal_fallback_secondary_input_count,
        0
    );
    let route = &observation.execution_topology_summary.secondary_inputs[0];
    assert_eq!(route.source_id, "sidechain-feed");
    assert_eq!(route.source_bus_id.as_deref(), Some("bus:sidechain:kick"));
    assert_eq!(
        route.target_kind,
        RuntimeSecondaryInputTargetKind::NodeInput
    );
    assert_eq!(route.target_id, "plugin-compressor");
    assert_eq!(route.target_bus_id, "plugin:compressor:sidechain");
    assert_eq!(
        route.attachment_policy,
        crate::RuntimeSecondaryInputAttachmentPolicy::Required
    );
    assert_eq!(
        route.fallback_outcome,
        crate::RuntimeSecondaryInputFallbackOutcome::SafeModeDegradation
    );
    assert!(observation
        .execution_topology_summary
        .nodes
        .iter()
        .any(|node| {
            node.node_id == "plugin-compressor"
                && node
                    .secondary_input
                    .as_ref()
                    .is_some_and(|secondary_input| {
                        secondary_input.target_kind == RuntimeSecondaryInputTargetKind::NodeInput
                            && secondary_input.source_id == "sidechain-feed"
                    })
        }));
    let stage = &observation.plugin_chain_snapshot.chains[0].stages[0];
    let stage_secondary_input = stage
        .secondary_input
        .as_ref()
        .expect("plugin stage should carry sidechain route");
    assert_eq!(
        stage_secondary_input.target_kind,
        RuntimeSecondaryInputTargetKind::PluginInput
    );
    assert_eq!(stage_secondary_input.target_id, "plugin-compressor");
    assert_eq!(
        stage_secondary_input.fallback_outcome,
        crate::RuntimeSecondaryInputFallbackOutcome::SafeModeDegradation
    );

    let supervisor = RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
    let json = supervisor.render_json();
    assert!(json.contains("\"secondary_input_count\":1"));
    assert!(json.contains("\"target_kind\":\"PluginInput\""));
    assert!(json.contains("\"fallback_outcome\":\"SafeModeDegradation\""));
}
