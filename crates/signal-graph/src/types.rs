// Core type definitions for signal-graph
use signal_primitives::ChannelLayout;

mod contract_types;
mod execution_types;

pub use contract_types::*;
pub use execution_types::*;

/// Processing stage that can be chained inside a graph node.
///
/// Each variant encodes the stage kind and its declared parameter values.
#[derive(Clone, Debug, PartialEq)]
pub enum GraphStageSpec {
    /// Linear gain stage.
    Gain {
        /// Direct amplitude multiplier (1.0 = unity gain).
        linear: f32,
    },
    /// DC bias (constant additive offset).
    Bias {
        /// Value added to every sample.
        amount: f32,
    },
    /// Tanh soft-clip/drive stage.
    TanhDrive {
        /// Scales the signal before the tanh function.
        drive: f32,
    },
    /// Stereo balance pan.
    StereoBalance {
        /// Pan position: -1.0 = full left, 0.0 = centre, 1.0 = full right.
        balance: f32,
    },
    /// Hard clipping at a symmetric threshold.
    HardClip {
        /// Absolute sample value at which clipping occurs.
        threshold: f32,
    },
}

/// Execution class assigned to a graph node, governing scheduler decisions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphNodeExecutionClass {
    /// Stateless transform with no side-effects. Safe to skip on silent input.
    PureTransform,
    /// Maintains internal state across blocks (e.g. filters, envelopes).
    Stateful,
    /// Introduces an inherent processing latency (e.g. look-ahead limiters).
    LatencyBearing,
    /// Wraps an external plugin with its own state and latency reporting.
    PluginBacked,
}

/// Planning group that controls which execution lane a node is scheduled into.
///
/// Lane names model the scheduling intent of a hypothetical realtime/anticipative
/// split; in this engine all lanes execute synchronously on the calling thread.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphNodePlanningGroup {
    /// Lightweight, stateless node scheduled inline on the primary lane.
    InlineRealtime,
    /// Stateful node scheduled on the primary lane in a dedicated phase.
    StatefulRealtime,
    /// Node that may be pre-computed on the anticipative lane ahead of the
    /// primary lane.
    AnticipativeEligible,
}

/// The modeled lane on which a dispatch executes.
///
/// Lanes are a scheduling classification, not threads: both lanes run
/// synchronously on the caller's thread during [`crate::ExecutableGraph`]
/// execution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphExecutionLane {
    /// The primary lane, dispatched after any anticipative pre-work.
    Realtime,
    /// The anticipative (pre-computation) lane, dispatched before the
    /// primary lane.
    Anticipative,
}

/// Governs what the graph does when the input bus is detected as silent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphNodeSilencePolicy {
    /// Always process, even when the input is silent.
    Process,
    /// Pass the silent input through without running the node's stages.
    Bypass,
    /// Zero out the output buffer instead of running the node's stages.
    ClearOutput,
}

/// How the graph adapts when a node's input and output channel layouts differ.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphChannelAdaptationMode {
    /// Only accept identical layouts. Mismatches are flagged as contract issues.
    MatchOnly,
    /// Automatically up-mix mono→stereo or down-mix stereo→mono as needed.
    AdaptiveMonoStereo,
}

/// Controls when a node's internal state is reset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphNodeResetPolicy {
    /// State persists across every block and every graph rebuild.
    RetainAcrossBlocks,
    /// State is cleared when the graph is rebuilt (e.g. topology change).
    ResetOnGraphRebuild,
    /// State is cleared when transport stops.
    ResetOnTransportStop,
}

/// Semantic role of a node in the mixing topology, used for reporting and
/// routing decisions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphNodeTopologyRole {
    /// General-purpose utility node with no specific topology semantics.
    Utility,
    /// Node sitting on an individual track lane.
    TrackLane,
    /// Summing or distribution bus node.
    Bus,
    /// Sends audio from a track to an auxiliary bus.
    Send,
    /// Receives audio from a send and re-injects it into the main path.
    Return,
    /// Console-model node (e.g. strip or master bus in a console emulation).
    ConsoleNode,
}

/// One side of a node's buffer contract — identifies the bus and its channel format.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphNodeBusEndpoint {
    /// Identifier of the routing bus (e.g. `"main:in"`, `"fx1:out"`).
    pub bus_id: String,
    /// Channel layout expected on this bus endpoint.
    pub channels: ChannelLayout,
}

