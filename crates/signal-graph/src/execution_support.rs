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
    pub graph_id: String,
    pub context: GraphExecutionContext,
    pub node_count: usize,
    pub stateful_node_count: usize,
    pub latency_node_count: usize,
    pub plugin_backed_node_count: usize,
    pub contract_issue_count: usize,
    pub silence_clear_node_count: usize,
    pub adaptive_channel_node_count: usize,
    pub resettable_node_count: usize,
    pub scratch_buffer_count: usize,
    pub track_lane_node_count: usize,
    pub bus_node_count: usize,
    pub send_return_node_count: usize,
    pub console_node_count: usize,
    pub routed_bus_count: usize,
    pub direct_edge_count: usize,
    pub fan_in_bus_count: usize,
    pub fan_out_bus_count: usize,
    pub mixed_bus_count: usize,
    pub silent_source_bus_count: usize,
    pub phase_count: usize,
    pub anticipative_phase_count: usize,
    pub phase_order: Vec<crate::GraphNodePlanningGroup>,
    pub lane_count: usize,
    pub anticipative_lane_count: usize,
    pub lane_order: Vec<crate::GraphExecutionLane>,
    pub dispatch_count: usize,
    pub dispatch_boundary_count: usize,
    pub dispatch_order: Vec<crate::GraphExecutionLane>,
    pub prepared_dispatch_count: usize,
    pub realtime_dispatch_count: usize,
    pub dispatch_handoff_count: usize,
    pub stage_count: usize,
    pub dynamic_kernel_stage_count: usize,
    pub dynamic_stage_state_model: crate::GraphDynamicStageStateModel,
    pub total_latency_samples: u32,
    pub max_node_latency_samples: u32,
    pub total_tail_samples: u32,
    pub max_node_tail_samples: u32,
    pub output_latency_samples: u32,
    pub max_bus_latency_samples: u32,
    pub output_tail_samples: u32,
    pub max_bus_tail_samples: u32,
    pub parameter_epoch: Option<u64>,
    pub parameter_event_count: usize,
    pub parameter_targeted_node_count: usize,
    pub parameter_ignored_event_count: usize,
    pub parameter_sub_block_count: usize,
    pub parameter_coalesced_event_count: usize,
    pub frame_count: usize,
    pub channel_count: usize,
    pub input_peak: f32,
    pub prework_output_peak: Option<f32>,
    pub realtime_input_peak: Option<f32>,
    pub output_peak: f32,
    pub output_rms: f32,
    pub bus_level_count: usize,
    pub bus_levels: Vec<GraphBusLevelReport>,
    pub first_output_sample: Option<f32>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct GraphBusState {
    pub(crate) buses: BTreeMap<String, AudioBuffer>,
    pub(crate) latencies: BTreeMap<String, u32>,
    pub(crate) tails: BTreeMap<String, u32>,
    pub(crate) silent_source_bus_count: usize,
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
        dispatch_handoff_count: usize::from(prepared_dispatch_count > 0 && realtime_dispatch_count > 0),
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
