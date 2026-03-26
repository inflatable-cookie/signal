// Core type definitions for signal-graph
use signal_primitives::{AudioBuffer, ChannelLayout};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(pub u64);

pub trait AudioNode {
    fn process(&mut self, buffer: &mut AudioBuffer);
}

#[derive(Clone, Debug, PartialEq)]
pub enum GraphStageSpec {
    Gain { linear: f32 },
    Bias { amount: f32 },
    TanhDrive { drive: f32 },
    StereoBalance { balance: f32 },
    HardClip { threshold: f32 },
    LowPass { cutoff_hz: f32 },
    Delay { delay_samples: usize, feedback: f32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphStageParameter {
    GainLinear,
    BiasAmount,
    TanhDrive,
    StereoBalance,
    HardClipThreshold,
    LowPassCutoffHz,
    DelayFeedback,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphParameterTarget {
    pub node_id: String,
    pub stage_index: usize,
    pub parameter: GraphStageParameter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphParameterApplicationStrategy {
    SplitAtEvents { max_sub_blocks: usize },
}

impl Default for GraphParameterApplicationStrategy {
    fn default() -> Self {
        Self::SplitAtEvents { max_sub_blocks: 8 }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphParameterEvent {
    pub sample_offset: usize,
    pub target: GraphParameterTarget,
    pub value: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphParameterBatch {
    /// Runtime remains authoritative for `epoch` assignment and for deciding
    /// which block the batch belongs to. Graph interprets `events` only as
    /// block-local sample offsets relative to the current processing block.
    pub epoch: u64,
    pub strategy: GraphParameterApplicationStrategy,
    pub events: Vec<GraphParameterEvent>,
}

impl Default for GraphParameterBatch {
    fn default() -> Self {
        Self {
            epoch: 0,
            strategy: GraphParameterApplicationStrategy::default(),
            events: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphNodeExecutionClass {
    PureTransform,
    Stateful,
    LatencyBearing,
    PluginBacked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphNodePlanningGroup {
    InlineRealtime,
    StatefulRealtime,
    AnticipativeEligible,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphExecutionLane {
    Realtime,
    Anticipative,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphNodeSilencePolicy {
    Process,
    Bypass,
    ClearOutput,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphChannelAdaptationMode {
    MatchOnly,
    AdaptiveMonoStereo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphNodeResetPolicy {
    RetainAcrossBlocks,
    ResetOnGraphRebuild,
    ResetOnTransportStop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphNodeTopologyRole {
    Utility,
    TrackLane,
    Bus,
    Send,
    Return,
    ConsoleNode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphDynamicStageStateModel {
    RebuiltPerBlock,
    RetainedAcrossBlocks,
}

impl Default for GraphDynamicStageStateModel {
    fn default() -> Self {
        Self::RebuiltPerBlock
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphNodeBusEndpoint {
    pub bus_id: String,
    pub channels: ChannelLayout,
}

impl GraphNodeBusEndpoint {
    pub fn new(bus_id: impl Into<String>, channels: ChannelLayout) -> Self {
        Self {
            bus_id: bus_id.into(),
            channels,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphNodeBufferContract {
    pub input: GraphNodeBusEndpoint,
    pub output: GraphNodeBusEndpoint,
    pub scratch_buffers: usize,
    pub silence_policy: GraphNodeSilencePolicy,
    pub channel_adaptation: GraphChannelAdaptationMode,
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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GraphNodeTopologyMetadata {
    pub role: Option<GraphNodeTopologyRole>,
    pub track_lane_id: Option<String>,
    pub bus_group_id: Option<String>,
    pub console_group_id: Option<String>,
    pub send_return_id: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphChannelAdaptationResult {
    Exact,
    MonoToStereo,
    StereoToMono,
    Unsupported,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GraphContractIssue {
    EmptyInputBusId {
        node_id: String,
    },
    EmptyOutputBusId {
        node_id: String,
    },
    MissingInputBusProducer {
        node_id: String,
        bus_id: String,
    },
    UnsupportedForwardReference {
        node_id: String,
        bus_id: String,
    },
    UnsupportedChannelAdaptation {
        node_id: String,
        input: ChannelLayout,
        output: ChannelLayout,
        mode: GraphChannelAdaptationMode,
    },
    InconsistentOutputBusChannels {
        node_id: String,
        bus_id: String,
        expected: ChannelLayout,
        actual: ChannelLayout,
    },
    SendRequiresDistinctBuses {
        node_id: String,
    },
    ReturnRequiresDistinctBuses {
        node_id: String,
    },
    MissingTrackLaneId {
        node_id: String,
    },
    MissingBusGroupId {
        node_id: String,
    },
    MissingConsoleGroupId {
        node_id: String,
    },
    MissingSendReturnId {
        node_id: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphNodeContractSummary {
    pub node_id: String,
    pub input_bus_id: String,
    pub output_bus_id: String,
    pub input_channels: ChannelLayout,
    pub output_channels: ChannelLayout,
    pub silence_policy: GraphNodeSilencePolicy,
    pub channel_adaptation: GraphChannelAdaptationMode,
    pub adaptation_result: GraphChannelAdaptationResult,
    pub scratch_buffers: usize,
    pub reset_policy: GraphNodeResetPolicy,
    pub topology_role: GraphNodeTopologyRole,
    pub track_lane_id: Option<String>,
    pub bus_group_id: Option<String>,
    pub console_group_id: Option<String>,
    pub send_return_id: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GraphContractSummary {
    pub issue_count: usize,
    pub issues: Vec<GraphContractIssue>,
    pub silence_clear_node_count: usize,
    pub adaptive_channel_node_count: usize,
    pub resettable_node_count: usize,
    pub scratch_buffer_count: usize,
    pub track_lane_node_count: usize,
    pub bus_node_count: usize,
    pub send_return_node_count: usize,
    pub console_node_count: usize,
    pub node_contracts: Vec<GraphNodeContractSummary>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GraphRoutingSummary {
    pub routed_bus_count: usize,
    pub direct_edge_count: usize,
    pub fan_in_bus_count: usize,
    pub fan_out_bus_count: usize,
    pub mixed_bus_count: usize,
    pub output_latency_samples: u32,
    pub max_bus_latency_samples: u32,
    pub output_tail_samples: u32,
    pub max_bus_tail_samples: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphNodeSpec {
    pub node_id: String,
    pub execution_class: GraphNodeExecutionClass,
    pub latency_samples: u32,
    pub tail_samples: u32,
    pub buffer_contract: GraphNodeBufferContract,
    pub topology: GraphNodeTopologyMetadata,
    pub stages: Vec<GraphStageSpec>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphExecutionPlan {
    pub graph_id: String,
    pub nodes: Vec<GraphNodeSpec>,
}

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

#[derive(Clone, Debug, PartialEq)]
pub struct GraphBusLevelReport {
    pub bus_id: String,
    pub peak: f32,
    pub rms: f32,
    pub latency_samples: u32,
    pub tail_samples: u32,
}
