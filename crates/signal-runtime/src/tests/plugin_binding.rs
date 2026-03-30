use super::*;

#[test]
fn runtime_plugin_bindings_project_into_snapshot_and_track_bound_sessions() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:plugin-bindings".into(),
            node_count: 3,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "inline".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                },
                GraphNodeProjection {
                    node_id: "plugin".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.8 }],
                },
                GraphNodeProjection {
                    node_id: "latency".into(),
                    execution_class: GraphNodeExecutionClass::LatencyBearing,
                    latency_samples: 96,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                },
            ],
        })
        .unwrap();
    runtime
        .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
            graph_id: "graph:runtime:plugin-bindings".into(),
            bindings: vec![PluginBackedNodeBinding {
                node_id: "plugin".into(),
                sandbox_id: "sandbox-bound".into(),
            }],
        })
        .expect("apply plugin-backed bindings");

    let snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(snapshot.prework_service_bound_plugin_sandboxes, 1);
    assert_eq!(snapshot.prework_service_active_bound_plugin_sandboxes, 0);
    assert_eq!(snapshot.prework_service_degraded_bound_plugin_sandboxes, 0);
    assert_eq!(snapshot.prework_service_missing_bound_plugin_sandboxes, 1);
    assert!(snapshot.planned_nodes.iter().any(|node| {
        node.node_id == "plugin" && node.plugin_sandbox_id.as_deref() == Some("sandbox-bound")
    }));

    runtime
        .begin_transport_session(
            "sandbox-bound",
            "lease-bound",
            "region-bound",
            TransportAttachIntent::SteadyState,
        )
        .expect("begin bound transport session");
    runtime.record_plugin_sandbox_transport(
        "sandbox-bound",
        "lease-bound",
        "region-bound",
        PluginSandboxTransportStage::Attached,
        Some(1),
        None,
    );

    let snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(snapshot.prework_service_bound_plugin_sandboxes, 1);
    assert_eq!(snapshot.prework_service_active_bound_plugin_sandboxes, 1);
    assert_eq!(snapshot.prework_service_degraded_bound_plugin_sandboxes, 0);
    assert_eq!(snapshot.prework_service_missing_bound_plugin_sandboxes, 0);
    assert_eq!(
        snapshot.prework_service_semantic_policy,
        RuntimePreworkServiceSemanticPolicy::PluginConstrained
    );
}

#[test]
fn runtime_consumes_plugin_node_render_batch_on_matching_engine_block() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:plugin-render".into(),
            node_count: 1,
            nodes: vec![GraphNodeProjection {
                node_id: "plugin".into(),
                execution_class: GraphNodeExecutionClass::PluginBacked,
                latency_samples: 0,
                stages: vec![GraphStageSpec::HardClip { threshold: 0.2 }],
            }],
        })
        .expect("apply graph");
    runtime
        .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
            graph_id: "graph:runtime:plugin-render".into(),
            bindings: vec![PluginBackedNodeBinding {
                node_id: "plugin".into(),
                sandbox_id: "sandbox:render".into(),
            }],
        })
        .expect("apply bindings");
    runtime
        .apply_plugin_node_render_batch(PluginNodeRenderBatch {
            graph_id: "graph:runtime:plugin-render".into(),
            processing_epoch: 1,
            block_sequence: 1,
            renders: vec![PluginNodeRender {
                node_id: "plugin".into(),
                sandbox_id: "sandbox:render".into(),
                output: AudioBuffer::from_interleaved(
                    SampleRate(48_000),
                    ChannelLayout::Stereo,
                    vec![0.75, -0.5, 0.5, -0.25, 0.25, -0.125, 0.125, -0.0625],
                ),
                latency_samples: 24,
                tail_samples: 40,
                bypassed: false,
            }],
        })
        .expect("apply plugin node render batch");

    let first = runtime
        .process_engine_block(
            1,
            1,
            AudioBuffer::new(SampleRate(48_000), ChannelLayout::Stereo, FrameCount(4)),
        )
        .expect("process plugin render block");
    let second = runtime
        .process_engine_block(
            1,
            2,
            AudioBuffer::new(SampleRate(48_000), ChannelLayout::Stereo, FrameCount(4)),
        )
        .expect("process fallback block");

    assert_eq!(
        first.output.samples(),
        &[0.75, -0.5, 0.5, -0.25, 0.25, -0.125, 0.125, -0.0625]
    );
    assert_eq!(first.snapshot.output_tail_samples, 40);
    assert_eq!(second.output.samples(), &[0.0; 8]);
    assert_eq!(second.snapshot.output_tail_samples, 0);
}
