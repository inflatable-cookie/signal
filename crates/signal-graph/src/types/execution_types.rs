use signal_primitives::AudioBuffer;

use super::{
    GraphContractSummary, GraphExecutionLane, GraphNodeExecutionClass, GraphNodePlanningGroup,
    GraphParameterBatch, GraphRoutingSummary,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphPlannedNode {
    pub node_id: String,
    pub execution_class: GraphNodeExecutionClass,
    pub group: GraphNodePlanningGroup,
    pub latency_samples: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphPlannedPhase {
    pub group: GraphNodePlanningGroup,
    pub node_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphLaneDispatch {
    pub lane: GraphExecutionLane,
    pub phase_order: Vec<GraphNodePlanningGroup>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GraphPlanningSummary {
    pub inline_realtime_node_count: usize,
    pub stateful_realtime_node_count: usize,
    pub anticipative_eligible_node_count: usize,
    pub plugin_backed_node_count: usize,
    pub phase_count: usize,
    pub anticipative_phase_count: usize,
    pub phase_order: Vec<GraphNodePlanningGroup>,
    pub lane_count: usize,
    pub anticipative_lane_count: usize,
    pub lane_order: Vec<GraphExecutionLane>,
    pub dispatch_count: usize,
    pub dispatch_boundary_count: usize,
    pub dispatches: Vec<GraphLaneDispatch>,
    pub phases: Vec<GraphPlannedPhase>,
    pub planned_nodes: Vec<GraphPlannedNode>,
}

/// Per-block execution context supplied by runtime.
///
/// The graph does not invent processing epochs, projection epochs, transport
/// state, or parameter epochs itself; it consumes the runtime-owned context and
/// reports it back through [`GraphBlockReport`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GraphExecutionContext {
    pub processing_epoch: u64,
    pub block_sequence: u64,
    pub projection_epoch: u64,
    pub parameter_epoch: u64,
    pub configured_block_size: usize,
    pub anticipative_enabled: bool,
    pub transport_playing: bool,
    pub transport_tempo_bpm: f64,
    pub timeline_position_samples: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphExecutionRequest {
    pub context: GraphExecutionContext,
    pub buffer: AudioBuffer,
    pub parameter_batch: Option<GraphParameterBatch>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphNodeRenderOverride {
    pub node_id: String,
    pub buffer: AudioBuffer,
    pub latency_samples: u32,
    pub tail_samples: u32,
    pub bypassed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphCapturedBusOutput {
    pub bus_id: String,
    pub buffer: AudioBuffer,
}

/// Prepared anticipative dispatch output that can be handed into the later
/// realtime dispatch path.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphPreparedDispatch {
    pub buses: Vec<GraphPreparedBus>,
    pub output_peak: f32,
    pub dispatch_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphPreparedBus {
    pub bus_id: String,
    pub buffer: AudioBuffer,
    pub latency_samples: u32,
    pub tail_samples: u32,
}

#[derive(Clone, Debug)]
pub struct GraphRealtimeExecutionRequest<'a> {
    pub input: &'a AudioBuffer,
    pub input_peak: f32,
    pub prepared: Option<GraphPreparedDispatch>,
    pub context: GraphExecutionContext,
    pub parameter_batch: Option<&'a GraphParameterBatch>,
    pub planning: &'a GraphPlanningSummary,
    pub contract: &'a GraphContractSummary,
    pub routing: &'a GraphRoutingSummary,
    pub node_render_overrides: &'a [GraphNodeRenderOverride],
    pub captured_bus_ids: &'a [String],
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphBusLevelReport {
    pub bus_id: String,
    pub peak: f32,
    pub rms: f32,
    pub latency_samples: u32,
    pub tail_samples: u32,
}