impl GraphNodeBusEndpoint {
    /// Construct an endpoint from a bus ID string and a channel layout.
    pub fn new(bus_id: impl Into<String>, channels: ChannelLayout) -> Self {
        Self {
            bus_id: bus_id.into(),
            channels,
        }
    }
}

/// Declares a node's buffer requirements: buses, scratch allocation, and
/// behavioural policies for silence, channel adaptation, and state resets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphNodeBufferContract {
    /// Input bus this node reads from.
    pub input: GraphNodeBusEndpoint,
    /// Output bus this node writes to.
    pub output: GraphNodeBusEndpoint,
    /// Number of temporary single-block scratch buffers the node requires.
    pub scratch_buffers: usize,
    /// What to do when the input is detected as silent.
    pub silence_policy: GraphNodeSilencePolicy,
    /// How to handle input/output channel layout mismatches.
    pub channel_adaptation: GraphChannelAdaptationMode,
    /// When to clear this node's internal state.
    pub reset_policy: GraphNodeResetPolicy,
}

impl Default for GraphNodeBufferContract {
    fn default() -> Self {
        use signal_primitives::ChannelLayout;
        Self {
            input: GraphNodeBusEndpoint::new("main:in", ChannelLayout::Stereo),
            output: GraphNodeBusEndpoint::new("main:out", ChannelLayout::Stereo),
            scratch_buffers: 0,
            silence_policy: GraphNodeSilencePolicy::Process,
            channel_adaptation: GraphChannelAdaptationMode::AdaptiveMonoStereo,
            reset_policy: GraphNodeResetPolicy::RetainAcrossBlocks,
        }
    }
}

/// Optional topology annotations for a node, used for routing classification
/// and per-group reporting.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GraphNodeTopologyMetadata {
    /// Semantic role, if the node occupies a specific topology position.
    pub role: Option<GraphNodeTopologyRole>,
    /// Track lane identifier for [`GraphNodeTopologyRole::TrackLane`] nodes.
    pub track_lane_id: Option<String>,
    /// Bus group identifier for [`GraphNodeTopologyRole::Bus`] nodes.
    pub bus_group_id: Option<String>,
    /// Console group identifier for [`GraphNodeTopologyRole::ConsoleNode`] nodes.
    pub console_group_id: Option<String>,
    /// Send/return pair identifier for [`GraphNodeTopologyRole::Send`] and
    /// [`GraphNodeTopologyRole::Return`] nodes.
    pub send_return_id: Option<String>,
}

/// Result of resolving a channel layout adaptation for a node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphChannelAdaptationResult {
    /// Input and output layouts match exactly — no conversion needed.
    Exact,
    /// Mono input was up-mixed to stereo output.
    MonoToStereo,
    /// Stereo input was down-mixed to mono output.
    StereoToMono,
    /// The requested adaptation is not supported by the mode in use.
    Unsupported,
}

/// Specification for a single node in an executable graph.
///
/// Combines identity, scheduling metadata, buffer routing contracts, topology
/// annotations, and the ordered list of DSP stages that the node runs each
/// block.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphNodeSpec {
    /// Unique string identifier for this node within the graph.
    pub node_id: String,
    /// Execution class that drives scheduler and lane assignment.
    pub execution_class: GraphNodeExecutionClass,
    /// Fixed algorithmic latency introduced by this node, in samples.
    pub latency_samples: u32,
    /// Tail time after the input goes silent, in samples.
    pub tail_samples: u32,
    /// Buffer routing and behavioural contracts for this node.
    pub buffer_contract: GraphNodeBufferContract,
    /// Optional topology role and grouping metadata.
    pub topology: GraphNodeTopologyMetadata,
    /// Ordered list of DSP stages to run on each processed block.
    pub stages: Vec<GraphStageSpec>,
}

/// Complete description of an executable graph: its identity and the ordered
/// list of nodes to process each block.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphExecutionPlan {
    /// Unique identifier for this graph instance.
    pub graph_id: String,
    /// Ordered sequence of nodes. Processing order follows the slice order
    /// unless the planner reorders for lane dispatch.
    pub nodes: Vec<GraphNodeSpec>,
}
