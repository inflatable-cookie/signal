use super::*;

pub(crate) fn filled_stereo_buffer(sample_rate_hz: u32, frames: usize, value: f32) -> AudioBuffer {
    let mut buffer = AudioBuffer::new(
        SampleRate(sample_rate_hz),
        ChannelLayout::Stereo,
        FrameCount(frames),
    );
    buffer.samples_mut().fill(value);
    buffer
}

pub(crate) fn handshake_and_configure_with_disabled_forecast(
    runtime: &mut SignalRuntime,
    anticipative_enabled: bool,
) {
    handshake_and_configure_with_anticipative(runtime, anticipative_enabled);
    runtime
        .set_prework_forecast_mode(RuntimePreworkForecastMode::Disabled)
        .unwrap();
}

pub(crate) fn seed_pending_prework_targets(
    runtime: &mut SignalRuntime,
    admitted_from_block_sequence: u64,
    target_block_sequences: &[u64],
) {
    runtime.engine.pending_prework_targets.clear();
    let targets = target_block_sequences
        .iter()
        .map(|target_block_sequence| RuntimePreworkWindowTarget {
            target_block_sequence: *target_block_sequence,
            admitted_from_block_sequence,
            buffer: synthetic_stereo_block(
                runtime.config.sample_rate,
                FrameCount(runtime.config.graph.block_size),
                *target_block_sequence,
            ),
            parameter_epoch_override: None,
            transport_override: None,
        })
        .collect::<Vec<_>>();
    let graph_id = runtime
        .engine
        .graph
        .as_ref()
        .map(|graph| graph.graph_id().to_string());
    runtime.engine.reconcile_pending_prework_targets(
        &targets,
        graph_id.as_deref(),
        runtime.projection_epoch,
        runtime.latest_parameter_epoch,
        runtime.applied_transport,
        runtime.config.graph.block_size,
    );
}

pub(crate) fn apply_current_forecast_block_state(runtime: &mut SignalRuntime, block_sequence: u64) {
    let policy = runtime
        .prework_forecast_policy
        .clone()
        .expect("forecast policy configured");
    runtime
        .apply_forecast_transport_projection(
            runtime.forecast_transport_projection_for_block(block_sequence, &policy),
        )
        .expect("apply forecast transport projection");
    runtime
        .apply_parameter_batch(runtime.forecast_parameter_batch_for_block(block_sequence, &policy))
        .expect("apply forecast parameter batch");
}

pub(crate) fn apply_latency_runtime_graph(runtime: &mut SignalRuntime, graph_id: &str) {
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: graph_id.into(),
            node_count: 2,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "inline".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                },
                GraphNodeProjection {
                    node_id: "latency".into(),
                    execution_class: GraphNodeExecutionClass::LatencyBearing,
                    latency_samples: 24,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                },
            ],
        })
        .unwrap();
}

pub(crate) fn install_scheduler_topology_runtime_graph(
    runtime: &mut SignalRuntime,
    graph_id: &str,
    track_lane_ids: &[&str],
    include_missing_track_lane_id: bool,
) {
    let mut nodes = vec![GraphNodeSpec {
        node_id: "lookahead".into(),
        execution_class: GraphNodeExecutionClass::LatencyBearing,
        latency_samples: 32,
        tail_samples: 0,
        buffer_contract: GraphNodeBufferContract {
            input: GraphNodeBusEndpoint::new("main:in", ChannelLayout::Stereo),
            output: GraphNodeBusEndpoint::new("bus:lookahead", ChannelLayout::Stereo),
            ..GraphNodeBufferContract::default()
        },
        topology: GraphNodeTopologyMetadata {
            role: Some(GraphNodeTopologyRole::Utility),
            track_lane_id: None,
            bus_group_id: None,
            console_group_id: None,
            send_return_id: None,
        },
        stages: vec![GraphStageSpec::Gain { linear: 0.5 }],
    }];

    for (index, lane_id) in track_lane_ids.iter().enumerate() {
        nodes.push(GraphNodeSpec {
            node_id: format!("track-{index}"),
            execution_class: GraphNodeExecutionClass::Stateful,
            latency_samples: 0,
            tail_samples: 0,
            buffer_contract: GraphNodeBufferContract {
                input: GraphNodeBusEndpoint::new("main:in", ChannelLayout::Stereo),
                output: GraphNodeBusEndpoint::new("bus:tracks", ChannelLayout::Stereo),
                ..GraphNodeBufferContract::default()
            },
            topology: GraphNodeTopologyMetadata {
                role: Some(GraphNodeTopologyRole::TrackLane),
                track_lane_id: Some((*lane_id).into()),
                bus_group_id: Some("mix:tracks".into()),
                console_group_id: None,
                send_return_id: None,
            },
            stages: vec![GraphStageSpec::Gain { linear: 0.8 }],
        });
    }

    if include_missing_track_lane_id {
        nodes.push(GraphNodeSpec {
            node_id: "track-missing".into(),
            execution_class: GraphNodeExecutionClass::Stateful,
            latency_samples: 0,
            tail_samples: 0,
            buffer_contract: GraphNodeBufferContract {
                input: GraphNodeBusEndpoint::new("main:in", ChannelLayout::Stereo),
                output: GraphNodeBusEndpoint::new("bus:tracks", ChannelLayout::Stereo),
                ..GraphNodeBufferContract::default()
            },
            topology: GraphNodeTopologyMetadata {
                role: Some(GraphNodeTopologyRole::TrackLane),
                track_lane_id: None,
                bus_group_id: Some("mix:tracks".into()),
                console_group_id: None,
                send_return_id: None,
            },
            stages: vec![GraphStageSpec::Gain { linear: 0.7 }],
        });
    }

    nodes.push(GraphNodeSpec {
        node_id: "bus-main".into(),
        execution_class: GraphNodeExecutionClass::Stateful,
        latency_samples: 0,
        tail_samples: 0,
        buffer_contract: GraphNodeBufferContract {
            input: GraphNodeBusEndpoint::new("bus:tracks", ChannelLayout::Stereo),
            output: GraphNodeBusEndpoint::new("bus:master", ChannelLayout::Stereo),
            ..GraphNodeBufferContract::default()
        },
        topology: GraphNodeTopologyMetadata {
            role: Some(GraphNodeTopologyRole::Bus),
            track_lane_id: None,
            bus_group_id: Some("mix:master".into()),
            console_group_id: None,
            send_return_id: None,
        },
        stages: vec![GraphStageSpec::HardClip { threshold: 0.9 }],
    });

    nodes.push(GraphNodeSpec {
        node_id: "console-main".into(),
        execution_class: GraphNodeExecutionClass::PureTransform,
        latency_samples: 0,
        tail_samples: 0,
        buffer_contract: GraphNodeBufferContract {
            input: GraphNodeBusEndpoint::new("bus:master", ChannelLayout::Stereo),
            output: GraphNodeBusEndpoint::new("main:out", ChannelLayout::Stereo),
            ..GraphNodeBufferContract::default()
        },
        topology: GraphNodeTopologyMetadata {
            role: Some(GraphNodeTopologyRole::ConsoleNode),
            track_lane_id: None,
            bus_group_id: None,
            console_group_id: Some("console:main".into()),
            send_return_id: None,
        },
        stages: vec![GraphStageSpec::HardClip { threshold: 0.8 }],
    });

    runtime.engine.graph = Some(ExecutableGraph::new(graph_id, nodes));
    runtime
        .engine
        .refresh_planning(runtime.anticipative_enabled);
    runtime.refresh_scheduler_topology_summary();
}
