use super::*;

#[test]
fn runtime_multichannel_layout_summary_maps_canonical_and_custom_roles() {
    let stereo = RuntimeMultichannelLayoutSummary::from_channel_layout(ChannelLayout::Stereo);
    assert_eq!(
        stereo.canonical_layout,
        Some(RuntimeCanonicalChannelLayout::Stereo)
    );
    assert_eq!(
        stereo.channel_roles,
        vec![
            RuntimeChannelRole::FrontLeft,
            RuntimeChannelRole::FrontRight
        ]
    );
    assert!(!stereo.uses_custom_fallback);

    let custom = RuntimeMultichannelLayoutSummary::from_channel_layout(ChannelLayout::Count(
        signal_primitives::ChannelCount(7),
    ));
    assert_eq!(custom.canonical_layout, None);
    assert_eq!(custom.channel_roles.len(), 7);
    assert!(matches!(
        custom.channel_roles.last(),
        Some(RuntimeChannelRole::Discrete(6))
    ));
    assert!(custom.uses_custom_fallback);
}

#[test]
fn runtime_execution_topology_summary_carries_multichannel_layout_and_bus_intents() {
    let snapshot = RuntimeEngineBlockSnapshot {
        planned_nodes: vec![RuntimePlannedGraphNode {
            node_id: "track-main".into(),
            execution_class: GraphNodeExecutionClass::PluginBacked,
            group: GraphNodePlanningGroup::InlineRealtime,
            latency_samples: 32,
            topology_role: GraphNodeTopologyRole::TrackLane,
            track_lane_id: Some("track:main".into()),
            bus_group_id: Some("bus:main".into()),
            console_group_id: None,
            send_return_id: None,
            input_bus_id: "track:main-in".into(),
            output_bus_id: "track:main-out".into(),
            input_channels: ChannelLayout::Stereo,
            output_channels: ChannelLayout::Count(signal_primitives::ChannelCount(6)),
            input_layout: RuntimeMultichannelLayoutSummary::from_channel_layout(
                ChannelLayout::Stereo,
            ),
            output_layout: RuntimeMultichannelLayoutSummary::from_channel_layout(
                ChannelLayout::Count(signal_primitives::ChannelCount(6)),
            ),
            input_bus_intent: RuntimeBusIntent::MainProgram,
            output_bus_intent: RuntimeBusIntent::MainProgram,
            secondary_input: None,
            spatial_execution: None,
            plugin_sandbox_id: Some("sandbox:track-main".into()),
        }],
        lane_order: vec![GraphExecutionLane::Realtime],
        ..RuntimeEngineBlockSnapshot::default()
    };

    let topology = RuntimeExecutionTopologySummary::from_snapshot(&snapshot);
    assert_eq!(topology.node_count, 1);
    assert_eq!(
        topology.nodes[0].input_layout.canonical_layout,
        Some(RuntimeCanonicalChannelLayout::Stereo)
    );
    assert_eq!(
        topology.nodes[0].output_layout.canonical_layout,
        Some(RuntimeCanonicalChannelLayout::Surround5_1)
    );
    assert_eq!(
        topology.nodes[0].input_bus_intent,
        RuntimeBusIntent::MainProgram
    );
    assert_eq!(
        topology.nodes[0].output_bus_intent,
        RuntimeBusIntent::MainProgram
    );
}
