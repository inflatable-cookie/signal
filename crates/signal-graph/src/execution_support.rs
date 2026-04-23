use std::collections::BTreeMap;

use signal_primitives::AudioBuffer;

use crate::{
    bus, ExecutableGraph, GraphBusLevelReport, GraphExecutionContext, GraphParameterBatch,
    GraphRealtimeExecutionRequest,
};

/// Summary of one processed graph block.
///
/// This is the main current-state observation surface for graph execution. It
/// combines contract/routing/planning counts with parameter-event application
/// stats and basic output telemetry so runtime can snapshot graph behavior
/// without re-deriving scheduler details itself.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphBlockReport {
    /// Identifier of the graph that produced this report.
    pub graph_id: String,
    /// Execution context that was active for this block.
    pub context: GraphExecutionContext,
    /// Total number of nodes in the graph.
    pub node_count: usize,
    /// Number of nodes with [`GraphNodeExecutionClass::Stateful`].
    pub stateful_node_count: usize,
    /// Number of nodes with [`GraphNodeExecutionClass::LatencyBearing`].
    pub latency_node_count: usize,
    /// Number of nodes with [`GraphNodeExecutionClass::PluginBacked`].
    pub plugin_backed_node_count: usize,
    /// Number of contract validation issues found in the plan.
    pub contract_issue_count: usize,
    /// Number of nodes that cleared their output due to silence policy.
    pub silence_clear_node_count: usize,
    /// Number of nodes that required channel layout adaptation.
    pub adaptive_channel_node_count: usize,
    /// Number of nodes with a non-`RetainAcrossBlocks` reset policy.
    pub resettable_node_count: usize,
    /// Total scratch buffers allocated across all nodes.
    pub scratch_buffer_count: usize,
    /// Number of nodes with the `TrackLane` topology role.
    pub track_lane_node_count: usize,
    /// Number of nodes with the `Bus` topology role.
    pub bus_node_count: usize,
    /// Number of nodes with the `Send` or `Return` topology role.
    pub send_return_node_count: usize,
    /// Number of nodes with the `ConsoleNode` topology role.
    pub console_node_count: usize,
    /// Total number of distinct routing buses.
    pub routed_bus_count: usize,
    /// Number of buses with a single writer and single reader.
    pub direct_edge_count: usize,
    /// Number of buses with multiple readers.
    pub fan_in_bus_count: usize,
    /// Number of buses with multiple writers.
    pub fan_out_bus_count: usize,
    /// Number of buses that are both fan-in and fan-out.
    pub mixed_bus_count: usize,
    /// Number of buses whose source contributed only silence this block.
    pub silent_source_bus_count: usize,
    /// Number of channel adaptation failures encountered during execution.
    pub failed_channel_adaptation_count: usize,
    /// Total number of planning phases across all dispatches.
    pub phase_count: usize,
    /// Number of phases that ran on the anticipative lane.
    pub anticipative_phase_count: usize,
    /// Planning group sequence across all phases, in execution order.
    pub phase_order: Vec<crate::GraphNodePlanningGroup>,
    /// Total number of execution lanes in the schedule.
    pub lane_count: usize,
    /// Number of lanes that ran on the anticipative path.
    pub anticipative_lane_count: usize,
    /// Lane sequence in execution order.
    pub lane_order: Vec<crate::GraphExecutionLane>,
    /// Total number of lane dispatches that ran this block.
    pub dispatch_count: usize,
    /// Number of transitions between realtime and anticipative lanes.
    pub dispatch_boundary_count: usize,
    /// Lane for each dispatch, in execution order.
    pub dispatch_order: Vec<crate::GraphExecutionLane>,
    /// Number of dispatches that ran on the anticipative lane (prepared).
    pub prepared_dispatch_count: usize,
    /// Number of dispatches that ran on the realtime lane.
    pub realtime_dispatch_count: usize,
    /// `1` when both a prepared and a realtime dispatch ran (handoff occurred),
    /// `0` otherwise.
    pub dispatch_handoff_count: usize,
    /// Total number of DSP stages across all nodes.
    pub stage_count: usize,
    /// Number of stages backed by dynamically-constructed kernels.
    pub dynamic_kernel_stage_count: usize,
    /// State model used for dynamic stage kernels this block.
    pub dynamic_stage_state_model: crate::GraphDynamicStageStateModel,
    /// Sum of all per-node latency values, in samples.
    pub total_latency_samples: u32,
    /// Highest per-node latency value, in samples.
    pub max_node_latency_samples: u32,
    /// Sum of all per-node tail values, in samples.
    pub total_tail_samples: u32,
    /// Highest per-node tail value, in samples.
    pub max_node_tail_samples: u32,
    /// Latency on the primary output bus (`"main:out"`), in samples.
    pub output_latency_samples: u32,
    /// Highest latency across all buses, in samples.
    pub max_bus_latency_samples: u32,
    /// Tail time on the primary output bus, in samples.
    pub output_tail_samples: u32,
    /// Highest tail time across all buses, in samples.
    pub max_bus_tail_samples: u32,
    /// Epoch of the parameter batch applied this block, if any.
    pub parameter_epoch: Option<u64>,
    /// Number of parameter events in the batch.
    pub parameter_event_count: usize,
    /// Number of distinct nodes targeted by at least one parameter event.
    pub parameter_targeted_node_count: usize,
    /// Number of parameter events that were ignored (unknown node/stage or
    /// out-of-range sample offset).
    pub parameter_ignored_event_count: usize,
    /// Total number of sub-blocks created by event-boundary splitting.
    pub parameter_sub_block_count: usize,
    /// Number of parameter events coalesced due to the sub-block cap.
    pub parameter_coalesced_event_count: usize,
    /// Number of frames (samples per channel) in the processed buffer.
    pub frame_count: usize,
    /// Number of channels in the processed buffer.
    pub channel_count: usize,
    /// Peak absolute sample value of the input buffer before processing.
    pub input_peak: f32,
    /// Peak absolute sample value at the end of the anticipative pass, if one
    /// ran.
    pub prework_output_peak: Option<f32>,
    /// Peak absolute sample value of the input at the start of the realtime
    /// pass, if an anticipative pass preceded it.
    pub realtime_input_peak: Option<f32>,
    /// Peak absolute sample value of the final output buffer.
    pub output_peak: f32,
    /// Root-mean-square level of the final output buffer.
    pub output_rms: f32,
    /// Number of entries in `bus_levels`.
    pub bus_level_count: usize,
    /// Per-bus peak and RMS levels measured after execution.
    pub bus_levels: Vec<GraphBusLevelReport>,
    /// The first sample of the output buffer, useful for deterministic testing.
    pub first_output_sample: Option<f32>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct GraphBusState {
    pub(crate) buses: BTreeMap<String, AudioBuffer>,
    pub(crate) latencies: BTreeMap<String, u32>,
    pub(crate) tails: BTreeMap<String, u32>,
    pub(crate) silent_source_bus_count: usize,
    pub(crate) failed_channel_adaptation_count: usize,
}

