use signal_primitives::AudioBuffer;

use super::{
    GraphContractSummary, GraphExecutionLane, GraphNodeExecutionClass, GraphNodePlanningGroup,
    GraphParameterBatch, GraphRoutingSummary,
};

/// A single node as resolved by the planner, including its scheduling group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphPlannedNode {
    /// Node identifier, matching [`GraphNodeSpec::node_id`].
    pub node_id: String,
    /// Execution class used to derive the planning group.
    pub execution_class: GraphNodeExecutionClass,
    /// Planning group this node was assigned to.
    pub group: GraphNodePlanningGroup,
    /// Latency this node introduces, in samples.
    pub latency_samples: u32,
}

/// A planning phase: a group of nodes that share the same
/// [`GraphNodePlanningGroup`] and can run together within a lane dispatch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphPlannedPhase {
    /// Planning group for all nodes in this phase.
    pub group: GraphNodePlanningGroup,
    /// Ordered list of node IDs in this phase.
    pub node_ids: Vec<String>,
}

/// A dispatch unit: one execution lane with the ordered phases it will run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphLaneDispatch {
    /// Lane on which this dispatch runs.
    pub lane: GraphExecutionLane,
    /// Phase groups to execute within this dispatch, in order.
    pub phase_order: Vec<GraphNodePlanningGroup>,
}

/// Aggregate result of the graph planner: node group counts, phase/lane
/// ordering, and the full dispatch schedule for one block.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GraphPlanningSummary {
    /// Number of nodes placed in the [`GraphNodePlanningGroup::InlineRealtime`] group.
    pub inline_realtime_node_count: usize,
    /// Number of nodes placed in the [`GraphNodePlanningGroup::StatefulRealtime`] group.
    pub stateful_realtime_node_count: usize,
    /// Number of nodes placed in the [`GraphNodePlanningGroup::AnticipativeEligible`] group.
    pub anticipative_eligible_node_count: usize,
    /// Number of plugin-backed nodes across all groups.
    pub plugin_backed_node_count: usize,
    /// Total number of phases across all dispatches.
    pub phase_count: usize,
    /// Number of phases that run on the anticipative lane.
    pub anticipative_phase_count: usize,
    /// Ordered sequence of planning groups as they appear in the schedule.
    pub phase_order: Vec<GraphNodePlanningGroup>,
    /// Total number of execution lanes in the schedule.
    pub lane_count: usize,
    /// Number of lanes assigned to the anticipative execution path.
    pub anticipative_lane_count: usize,
    /// Ordered sequence of execution lanes in the schedule.
    pub lane_order: Vec<GraphExecutionLane>,
    /// Total number of lane dispatches in the schedule.
    pub dispatch_count: usize,
    /// Number of dispatch boundaries (transitions between realtime and
    /// anticipative lanes).
    pub dispatch_boundary_count: usize,
    /// Full ordered list of lane dispatches to execute this block.
    pub dispatches: Vec<GraphLaneDispatch>,
    /// Full ordered list of planning phases.
    pub phases: Vec<GraphPlannedPhase>,
    /// All planned nodes with their resolved scheduling metadata.
    pub planned_nodes: Vec<GraphPlannedNode>,
}

/// Per-block execution context supplied by runtime.
///
/// The graph does not invent processing epochs, projection epochs, transport
/// state, or parameter epochs itself; it consumes the runtime-owned context and
/// reports it back through [`GraphBlockReport`].
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GraphExecutionContext {
    /// Monotonically increasing counter incremented each time the audio engine
    /// processes a block.
    pub processing_epoch: u64,
    /// Block sequence number within the current session.
    pub block_sequence: u64,
    /// Epoch used to invalidate anticipative pre-computation results.
    pub projection_epoch: u64,
    /// Epoch associated with the current parameter batch, used for
    /// cache-invalidation.
    pub parameter_epoch: u64,
    /// Nominal block size in frames as configured by the runtime.
    pub configured_block_size: usize,
    /// Whether the anticipative execution lane is active this block.
    pub anticipative_enabled: bool,
    /// Whether the transport is currently rolling.
    pub transport_playing: bool,
    /// Current transport tempo in beats per minute.
    pub transport_tempo_bpm: f64,
    /// Current playhead position in samples from the session start.
    pub timeline_position_samples: i64,
}

