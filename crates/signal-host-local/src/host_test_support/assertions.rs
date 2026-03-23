use super::super::{
    LocalRuntimeHostSummary, LOCAL_DEMO_PLUGIN_LATENCY_SAMPLES, LOCAL_DEMO_PLUGIN_NODE_ID,
    LOCAL_DEMO_PLUGIN_TAIL_SAMPLES,
};
use signal_plugin::LoopRange;
use signal_runtime::RuntimeSupervisorReport;

pub(crate) fn assert_runtime_automation_values(
    supervisor: &RuntimeSupervisorReport,
    value_events: usize,
    modulation_events: usize,
    gesture_begin_events: usize,
    gesture_end_events: usize,
    first_value: f32,
    last_value: f32,
    last_modulation: f32,
) {
    let snapshot = &supervisor.observation.automation_snapshot;
    assert_eq!(snapshot.parameter_id, 4096);
    assert_eq!(snapshot.value_events, value_events);
    assert_eq!(snapshot.modulation_events, modulation_events);
    assert_eq!(snapshot.gesture_begin_events, gesture_begin_events);
    assert_eq!(snapshot.gesture_end_events, gesture_end_events);
    assert!(snapshot
        .first_value
        .is_some_and(|observed| (observed - first_value).abs() < 1.0e-6));
    assert!(snapshot
        .last_value
        .is_some_and(|observed| (observed - last_value).abs() < 1.0e-6));
    assert!(snapshot
        .last_modulation
        .is_some_and(|observed| (observed - last_modulation).abs() < 1.0e-6));
}

pub(crate) fn assert_runtime_automation_continuity(
    supervisor: &RuntimeSupervisorReport,
    first_epoch: u64,
    last_epoch: u64,
    epochs: &[u64],
    lease_rollovers: usize,
) {
    let snapshot = &supervisor.observation.automation_snapshot;
    assert_eq!(snapshot.first_epoch, Some(first_epoch));
    assert_eq!(snapshot.last_epoch, Some(last_epoch));
    assert_eq!(snapshot.segment_count, epochs.len());
    assert_eq!(snapshot.segment_epochs, epochs);
    assert_eq!(snapshot.lease_rollovers, lease_rollovers);
}

pub(crate) fn assert_runtime_plugin_event_snapshot(
    supervisor: &RuntimeSupervisorReport,
    first_epoch: u64,
    last_epoch: u64,
    epochs: &[u64],
    lease_rollovers: usize,
) {
    let snapshot = &supervisor.observation.plugin_event_snapshot;
    assert!(snapshot.total_events > 0, "{snapshot:?}");
    assert!(snapshot.note_events > 0, "{snapshot:?}");
    assert!(snapshot.note_expression_events > 0, "{snapshot:?}");
    assert!(snapshot.midi_events > 0, "{snapshot:?}");
    assert!(snapshot.last_generated_event_bytes > 0, "{snapshot:?}");
    assert_eq!(snapshot.first_epoch, Some(first_epoch));
    assert_eq!(snapshot.last_epoch, Some(last_epoch));
    assert_eq!(snapshot.segment_count, epochs.len());
    assert_eq!(snapshot.segment_epochs, epochs);
    assert_eq!(snapshot.lease_rollovers, lease_rollovers);
    assert!(snapshot.last_block_sequence.is_some(), "{snapshot:?}");
}

pub(crate) fn assert_runtime_sequence_continuity(
    supervisor: &RuntimeSupervisorReport,
    epochs: &[u64],
    first_block_sequence: u64,
    last_block_sequence: u64,
    sequence_gaps: usize,
    lease_rollovers: usize,
) {
    let timeline = &supervisor
        .observation
        .timeline_snapshot
        .block_sequence_continuity;
    assert_eq!(timeline.segment_count(), epochs.len(), "{timeline:?}");
    assert_eq!(timeline.segment_epochs(), epochs, "{timeline:?}");
    assert_eq!(
        timeline.first_block_sequence(),
        Some(first_block_sequence),
        "{timeline:?}"
    );
    assert_eq!(
        timeline.last_block_sequence(),
        Some(last_block_sequence),
        "{timeline:?}"
    );
    assert_eq!(timeline.sequence_gaps, sequence_gaps, "{timeline:?}");
    assert_eq!(timeline.lease_rollovers, lease_rollovers, "{timeline:?}");
}

