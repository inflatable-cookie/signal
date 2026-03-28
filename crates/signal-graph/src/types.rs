// Core type definitions for signal-graph
use signal_primitives::{AudioBuffer, ChannelLayout};

mod contract_types;
mod execution_types;

pub use contract_types::*;
pub use execution_types::*;

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

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GraphParameterBatch {
    /// Runtime remains authoritative for `epoch` assignment and for deciding
    /// which block the batch belongs to. Graph interprets `events` only as
    /// block-local sample offsets relative to the current processing block.
    pub epoch: u64,
    pub strategy: GraphParameterApplicationStrategy,
    pub events: Vec<GraphParameterEvent>,
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GraphDynamicStageStateModel {
    #[default]
    RebuiltPerBlock,
    RetainedAcrossBlocks,
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
