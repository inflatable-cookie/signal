use super::*;

#[test]
fn local_shared_host_edge_exports_runtime_sidechain_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-local-sidechain".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("local host-edge sidechain handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("local host-edge sidechain configure should succeed");
    apply_public_sidechain_graph(&mut runtime, "graph:host-local:sidechain");
    record_public_plugin_sandbox_ready(
        &mut runtime,
        "sandbox:host-local:sidechain",
        PluginFormat::Clap,
        "plugin:clap:host-local-sidechain-compressor",
        1,
    );

    let host = LocalRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    let topology = &report.observation.execution_topology_summary;
    assert_eq!(topology.secondary_input_count, 1);
    assert_eq!(topology.required_secondary_input_count, 1);
    let route = &topology.secondary_inputs[0];
    assert_eq!(route.source_id, "kick-sidechain");
    assert_eq!(route.source_bus_id.as_deref(), Some("bus:sidechain:kick"));
    assert_eq!(
        route.target_kind,
        RuntimeSecondaryInputTargetKind::NodeInput
    );
    assert_eq!(route.target_id, "compressor");
    assert_eq!(route.target_bus_id, "plugin:compressor:sidechain");
    assert_eq!(
        route.attachment_policy,
        RuntimeSecondaryInputAttachmentPolicy::Required
    );
    assert_eq!(
        route.fallback_outcome,
        RuntimeSecondaryInputFallbackOutcome::SafeModeDegradation
    );

    let compressor = topology
        .nodes
        .iter()
        .find(|node| node.node_id == "compressor")
        .expect("compressor node should be present");
    let node_secondary_input = compressor
        .secondary_input
        .as_ref()
        .expect("compressor should carry sidechain receipt");
    assert_eq!(node_secondary_input.source_id, "kick-sidechain");
    assert_eq!(
        node_secondary_input.target_kind,
        RuntimeSecondaryInputTargetKind::NodeInput
    );

    let stage_secondary_input = report.observation.plugin_chain_snapshot.chains[0].stages[0]
        .secondary_input
        .as_ref()
        .expect("local host-edge sidechain plugin stage should be exported");
    assert_eq!(
        stage_secondary_input.target_kind,
        RuntimeSecondaryInputTargetKind::PluginInput
    );
    assert_eq!(stage_secondary_input.target_id, "compressor");

}
