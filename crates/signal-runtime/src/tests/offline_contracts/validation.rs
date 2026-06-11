use super::*;

#[test]
fn runtime_offline_render_contract_preview_rejects_misaligned_chain_and_recall_contracts() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:offline-render-misaligned-contract".into(),
            node_count: 1,
            nodes: vec![GraphNodeProjection {
                node_id: "plugin-a".into(),
                execution_class: GraphNodeExecutionClass::PluginBacked,
                latency_samples: 24,
                stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
            }],
        })
        .expect("apply graph");
    runtime
        .apply_graph_contract_projection(GraphContractProjection {
            graph_id: "graph:runtime:offline-render-misaligned-contract".into(),
            contract_count: 1,
            nodes: vec![GraphNodeContractProjection {
                node_id: "plugin-a".into(),
                buffer_contract: GraphNodeBufferContractProjection::default(),
                topology: GraphNodeTopologyProjection {
                    role: Some(GraphNodeTopologyRole::TrackLane),
                    track_lane_id: Some("track:lead".into()),
                    bus_group_id: Some("mix:tracks".into()),
                    console_group_id: None,
                    send_return_id: None,
                },
            }],
        })
        .expect("apply graph contracts");
    runtime
        .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
            graph_id: "graph:runtime:offline-render-misaligned-contract".into(),
            bindings: vec![PluginBackedNodeBinding {
                node_id: "plugin-a".into(),
                sandbox_id: "sandbox-a".into(),
            }],
        })
        .expect("apply bindings");
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox-a",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );

    let mut handoff = runtime.get_plugin_recall_handoff_snapshot();
    handoff.stage_count = 0;
    handoff.stages.clear();
    let request = RuntimeOfflineRenderRequest {
        request_id: "render:misaligned".into(),
        timeline_start_samples: 0,
        duration_samples: 48_000,
        export_sample_rate_hz: 48_000,
        include_main_mix: true,
        artifact_root_path: None,
        stem_targets: Vec::new(),
        freeze_artifacts: Vec::new(),
    };

    let error = RuntimeOfflineRenderContractPreview::from_runtime_state(
        &request,
        &runtime.get_execution_topology_summary(),
        &runtime.get_clip_processing_pipeline_snapshot(),
        &runtime.get_media_pipeline_snapshot(),
        &runtime.get_tempo_map_snapshot(),
        &runtime.get_marker_analysis_snapshot(),
        &handoff,
    )
    .expect_err("misaligned chain and recall contracts should fail");
    assert_eq!(error.kind, RuntimeErrorKind::InvalidState);
    assert!(error
        .message
        .contains("aligned plugin chain and recall handoff"));
}

#[test]
fn runtime_offline_render_contract_preview_carries_sidechain_dependency_receipts() {
    let runtime = prepare_sidechain_runtime();
    let handoff = runtime.get_plugin_recall_handoff_snapshot();
    let request = RuntimeOfflineRenderRequest {
        request_id: "render:sidechain-preview".into(),
        timeline_start_samples: 0,
        duration_samples: 24_000,
        export_sample_rate_hz: 48_000,
        include_main_mix: true,
        artifact_root_path: None,
        stem_targets: Vec::new(),
        freeze_artifacts: Vec::new(),
    };

    let preview = RuntimeOfflineRenderContractPreview::from_runtime_state(
        &request,
        &runtime.get_execution_topology_summary(),
        &runtime.get_clip_processing_pipeline_snapshot(),
        &runtime.get_media_pipeline_snapshot(),
        &runtime.get_tempo_map_snapshot(),
        &runtime.get_marker_analysis_snapshot(),
        &handoff,
    )
    .expect("build offline render sidechain preview");

    assert_eq!(preview.chain_contract.secondary_input_count, 1);
    assert_eq!(preview.chain_contract.required_secondary_input_count, 1);
    assert_eq!(preview.chain_contract.optional_secondary_input_count, 0);
    assert_eq!(preview.chain_contract.disabled_secondary_input_count, 0);
    assert_eq!(
        preview
            .chain_contract
            .terminal_fallback_secondary_input_count,
        0
    );
    assert_eq!(preview.chain_contract.bus_connection_count, 2);
    assert_eq!(preview.chain_contract.auxiliary_path_count, 1);
    let route = &preview.chain_contract.secondary_inputs[0];
    assert_eq!(route.source_id, "sidechain-feed");
    assert_eq!(
        route.target_kind,
        RuntimeSecondaryInputTargetKind::RenderInput
    );
    assert_eq!(route.target_id, "offline-render");
    assert_eq!(route.target_bus_id, "plugin:compressor:sidechain");
    assert_eq!(
        route.fallback_outcome,
        crate::RuntimeSecondaryInputFallbackOutcome::SafeModeDegradation
    );
    assert!(preview
        .chain_contract
        .bus_connections
        .iter()
        .any(|connection| {
            connection.connection_id
                == "track-input:bus:track:lead->plugin-compressor:bus:track:lead"
                && connection.source_bus_role == crate::RuntimeBusRole::ProgramMain
                && connection.target_bus_role == crate::RuntimeBusRole::ProgramMain
        }));
    assert!(preview.chain_contract.auxiliary_paths.iter().any(|path| {
        path.auxiliary_path_id == "bus_group:mix:tracks"
            && path.path_kind == crate::RuntimeAuxiliaryPathKind::Submix
    }));
}