pub(crate) fn assert_plugin_dispatch_summary(
    summary: &LocalRuntimeHostSummary,
    supervisor: &RuntimeSupervisorReport,
    expected_bypass_count: u32,
) {
    let dispatch = summary
        .plugin_dispatch
        .as_ref()
        .expect("plugin dispatch summary");
    let expected_timeline = ((dispatch.block_sequence as i64) * 512).rem_euclid(16 * 512);
    let expected_automation = ((dispatch.block_sequence % 8) as f32) / 7.0;

    assert_eq!(
        dispatch.processing_epoch,
        summary.execution.processing_epoch
    );
    assert_eq!(
        dispatch.block_sequence,
        summary.execution.last_block_sequence
    );
    assert_eq!(dispatch.render_context.sample_rate_hz, 48_000);
    assert_eq!(dispatch.render_context.tempo_bpm, 126.0);
    assert_eq!(
        dispatch.render_context.timeline_position_samples,
        expected_timeline
    );
    assert!(dispatch.render_context.playing);
    assert!(!dispatch.render_context.bypassed);
    assert_eq!(
        dispatch.render_context.loop_range,
        Some(LoopRange {
            start_samples: 0,
            end_samples: 16 * 512,
        })
    );
    assert_eq!(dispatch.render_context.deadline_frames, 512);
    assert!(dispatch
        .automation_value
        .is_some_and(|value| (value - expected_automation).abs() < 1.0e-6));
    assert_eq!(dispatch.render_bypass_count, expected_bypass_count);
    assert!(!dispatch.last_render_bypassed);
    assert_eq!(
        dispatch.last_render_latency_samples,
        LOCAL_DEMO_PLUGIN_LATENCY_SAMPLES
    );
    assert_eq!(
        dispatch.last_render_tail_samples,
        LOCAL_DEMO_PLUGIN_TAIL_SAMPLES
    );
    assert!(supervisor
        .observation
        .engine_block_snapshot
        .planned_nodes
        .iter()
        .any(|node| node.node_id == LOCAL_DEMO_PLUGIN_NODE_ID
            && node.plugin_sandbox_id.as_deref() == Some("local-default-sandbox")));
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .last_execution_context
            .as_ref()
            .map(|context| context.transport_playing),
        Some(dispatch.render_context.playing)
    );
    assert_eq!(
        supervisor
            .observation
            .engine_block_snapshot
            .last_execution_context
            .as_ref()
            .map(|context| context.timeline_position_samples),
        Some(dispatch.render_context.timeline_position_samples)
    );
}

pub(crate) fn assert_local_plugin_topology(summary: &LocalRuntimeHostSummary) {
    let topology = &summary.topology;
    assert_eq!(topology.node_count, 4);
    assert_eq!(topology.track_lane_node_count, 2);
    assert_eq!(topology.bus_node_count, 1);
    assert_eq!(topology.console_node_count, 1);
    assert_eq!(topology.track_lane_group_count, 1);
    assert_eq!(topology.bus_group_count, 2);
    assert_eq!(topology.console_group_count, 1);
    assert!(topology.nodes.iter().any(|node| {
        node.node_id == "track-input"
            && node.track_lane_id.as_deref() == Some("track:lead")
            && node.bus_group_id.as_deref() == Some("mix:tracks")
            && node.input_bus_id == "main:in"
            && node.output_bus_id == "bus:track:lead"
    }));
    assert!(topology.nodes.iter().any(|node| {
        node.node_id == LOCAL_DEMO_PLUGIN_NODE_ID
            && node.topology_role == signal_graph::GraphNodeTopologyRole::TrackLane
            && node.track_lane_id.as_deref() == Some("track:lead")
            && node.bus_group_id.as_deref() == Some("mix:tracks")
            && node.plugin_sandbox_id.as_deref() == Some("local-default-sandbox")
            && node.input_bus_id == "bus:track:lead"
            && node.output_bus_id == "bus:mix:tracks"
    }));
    assert!(topology.nodes.iter().any(|node| {
        node.node_id == "bus-main"
            && node.topology_role == signal_graph::GraphNodeTopologyRole::Bus
            && node.bus_group_id.as_deref() == Some("mix:master")
            && node.input_bus_id == "bus:mix:tracks"
            && node.output_bus_id == "bus:console:main"
    }));
    assert!(topology.nodes.iter().any(|node| {
        node.node_id == "output-main"
            && node.topology_role == signal_graph::GraphNodeTopologyRole::ConsoleNode
            && node.console_group_id.as_deref() == Some("console:main")
            && node.input_bus_id == "bus:console:main"
            && node.output_bus_id == "main:out"
    }));
}