pub(crate) fn build_block_report(
    graph: &ExecutableGraph,
    request: &GraphRealtimeExecutionRequest<'_>,
    parameter_batch: Option<&GraphParameterBatch>,
    prepared_dispatch_count: usize,
    realtime_dispatch_count: usize,
    prework_output_peak: Option<f32>,
    realtime_input_peak: Option<f32>,
    working_buffer: &AudioBuffer,
    working_state: &GraphBusState,
) -> GraphBlockReport {
    let parameter_report =
        crate::parameter_application_report(&graph.plan, request.input.frames().0, parameter_batch);
    let output_latency_samples = working_state
        .latencies
        .get("main:out")
        .copied()
        .unwrap_or(request.routing.output_latency_samples);
    let max_bus_latency_samples = working_state
        .latencies
        .values()
        .copied()
        .max()
        .unwrap_or(request.routing.max_bus_latency_samples);
    let output_tail_samples = working_state
        .tails
        .get("main:out")
        .copied()
        .unwrap_or(request.routing.output_tail_samples);
    let max_bus_tail_samples = working_state
        .tails
        .values()
        .copied()
        .max()
        .unwrap_or(request.routing.max_bus_tail_samples);
    let bus_levels = working_state
        .buses
        .iter()
        .map(|(bus_id, buffer)| GraphBusLevelReport {
            bus_id: bus_id.clone(),
            peak: bus::peak_abs(buffer.samples()),
            rms: bus::rms(buffer.samples()),
            latency_samples: working_state.latencies.get(bus_id).copied().unwrap_or(0),
            tail_samples: working_state.tails.get(bus_id).copied().unwrap_or(0),
        })
        .collect::<Vec<_>>();

    GraphBlockReport {
        graph_id: graph.plan.graph_id.clone(),
        context: request.context.clone(),
        node_count: graph.node_count(),
        stateful_node_count: graph.stateful_node_count(),
        latency_node_count: graph.latency_node_count(),
        plugin_backed_node_count: graph.plugin_backed_node_count(),
        contract_issue_count: request.contract.issue_count,
        silence_clear_node_count: request.contract.silence_clear_node_count,
        adaptive_channel_node_count: request.contract.adaptive_channel_node_count,
        resettable_node_count: request.contract.resettable_node_count,
        scratch_buffer_count: request.contract.scratch_buffer_count,
        track_lane_node_count: request.contract.track_lane_node_count,
        bus_node_count: request.contract.bus_node_count,
        send_return_node_count: request.contract.send_return_node_count,
        console_node_count: request.contract.console_node_count,
        routed_bus_count: request.routing.routed_bus_count,
        direct_edge_count: request.routing.direct_edge_count,
        fan_in_bus_count: request.routing.fan_in_bus_count,
        fan_out_bus_count: request.routing.fan_out_bus_count,
        mixed_bus_count: request.routing.mixed_bus_count,
        silent_source_bus_count: working_state.silent_source_bus_count,
        failed_channel_adaptation_count: working_state.failed_channel_adaptation_count,
        phase_count: request.planning.phase_count,
        anticipative_phase_count: request.planning.anticipative_phase_count,
        phase_order: request.planning.phase_order.clone(),
        lane_count: request.planning.lane_count,
        anticipative_lane_count: request.planning.anticipative_lane_count,
        lane_order: request.planning.lane_order.clone(),
        dispatch_count: request.planning.dispatch_count,
        dispatch_boundary_count: request.planning.dispatch_boundary_count,
        dispatch_order: request
            .planning
            .dispatches
            .iter()
            .map(|dispatch| dispatch.lane)
            .collect(),
        prepared_dispatch_count,
        realtime_dispatch_count,
        dispatch_handoff_count: usize::from(
            prepared_dispatch_count > 0 && realtime_dispatch_count > 0,
        ),
        stage_count: graph.stage_count(),
        dynamic_kernel_stage_count: graph.dynamic_kernel_stage_count(),
        dynamic_stage_state_model: graph.dynamic_stage_state_model(),
        total_latency_samples: graph.total_latency_samples(),
        max_node_latency_samples: graph.max_node_latency_samples(),
        total_tail_samples: graph.total_tail_samples(),
        max_node_tail_samples: graph.max_node_tail_samples(),
        output_latency_samples,
        max_bus_latency_samples,
        output_tail_samples,
        max_bus_tail_samples,
        parameter_epoch: parameter_batch.map(|batch| batch.epoch),
        parameter_event_count: parameter_report.event_count,
        parameter_targeted_node_count: parameter_report.targeted_node_count,
        parameter_ignored_event_count: parameter_report.ignored_event_count,
        parameter_sub_block_count: parameter_report.sub_block_count,
        parameter_coalesced_event_count: parameter_report.coalesced_event_count,
        frame_count: working_buffer.frames().0,
        channel_count: working_buffer.channel_count().0,
        input_peak: request.input_peak,
        prework_output_peak,
        realtime_input_peak,
        output_peak: bus::peak_abs(working_buffer.samples()),
        output_rms: bus::rms(working_buffer.samples()),
        bus_level_count: bus_levels.len(),
        bus_levels,
        first_output_sample: working_buffer.samples().first().copied(),
    }
}