/// A self-contained execution request carrying context, buffer, and optional
/// parameter events.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphExecutionRequest {
    /// Execution context for this block.
    pub context: GraphExecutionContext,
    /// Audio buffer to process (owned).
    pub buffer: AudioBuffer,
    /// Optional parameter events to apply within this block.
    pub parameter_batch: Option<GraphParameterBatch>,
}

/// Overrides the rendered output for a specific node during execution, used
/// for offline or test scenarios where pre-computed audio replaces live DSP.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphNodeRenderOverride {
    /// The node whose output to replace.
    pub node_id: String,
    /// Pre-rendered buffer to inject as the node's output.
    pub buffer: AudioBuffer,
    /// Latency associated with the pre-rendered buffer, in samples.
    pub latency_samples: u32,
    /// Tail time associated with the pre-rendered buffer, in samples.
    pub tail_samples: u32,
    /// When `true`, substitute silence rather than `buffer`.
    pub bypassed: bool,
}

/// A snapshot of a bus buffer captured during execution, keyed by bus ID.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphCapturedBusOutput {
    /// Identifier of the captured bus.
    pub bus_id: String,
    /// Buffer contents at the time of capture.
    pub buffer: AudioBuffer,
}

/// Prepared anticipative dispatch output that can be handed into the later
/// primary-lane ([`crate::GraphExecutionLane::Realtime`]) dispatch path.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphPreparedDispatch {
    /// Pre-computed bus buffers, one per bus touched by the anticipative lane.
    pub buses: Vec<GraphPreparedBus>,
    /// Output peak measured at the end of the anticipative dispatch.
    pub output_peak: f32,
    /// Number of dispatches that ran on the anticipative lane.
    pub dispatch_count: usize,
}

/// A single pre-computed bus result from an anticipative dispatch.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphPreparedBus {
    /// Identifier of the bus this buffer belongs to.
    pub bus_id: String,
    /// Pre-computed audio data for this bus.
    pub buffer: AudioBuffer,
    /// Latency accumulated on this bus, in samples.
    pub latency_samples: u32,
    /// Tail time accumulated on this bus, in samples.
    pub tail_samples: u32,
}

/// Full input bundle for the primary-lane execution path.
///
/// Carries all pre-computed summaries (planning, contract, routing) plus the
/// input buffer and any prepared anticipative output so the primary lane does
/// not need to re-derive them.
#[derive(Clone, Debug)]
pub struct GraphRealtimeExecutionRequest<'a> {
    /// The incoming audio buffer.
    pub input: &'a AudioBuffer,
    /// Peak absolute sample value of the input buffer, measured before
    /// processing starts.
    pub input_peak: f32,
    /// Pre-computed anticipative lane output, if the anticipative path ran.
    pub prepared: Option<GraphPreparedDispatch>,
    /// Execution context for this block.
    pub context: GraphExecutionContext,
    /// Parameter events to apply this block, if any.
    pub parameter_batch: Option<&'a GraphParameterBatch>,
    /// Pre-computed planning summary.
    pub planning: &'a GraphPlanningSummary,
    /// Pre-computed contract summary.
    pub contract: &'a GraphContractSummary,
    /// Pre-computed routing summary.
    pub routing: &'a GraphRoutingSummary,
    /// Per-node render overrides (empty slice when none).
    pub node_render_overrides: &'a [GraphNodeRenderOverride],
    /// Bus IDs whose output should be captured and returned after execution.
    pub captured_bus_ids: &'a [String],
}

/// Peak and RMS levels for a single routing bus after a block is processed.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphBusLevelReport {
    /// Bus identifier.
    pub bus_id: String,
    /// Peak absolute sample value across the block.
    pub peak: f32,
    /// Root-mean-square level across the block.
    pub rms: f32,
    /// Latency accumulated on this bus, in samples.
    pub latency_samples: u32,
    /// Tail time accumulated on this bus, in samples.
    pub tail_samples: u32,
}
