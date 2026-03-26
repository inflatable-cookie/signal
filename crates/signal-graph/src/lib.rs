//! Graph model and execution semantics for Signal.
//!
//! The crate owns the executable block path that sits between reusable DSP
//! kernels and runtime orchestration. It models node contracts, routed buses,
//! planning groups, execution lanes, and block-local parameter-event
//! application.
//!
//! ```no_run
//! use signal_graph::{
//!     synthetic_stereo_block, ExecutableGraph, GraphExecutionContext,
//! };
//! use signal_primitives::{FrameCount, SampleRate};
//!
//! let graph = ExecutableGraph::new("demo", Vec::new());
//! let mut buffer = synthetic_stereo_block(SampleRate(48_000), FrameCount(64), 1);
//! let report = graph.process_with_context(
//!     &mut buffer,
//!     GraphExecutionContext {
//!         configured_block_size: 64,
//!         ..GraphExecutionContext::default()
//!     },
//! );
//!
//! assert_eq!(report.graph_id, "demo");
//! assert_eq!(report.frame_count, 64);
//! ```

#[path = "graph_summary.rs"]
mod graph_summary;

use std::collections::{BTreeMap, BTreeSet};

use graph_summary::{classify_channel_adaptation, planning_group_for_node};
use signal_dsp::{
    process_delay_with_feedback_control, process_low_pass_with_cutoff_control, DelayLine,
    OnePoleLowPass,
};
use signal_primitives::{AudioBuffer, ChannelCount, ChannelLayout, FrameCount, SampleRate};

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

#[derive(Clone, Debug, PartialEq)]
pub struct ExecutableGraph {
    plan: GraphExecutionPlan,
}

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
    pub phase_order: Vec<GraphNodePlanningGroup>,
    pub lane_count: usize,
    pub anticipative_lane_count: usize,
    pub lane_order: Vec<GraphExecutionLane>,
    pub dispatch_count: usize,
    pub dispatch_boundary_count: usize,
    pub dispatch_order: Vec<GraphExecutionLane>,
    pub prepared_dispatch_count: usize,
    pub realtime_dispatch_count: usize,
    pub dispatch_handoff_count: usize,
    pub stage_count: usize,
    pub dynamic_kernel_stage_count: usize,
    pub dynamic_stage_state_model: GraphDynamicStageStateModel,
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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct GraphParameterApplicationReport {
    event_count: usize,
    targeted_node_count: usize,
    ignored_event_count: usize,
    sub_block_count: usize,
    coalesced_event_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct StageParameterEvent {
    sample_offset: usize,
    value: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct GraphBusState {
    buses: BTreeMap<String, AudioBuffer>,
    latencies: BTreeMap<String, u32>,
    tails: BTreeMap<String, u32>,
    silent_source_bus_count: usize,
}

impl ExecutableGraph {
    pub fn new(graph_id: impl Into<String>, nodes: Vec<GraphNodeSpec>) -> Self {
        Self {
            plan: GraphExecutionPlan {
                graph_id: graph_id.into(),
                nodes,
            },
        }
    }

    pub fn graph_id(&self) -> &str {
        self.plan.graph_id.as_str()
    }

    pub fn plan(&self) -> &GraphExecutionPlan {
        &self.plan
    }

    pub fn node_count(&self) -> usize {
        self.plan.nodes.len()
    }

    pub fn stage_count(&self) -> usize {
        self.plan.nodes.iter().map(|node| node.stages.len()).sum()
    }

    pub fn dynamic_kernel_stage_count(&self) -> usize {
        self.plan
            .nodes
            .iter()
            .flat_map(|node| node.stages.iter())
            .filter(|stage| {
                matches!(
                    stage,
                    GraphStageSpec::LowPass { .. } | GraphStageSpec::Delay { .. }
                )
            })
            .count()
    }

    pub fn dynamic_stage_state_model(&self) -> GraphDynamicStageStateModel {
        GraphDynamicStageStateModel::RebuiltPerBlock
    }

    pub fn stateful_node_count(&self) -> usize {
        self.plan
            .nodes
            .iter()
            .filter(|node| {
                matches!(
                    node.execution_class,
                    GraphNodeExecutionClass::Stateful
                        | GraphNodeExecutionClass::LatencyBearing
                        | GraphNodeExecutionClass::PluginBacked
                )
            })
            .count()
    }

    pub fn latency_node_count(&self) -> usize {
        self.plan
            .nodes
            .iter()
            .filter(|node| {
                matches!(
                    node.execution_class,
                    GraphNodeExecutionClass::LatencyBearing
                ) || node.latency_samples > 0
            })
            .count()
    }

    pub fn total_latency_samples(&self) -> u32 {
        self.plan
            .nodes
            .iter()
            .map(|node| node.latency_samples)
            .sum()
    }

    pub fn max_node_latency_samples(&self) -> u32 {
        self.plan
            .nodes
            .iter()
            .map(|node| node.latency_samples)
            .max()
            .unwrap_or(0)
    }

    pub fn total_tail_samples(&self) -> u32 {
        self.plan.nodes.iter().map(|node| node.tail_samples).sum()
    }

    pub fn max_node_tail_samples(&self) -> u32 {
        self.plan
            .nodes
            .iter()
            .map(|node| node.tail_samples)
            .max()
            .unwrap_or(0)
    }

    pub fn plugin_backed_node_count(&self) -> usize {
        self.plan
            .nodes
            .iter()
            .filter(|node| matches!(node.execution_class, GraphNodeExecutionClass::PluginBacked))
            .count()
    }

    pub fn execute(&self, request: GraphExecutionRequest) -> (AudioBuffer, GraphBlockReport) {
        let GraphExecutionRequest {
            context,
            mut buffer,
            parameter_batch,
        } = request;
        let report =
            self.process_with_parameter_batch(&mut buffer, context, parameter_batch.as_ref());
        (buffer, report)
    }

    pub fn process(&self, buffer: &mut AudioBuffer) -> GraphBlockReport {
        self.process_with_context(buffer, GraphExecutionContext::default())
    }

    pub fn process_with_context(
        &self,
        buffer: &mut AudioBuffer,
        context: GraphExecutionContext,
    ) -> GraphBlockReport {
        self.process_with_parameter_batch(buffer, context, None)
    }

    pub fn process_with_parameter_batch_and_node_overrides(
        &self,
        buffer: &mut AudioBuffer,
        context: GraphExecutionContext,
        parameter_batch: Option<&GraphParameterBatch>,
        node_render_overrides: &[GraphNodeRenderOverride],
    ) -> GraphBlockReport {
        let input_peak = peak_abs(buffer.samples());
        let planning = self.planning_summary(context.anticipative_enabled);
        let contract = self.contract_summary();
        let routing = self.routing_summary();
        let prepared = self.prepare_anticipative(buffer, &context, parameter_batch);
        let (working_buffer, report) = self.execute_realtime_from_prepared_with_node_overrides(
            buffer,
            input_peak,
            prepared,
            context,
            parameter_batch,
            &planning,
            &contract,
            &routing,
            node_render_overrides,
        );
        *buffer = working_buffer;
        report
    }

    pub fn process_with_parameter_batch(
        &self,
        buffer: &mut AudioBuffer,
        context: GraphExecutionContext,
        parameter_batch: Option<&GraphParameterBatch>,
    ) -> GraphBlockReport {
        self.process_with_parameter_batch_and_node_overrides(buffer, context, parameter_batch, &[])
    }

    pub fn prepare_anticipative(
        &self,
        buffer: &AudioBuffer,
        context: &GraphExecutionContext,
        parameter_batch: Option<&GraphParameterBatch>,
    ) -> Option<GraphPreparedDispatch> {
        let planning = self.planning_summary(context.anticipative_enabled);
        let anticipative_dispatches = planning
            .dispatches
            .iter()
            .filter(|dispatch| dispatch.lane == GraphExecutionLane::Anticipative)
            .collect::<Vec<_>>();
        if anticipative_dispatches.is_empty() {
            return None;
        }

        let mut prepared = seeded_bus_state(buffer);
        self.execute_dispatches(
            &mut prepared,
            &anticipative_dispatches,
            context.anticipative_enabled,
            parameter_batch,
            &[],
        );
        let latencies = prepared.latencies.clone();
        let tails = prepared.tails.clone();

        Some(GraphPreparedDispatch {
            output_peak: peak_abs_across_buses(&prepared),
            buses: prepared
                .buses
                .into_iter()
                .map(|(bus_id, buffer)| GraphPreparedBus {
                    latency_samples: latencies.get(&bus_id).copied().unwrap_or(0),
                    tail_samples: tails.get(&bus_id).copied().unwrap_or(0),
                    bus_id,
                    buffer,
                })
                .collect(),
            dispatch_count: anticipative_dispatches.len(),
        })
    }

    pub fn execute_realtime_from_prepared(
        &self,
        input: &AudioBuffer,
        input_peak: f32,
        prepared: Option<GraphPreparedDispatch>,
        context: GraphExecutionContext,
        parameter_batch: Option<&GraphParameterBatch>,
        planning: &GraphPlanningSummary,
        contract: &GraphContractSummary,
        routing: &GraphRoutingSummary,
    ) -> (AudioBuffer, GraphBlockReport) {
        self.execute_realtime_from_prepared_with_node_overrides(
            input,
            input_peak,
            prepared,
            context,
            parameter_batch,
            planning,
            contract,
            routing,
            &[],
        )
    }

    pub fn execute_realtime_from_prepared_with_node_overrides(
        &self,
        input: &AudioBuffer,
        input_peak: f32,
        prepared: Option<GraphPreparedDispatch>,
        context: GraphExecutionContext,
        parameter_batch: Option<&GraphParameterBatch>,
        planning: &GraphPlanningSummary,
        contract: &GraphContractSummary,
        routing: &GraphRoutingSummary,
        node_render_overrides: &[GraphNodeRenderOverride],
    ) -> (AudioBuffer, GraphBlockReport) {
        let (output, report, _) = self
            .execute_realtime_from_prepared_with_node_overrides_and_bus_captures(
                input,
                input_peak,
                prepared,
                context,
                parameter_batch,
                planning,
                contract,
                routing,
                node_render_overrides,
                &[],
            );
        (output, report)
    }

    pub fn execute_realtime_from_prepared_with_node_overrides_and_bus_captures(
        &self,
        input: &AudioBuffer,
        input_peak: f32,
        prepared: Option<GraphPreparedDispatch>,
        context: GraphExecutionContext,
        parameter_batch: Option<&GraphParameterBatch>,
        planning: &GraphPlanningSummary,
        contract: &GraphContractSummary,
        routing: &GraphRoutingSummary,
        node_render_overrides: &[GraphNodeRenderOverride],
        captured_bus_ids: &[String],
    ) -> (AudioBuffer, GraphBlockReport, Vec<GraphCapturedBusOutput>) {
        let realtime_dispatches = planning
            .dispatches
            .iter()
            .filter(|dispatch| dispatch.lane == GraphExecutionLane::Realtime)
            .collect::<Vec<_>>();

        let prework_output_peak = prepared.as_ref().map(|prepared| prepared.output_peak);
        let mut realtime_input_peak = prework_output_peak;
        let mut working_state = prepared
            .as_ref()
            .map(prepared_bus_state)
            .unwrap_or_else(|| seeded_bus_state(input));
        let parameter_report =
            parameter_application_report(&self.plan, input.frames().0, parameter_batch);

        if !realtime_dispatches.is_empty() {
            if prework_output_peak.is_none() {
                realtime_input_peak = Some(peak_abs(
                    graph_output_buffer(&working_state, input).samples(),
                ));
            }
            self.execute_dispatches(
                &mut working_state,
                &realtime_dispatches,
                context.anticipative_enabled,
                parameter_batch,
                node_render_overrides,
            );
        }

        let working_buffer = graph_output_buffer(&working_state, input);
        let output_latency_samples = working_state
            .latencies
            .get("main:out")
            .copied()
            .unwrap_or_else(|| routing.output_latency_samples);
        let max_bus_latency_samples = working_state
            .latencies
            .values()
            .copied()
            .max()
            .unwrap_or_else(|| routing.max_bus_latency_samples);
        let output_tail_samples = working_state
            .tails
            .get("main:out")
            .copied()
            .unwrap_or_else(|| routing.output_tail_samples);
        let max_bus_tail_samples = working_state
            .tails
            .values()
            .copied()
            .max()
            .unwrap_or_else(|| routing.max_bus_tail_samples);
        let bus_levels = working_state
            .buses
            .iter()
            .map(|(bus_id, buffer)| GraphBusLevelReport {
                bus_id: bus_id.clone(),
                peak: peak_abs(buffer.samples()),
                rms: rms(buffer.samples()),
                latency_samples: working_state.latencies.get(bus_id).copied().unwrap_or(0),
                tail_samples: working_state.tails.get(bus_id).copied().unwrap_or(0),
            })
            .collect::<Vec<_>>();
        let captured_buses = captured_bus_ids
            .iter()
            .filter_map(|bus_id| {
                working_state
                    .buses
                    .get(bus_id)
                    .cloned()
                    .map(|buffer| GraphCapturedBusOutput {
                        bus_id: bus_id.clone(),
                        buffer,
                    })
            })
            .collect::<Vec<_>>();

        (
            working_buffer.clone(),
            GraphBlockReport {
                graph_id: self.plan.graph_id.clone(),
                context,
                node_count: self.node_count(),
                stateful_node_count: self.stateful_node_count(),
                latency_node_count: self.latency_node_count(),
                plugin_backed_node_count: self.plugin_backed_node_count(),
                contract_issue_count: contract.issue_count,
                silence_clear_node_count: contract.silence_clear_node_count,
                adaptive_channel_node_count: contract.adaptive_channel_node_count,
                resettable_node_count: contract.resettable_node_count,
                scratch_buffer_count: contract.scratch_buffer_count,
                track_lane_node_count: contract.track_lane_node_count,
                bus_node_count: contract.bus_node_count,
                send_return_node_count: contract.send_return_node_count,
                console_node_count: contract.console_node_count,
                routed_bus_count: routing.routed_bus_count,
                direct_edge_count: routing.direct_edge_count,
                fan_in_bus_count: routing.fan_in_bus_count,
                fan_out_bus_count: routing.fan_out_bus_count,
                mixed_bus_count: routing.mixed_bus_count,
                silent_source_bus_count: working_state.silent_source_bus_count,
                phase_count: planning.phase_count,
                anticipative_phase_count: planning.anticipative_phase_count,
                phase_order: planning.phase_order.clone(),
                lane_count: planning.lane_count,
                anticipative_lane_count: planning.anticipative_lane_count,
                lane_order: planning.lane_order.clone(),
                dispatch_count: planning.dispatch_count,
                dispatch_boundary_count: planning.dispatch_boundary_count,
                dispatch_order: planning
                    .dispatches
                    .iter()
                    .map(|dispatch| dispatch.lane)
                    .collect(),
                prepared_dispatch_count: prepared
                    .as_ref()
                    .map_or(0, |prepared| prepared.dispatch_count),
                realtime_dispatch_count: realtime_dispatches.len(),
                dispatch_handoff_count: usize::from(
                    prepared.is_some() && !realtime_dispatches.is_empty(),
                ),
                stage_count: self.stage_count(),
                dynamic_kernel_stage_count: self.dynamic_kernel_stage_count(),
                dynamic_stage_state_model: self.dynamic_stage_state_model(),
                total_latency_samples: self.total_latency_samples(),
                max_node_latency_samples: self.max_node_latency_samples(),
                total_tail_samples: self.total_tail_samples(),
                max_node_tail_samples: self.max_node_tail_samples(),
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
                input_peak,
                prework_output_peak,
                realtime_input_peak,
                output_peak: peak_abs(working_buffer.samples()),
                output_rms: rms(working_buffer.samples()),
                bus_level_count: bus_levels.len(),
                bus_levels,
                first_output_sample: working_buffer.samples().first().copied(),
            },
            captured_buses,
        )
    }

    fn execute_dispatches(
        &self,
        state: &mut GraphBusState,
        dispatches: &[&GraphLaneDispatch],
        anticipative_enabled: bool,
        parameter_batch: Option<&GraphParameterBatch>,
        node_render_overrides: &[GraphNodeRenderOverride],
    ) {
        let node_render_overrides = node_render_override_map(node_render_overrides);
        for dispatch in dispatches {
            for phase in &dispatch.phase_order {
                for node in
                    self.plan.nodes.iter().filter(|node| {
                        planning_group_for_node(node, anticipative_enabled) == *phase
                    })
                {
                    let mut working = source_buffer_for_node(state, node);
                    let input_latency = state
                        .latencies
                        .get(&node.buffer_contract.input.bus_id)
                        .copied()
                        .unwrap_or(0);
                    let input_tail = state
                        .tails
                        .get(&node.buffer_contract.input.bus_id)
                        .copied()
                        .unwrap_or(0);
                    let input_was_silent = peak_abs(working.samples()) == 0.0;
                    if input_was_silent {
                        state.silent_source_bus_count += 1;
                    }
                    if let Some(node_render_override) =
                        node_render_overrides.get(node.node_id.as_str())
                    {
                        let output = if node_render_override.bypassed {
                            adapt_buffer_to_layout(
                                &working,
                                node.buffer_contract.output.channels,
                                node.buffer_contract.channel_adaptation,
                            )
                        } else {
                            adapt_buffer_to_layout(
                                &node_render_override.buffer,
                                node.buffer_contract.output.channels,
                                node.buffer_contract.channel_adaptation,
                            )
                        };
                        mix_buffer_into_bus(
                            state,
                            node.buffer_contract.output.bus_id.as_str(),
                            output,
                            input_latency.saturating_add(node_render_override.latency_samples),
                            input_tail.saturating_add(node_render_override.tail_samples),
                        );
                        continue;
                    }
                    if !apply_node_contract(&mut working, node) {
                        if node.buffer_contract.silence_policy
                            == GraphNodeSilencePolicy::ClearOutput
                        {
                            mix_buffer_into_bus(
                                state,
                                node.buffer_contract.output.bus_id.as_str(),
                                working,
                                input_latency.saturating_add(node.latency_samples),
                                input_tail.saturating_add(node.tail_samples),
                            );
                        }
                        continue;
                    }

                    for (stage_index, stage) in node.stages.iter().enumerate() {
                        let events = stage_parameter_events_for_node(
                            parameter_batch,
                            node,
                            stage_index,
                            stage,
                            working.frames().0,
                        );
                        apply_stage(
                            &mut working,
                            stage,
                            &events,
                            parameter_batch.map(|batch| batch.strategy),
                        );
                    }
                    let output = adapt_buffer_to_layout(
                        &working,
                        node.buffer_contract.output.channels,
                        node.buffer_contract.channel_adaptation,
                    );
                    mix_buffer_into_bus(
                        state,
                        node.buffer_contract.output.bus_id.as_str(),
                        output,
                        input_latency.saturating_add(node.latency_samples),
                        input_tail.saturating_add(node.tail_samples),
                    );
                }
            }
        }
    }
}

fn apply_stage(
    buffer: &mut AudioBuffer,
    stage: &GraphStageSpec,
    events: &[StageParameterEvent],
    strategy: Option<GraphParameterApplicationStrategy>,
) {
    let strategy = strategy.unwrap_or_default();
    let (events, _) = bounded_stage_events(events, strategy);
    let mut processor =
        GraphStageProcessor::new(stage, buffer.sample_rate(), buffer.channel_count().0);
    let mut frame_cursor = 0;
    let mut event_cursor = 0;

    while frame_cursor < buffer.frames().0 {
        while let Some(event) = events.get(event_cursor).copied() {
            if event.sample_offset != frame_cursor {
                break;
            }
            processor.set_parameter(event.value);
            event_cursor += 1;
        }

        let next_boundary = events
            .get(event_cursor)
            .map(|event| event.sample_offset)
            .unwrap_or(buffer.frames().0)
            .max(frame_cursor.saturating_add(1))
            .min(buffer.frames().0);
        let channel_count = buffer.channel_count().0;
        let sample_start = frame_cursor.saturating_mul(channel_count);
        let sample_end = next_boundary.saturating_mul(channel_count);
        processor.process_interleaved(
            &mut buffer.samples_mut()[sample_start..sample_end],
            channel_count,
        );
        frame_cursor = next_boundary;
    }
}

struct GraphStageProcessor {
    stage: GraphStageProcessorKind,
}

enum GraphStageProcessorKind {
    Gain {
        linear: f32,
    },
    Bias {
        amount: f32,
    },
    TanhDrive {
        drive: f32,
    },
    StereoBalance {
        balance: f32,
    },
    HardClip {
        threshold: f32,
    },
    LowPass {
        cutoff_hz: f32,
        filters: Vec<OnePoleLowPass>,
    },
    Delay {
        feedback: f32,
        delay_samples: usize,
        lines: Vec<DelayLine>,
    },
}

impl GraphStageProcessor {
    fn new(stage: &GraphStageSpec, sample_rate: SampleRate, channel_count: usize) -> Self {
        let stage = match *stage {
            GraphStageSpec::Gain { linear } => GraphStageProcessorKind::Gain { linear },
            GraphStageSpec::Bias { amount } => GraphStageProcessorKind::Bias { amount },
            GraphStageSpec::TanhDrive { drive } => GraphStageProcessorKind::TanhDrive { drive },
            GraphStageSpec::StereoBalance { balance } => {
                GraphStageProcessorKind::StereoBalance { balance }
            }
            GraphStageSpec::HardClip { threshold } => {
                GraphStageProcessorKind::HardClip { threshold }
            }
            GraphStageSpec::LowPass { cutoff_hz } => GraphStageProcessorKind::LowPass {
                cutoff_hz,
                filters: (0..channel_count)
                    .map(|_| {
                        OnePoleLowPass::new(sample_rate, signal_primitives::FrequencyHz(cutoff_hz))
                    })
                    .collect(),
            },
            GraphStageSpec::Delay {
                delay_samples,
                feedback,
            } => GraphStageProcessorKind::Delay {
                feedback,
                delay_samples,
                lines: (0..channel_count)
                    .map(|_| {
                        let mut delay = DelayLine::with_max_delay(delay_samples.max(1));
                        delay.set_delay_samples(delay_samples);
                        delay.set_feedback(feedback);
                        delay
                    })
                    .collect(),
            },
        };
        Self { stage }
    }

    fn set_parameter(&mut self, value: f32) {
        match &mut self.stage {
            GraphStageProcessorKind::Gain { linear } => *linear = value,
            GraphStageProcessorKind::Bias { amount } => *amount = value,
            GraphStageProcessorKind::TanhDrive { drive } => *drive = value,
            GraphStageProcessorKind::StereoBalance { balance } => *balance = value,
            GraphStageProcessorKind::HardClip { threshold } => *threshold = value.abs(),
            GraphStageProcessorKind::LowPass { cutoff_hz, filters } => {
                *cutoff_hz = value.max(0.0);
                for filter in filters {
                    filter.set_cutoff_hz(signal_primitives::FrequencyHz(*cutoff_hz));
                }
            }
            GraphStageProcessorKind::Delay {
                feedback, lines, ..
            } => {
                *feedback = value;
                for line in lines {
                    line.set_feedback(value);
                }
            }
        }
    }

    fn process_interleaved(&mut self, samples: &mut [f32], channel_count: usize) {
        match &mut self.stage {
            GraphStageProcessorKind::Gain { linear } => {
                for sample in samples {
                    *sample *= *linear;
                }
            }
            GraphStageProcessorKind::Bias { amount } => {
                for sample in samples {
                    *sample += *amount;
                }
            }
            GraphStageProcessorKind::TanhDrive { drive } => {
                let drive = drive.max(0.0);
                for sample in samples {
                    *sample = (*sample * drive).tanh();
                }
            }
            GraphStageProcessorKind::StereoBalance { balance } => {
                apply_stereo_balance_interleaved(samples, channel_count, *balance);
            }
            GraphStageProcessorKind::HardClip { threshold } => {
                let threshold = threshold.abs();
                for sample in samples {
                    *sample = sample.clamp(-threshold, threshold);
                }
            }
            GraphStageProcessorKind::LowPass { cutoff_hz, filters } => {
                process_low_pass_interleaved(samples, channel_count, filters, *cutoff_hz);
            }
            GraphStageProcessorKind::Delay {
                feedback,
                delay_samples,
                lines,
            } => {
                process_delay_interleaved(samples, channel_count, lines, *delay_samples, *feedback);
            }
        }
    }
}

fn seeded_bus_state(input: &AudioBuffer) -> GraphBusState {
    let mut buses = BTreeMap::new();
    let mut latencies = BTreeMap::new();
    let mut tails = BTreeMap::new();
    buses.insert("main:in".into(), input.clone());
    latencies.insert("main:in".into(), 0);
    tails.insert("main:in".into(), 0);
    GraphBusState {
        buses,
        latencies,
        tails,
        silent_source_bus_count: 0,
    }
}

fn prepared_bus_state(prepared: &GraphPreparedDispatch) -> GraphBusState {
    let mut buses = BTreeMap::new();
    let mut latencies = BTreeMap::new();
    let mut tails = BTreeMap::new();
    for bus in &prepared.buses {
        buses.insert(bus.bus_id.clone(), bus.buffer.clone());
        latencies.insert(bus.bus_id.clone(), bus.latency_samples);
        tails.insert(bus.bus_id.clone(), bus.tail_samples);
    }
    GraphBusState {
        buses,
        latencies,
        tails,
        silent_source_bus_count: 0,
    }
}

fn graph_output_buffer(state: &GraphBusState, fallback: &AudioBuffer) -> AudioBuffer {
    state.buses.get("main:out").cloned().unwrap_or_else(|| {
        if state.buses.len() == 1 && state.buses.contains_key("main:in") {
            fallback.clone()
        } else {
            AudioBuffer::new(
                fallback.sample_rate(),
                fallback.channels(),
                fallback.frames(),
            )
        }
    })
}

fn stage_parameter_events_for_node(
    parameter_batch: Option<&GraphParameterBatch>,
    node: &GraphNodeSpec,
    stage_index: usize,
    stage: &GraphStageSpec,
    frame_count: usize,
) -> Vec<StageParameterEvent> {
    let Some(parameter_batch) = parameter_batch else {
        return Vec::new();
    };

    let mut events = parameter_batch
        .events
        .iter()
        .filter(|event| {
            event.target.node_id == node.node_id
                && event.target.stage_index == stage_index
                && event.target.parameter.applies_to(stage)
                && event.sample_offset < frame_count
        })
        .map(|event| StageParameterEvent {
            sample_offset: event.sample_offset,
            value: event.value,
        })
        .collect::<Vec<_>>();
    events.sort_by_key(|event| event.sample_offset);
    events
}

fn bounded_stage_events(
    events: &[StageParameterEvent],
    strategy: GraphParameterApplicationStrategy,
) -> (Vec<StageParameterEvent>, usize) {
    match strategy {
        GraphParameterApplicationStrategy::SplitAtEvents { max_sub_blocks } => {
            let max_boundaries = max_sub_blocks.saturating_sub(1);
            if events.len() <= max_boundaries {
                return (events.to_vec(), 0);
            }

            if max_boundaries == 0 {
                let final_value = events.last().map(|event| event.value).unwrap_or(0.0);
                return (
                    vec![StageParameterEvent {
                        sample_offset: 0,
                        value: final_value,
                    }],
                    events.len(),
                );
            }

            let last_exact_index = max_boundaries.saturating_sub(1);
            let last_boundary = events[last_exact_index].sample_offset;
            let mut bounded = events[..max_boundaries].to_vec();
            if let Some(last) = bounded.last_mut() {
                last.value = events
                    .iter()
                    .skip(last_exact_index)
                    .last()
                    .map(|event| event.value)
                    .unwrap_or(last.value);
                last.sample_offset = last_boundary;
            }
            (bounded, events.len().saturating_sub(max_boundaries))
        }
    }
}

fn parameter_application_report(
    plan: &GraphExecutionPlan,
    frame_count: usize,
    parameter_batch: Option<&GraphParameterBatch>,
) -> GraphParameterApplicationReport {
    let Some(parameter_batch) = parameter_batch else {
        return GraphParameterApplicationReport::default();
    };

    let mut report = GraphParameterApplicationReport {
        event_count: parameter_batch.events.len(),
        ..GraphParameterApplicationReport::default()
    };
    let mut targeted_nodes = BTreeSet::new();

    for event in &parameter_batch.events {
        let Some(node) = plan
            .nodes
            .iter()
            .find(|node| node.node_id == event.target.node_id)
        else {
            report.ignored_event_count += 1;
            continue;
        };
        let Some(stage) = node.stages.get(event.target.stage_index) else {
            report.ignored_event_count += 1;
            continue;
        };
        if !event.target.parameter.applies_to(stage) || event.sample_offset >= frame_count {
            report.ignored_event_count += 1;
            continue;
        }
        targeted_nodes.insert(node.node_id.clone());
    }

    for node in &plan.nodes {
        for (stage_index, stage) in node.stages.iter().enumerate() {
            let stage_events = stage_parameter_events_for_node(
                Some(parameter_batch),
                node,
                stage_index,
                stage,
                frame_count,
            );
            if stage_events.is_empty() {
                continue;
            }
            let (bounded, coalesced) =
                bounded_stage_events(&stage_events, parameter_batch.strategy);
            let boundary_count = bounded
                .iter()
                .filter(|event| event.sample_offset > 0 && event.sample_offset < frame_count)
                .map(|event| event.sample_offset)
                .collect::<BTreeSet<_>>()
                .len();
            report.sub_block_count += boundary_count.saturating_add(1);
            report.coalesced_event_count += coalesced;
        }
    }

    report.targeted_node_count = targeted_nodes.len();
    report
}

fn peak_abs_across_buses(state: &GraphBusState) -> f32 {
    state
        .buses
        .values()
        .map(|buffer| peak_abs(buffer.samples()))
        .fold(0.0_f32, f32::max)
}

fn node_render_override_map(
    node_render_overrides: &[GraphNodeRenderOverride],
) -> BTreeMap<&str, &GraphNodeRenderOverride> {
    node_render_overrides
        .iter()
        .map(|node_render_override| (node_render_override.node_id.as_str(), node_render_override))
        .collect()
}

fn source_buffer_for_node(state: &GraphBusState, node: &GraphNodeSpec) -> AudioBuffer {
    let source = state
        .buses
        .get(&node.buffer_contract.input.bus_id)
        .cloned()
        .unwrap_or_else(|| {
            let fallback = state
                .buses
                .get("main:in")
                .or_else(|| state.buses.values().next());
            AudioBuffer::new(
                fallback
                    .map(|buffer| buffer.sample_rate())
                    .unwrap_or(SampleRate(48_000)),
                node.buffer_contract.input.channels,
                fallback
                    .map(|buffer| buffer.frames())
                    .unwrap_or(FrameCount(0)),
            )
        });
    adapt_buffer_to_layout(
        &source,
        node.buffer_contract.input.channels,
        node.buffer_contract.channel_adaptation,
    )
}

fn adapt_buffer_to_layout(
    input: &AudioBuffer,
    target_layout: ChannelLayout,
    mode: GraphChannelAdaptationMode,
) -> AudioBuffer {
    if input.channels() == target_layout {
        return input.clone();
    }

    match classify_channel_adaptation(input.channels(), target_layout, mode) {
        GraphChannelAdaptationResult::MonoToStereo => {
            let mono = input.samples();
            let mut samples = Vec::with_capacity(mono.len().saturating_mul(2));
            for sample in mono {
                samples.push(*sample);
                samples.push(*sample);
            }
            AudioBuffer::from_interleaved(input.sample_rate(), target_layout, samples)
        }
        GraphChannelAdaptationResult::StereoToMono => {
            AudioBuffer::from_interleaved(input.sample_rate(), target_layout, input.to_mono())
        }
        GraphChannelAdaptationResult::Exact | GraphChannelAdaptationResult::Unsupported => {
            AudioBuffer::new(input.sample_rate(), target_layout, input.frames())
        }
    }
}

fn mix_buffer_into_bus(
    state: &mut GraphBusState,
    bus_id: &str,
    mut buffer: AudioBuffer,
    latency: u32,
    tail: u32,
) {
    if let Some(existing) = state.buses.get_mut(bus_id) {
        if existing.channels() != buffer.channels() {
            buffer = adapt_buffer_to_layout(
                &buffer,
                existing.channels(),
                GraphChannelAdaptationMode::AdaptiveMonoStereo,
            );
        }
        for (dst, src) in existing.samples_mut().iter_mut().zip(buffer.samples()) {
            *dst += *src;
        }
        if let Some(existing_latency) = state.latencies.get_mut(bus_id) {
            *existing_latency = (*existing_latency).max(latency);
        }
        if let Some(existing_tail) = state.tails.get_mut(bus_id) {
            *existing_tail = (*existing_tail).max(tail);
        }
        return;
    }

    state.buses.insert(bus_id.to_string(), buffer);
    state.latencies.insert(bus_id.to_string(), latency);
    state.tails.insert(bus_id.to_string(), tail);
}

fn apply_node_contract(buffer: &mut AudioBuffer, node: &GraphNodeSpec) -> bool {
    let input_silent = peak_abs(buffer.samples()) == 0.0;
    if !input_silent {
        return true;
    }

    match node.buffer_contract.silence_policy {
        GraphNodeSilencePolicy::Process => true,
        GraphNodeSilencePolicy::Bypass => false,
        GraphNodeSilencePolicy::ClearOutput => {
            buffer.clear();
            false
        }
    }
}

impl GraphStageParameter {
    fn applies_to(self, stage: &GraphStageSpec) -> bool {
        matches!(
            (self, stage),
            (GraphStageParameter::GainLinear, GraphStageSpec::Gain { .. })
                | (GraphStageParameter::BiasAmount, GraphStageSpec::Bias { .. })
                | (
                    GraphStageParameter::TanhDrive,
                    GraphStageSpec::TanhDrive { .. }
                )
                | (
                    GraphStageParameter::StereoBalance,
                    GraphStageSpec::StereoBalance { .. }
                )
                | (
                    GraphStageParameter::HardClipThreshold,
                    GraphStageSpec::HardClip { .. }
                )
                | (
                    GraphStageParameter::LowPassCutoffHz,
                    GraphStageSpec::LowPass { .. }
                )
                | (
                    GraphStageParameter::DelayFeedback,
                    GraphStageSpec::Delay { .. }
                )
        )
    }
}

fn apply_stereo_balance_interleaved(samples: &mut [f32], channel_count: usize, balance: f32) {
    if channel_count != 2 {
        return;
    }

    let balance = balance.clamp(-1.0, 1.0);
    let left_gain = if balance >= 0.0 { 1.0 - balance } else { 1.0 };
    let right_gain = if balance <= 0.0 { 1.0 + balance } else { 1.0 };

    for frame in samples.chunks_exact_mut(channel_count) {
        frame[0] *= left_gain;
        frame[1] *= right_gain;
    }
}

fn process_low_pass_interleaved(
    samples: &mut [f32],
    channel_count: usize,
    filters: &mut [OnePoleLowPass],
    cutoff_hz: f32,
) {
    if channel_count == 0 {
        return;
    }

    for (channel_index, filter) in filters.iter_mut().enumerate().take(channel_count) {
        let mut mono = samples
            .chunks_exact(channel_count)
            .map(|frame| frame[channel_index])
            .collect::<Vec<_>>();
        let cutoff = vec![cutoff_hz; mono.len()];
        process_low_pass_with_cutoff_control(filter, &mut mono, &cutoff);
        for (frame, sample) in samples
            .chunks_exact_mut(channel_count)
            .zip(mono.into_iter())
        {
            frame[channel_index] = sample;
        }
    }
}

fn process_delay_interleaved(
    samples: &mut [f32],
    channel_count: usize,
    lines: &mut [DelayLine],
    delay_samples: usize,
    feedback: f32,
) {
    if channel_count == 0 {
        return;
    }

    for (channel_index, delay) in lines.iter_mut().enumerate().take(channel_count) {
        delay.set_delay_samples(delay_samples);
        let mut mono = samples
            .chunks_exact(channel_count)
            .map(|frame| frame[channel_index])
            .collect::<Vec<_>>();
        let feedback_block = vec![feedback; mono.len()];
        process_delay_with_feedback_control(delay, &mut mono, &feedback_block);
        for (frame, sample) in samples
            .chunks_exact_mut(channel_count)
            .zip(mono.into_iter())
        {
            frame[channel_index] = sample;
        }
    }
}

/// Graph-level execution config.
///
/// Today this is intentionally narrow and only carries block size because the
/// richer execution authority lives in [`GraphExecutionContext`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GraphConfig {
    pub block_size: usize,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self { block_size: 512 }
    }
}

pub fn synthetic_stereo_block(
    sample_rate: SampleRate,
    frames: FrameCount,
    seed: u64,
) -> AudioBuffer {
    let mut data = Vec::with_capacity(frames.0.saturating_mul(2));
    for frame in 0..frames.0 {
        let progress = frame as f32 / frames.0.max(1) as f32;
        let base = (seed as f32 * 0.03125) + (progress * 2.0 - 1.0);
        data.push(base);
        data.push(-base * 0.5);
    }
    AudioBuffer::from_interleaved(sample_rate, ChannelLayout::Count(ChannelCount(2)), data)
}

fn peak_abs(samples: &[f32]) -> f32 {
    samples
        .iter()
        .fold(0.0_f32, |peak, sample| peak.max(sample.abs()))
}

fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum = samples.iter().map(|sample| sample * sample).sum::<f32>();
    (sum / samples.len() as f32).sqrt()
}

#[cfg(test)]
mod tests {
    use signal_dsp::{
        process_delay_with_feedback_control, process_low_pass_with_cutoff_control, DelayLine,
        OnePoleLowPass,
    };

    use super::{
        synthetic_stereo_block, AudioBuffer, ChannelLayout, ExecutableGraph, FrameCount,
        GraphChannelAdaptationMode, GraphChannelAdaptationResult, GraphContractIssue,
        GraphDynamicStageStateModel, GraphExecutionContext, GraphExecutionLane,
        GraphExecutionRequest, GraphNodeBufferContract, GraphNodeBusEndpoint,
        GraphNodeExecutionClass, GraphNodePlanningGroup, GraphNodeRenderOverride,
        GraphNodeResetPolicy, GraphNodeSilencePolicy, GraphNodeSpec, GraphNodeTopologyMetadata,
        GraphNodeTopologyRole, GraphParameterApplicationStrategy, GraphParameterBatch,
        GraphParameterEvent, GraphParameterTarget, GraphStageParameter, GraphStageSpec, SampleRate,
    };

    fn test_node(
        node_id: &str,
        execution_class: GraphNodeExecutionClass,
        latency_samples: u32,
        stages: Vec<GraphStageSpec>,
    ) -> GraphNodeSpec {
        GraphNodeSpec {
            node_id: node_id.into(),
            execution_class,
            latency_samples,
            tail_samples: 0,
            buffer_contract: GraphNodeBufferContract::default(),
            topology: GraphNodeTopologyMetadata::default(),
            stages,
        }
    }

    fn routed_node(
        node_id: &str,
        input_bus: &str,
        input_channels: ChannelLayout,
        output_bus: &str,
        output_channels: ChannelLayout,
        role: GraphNodeTopologyRole,
        stages: Vec<GraphStageSpec>,
    ) -> GraphNodeSpec {
        let topology = match role {
            GraphNodeTopologyRole::Utility => GraphNodeTopologyMetadata {
                role: Some(role),
                track_lane_id: None,
                bus_group_id: None,
                console_group_id: None,
                send_return_id: None,
            },
            GraphNodeTopologyRole::TrackLane => GraphNodeTopologyMetadata {
                role: Some(role),
                track_lane_id: Some(node_id.into()),
                bus_group_id: None,
                console_group_id: None,
                send_return_id: None,
            },
            GraphNodeTopologyRole::Bus => GraphNodeTopologyMetadata {
                role: Some(role),
                track_lane_id: None,
                bus_group_id: Some(node_id.into()),
                console_group_id: None,
                send_return_id: None,
            },
            GraphNodeTopologyRole::Send | GraphNodeTopologyRole::Return => {
                GraphNodeTopologyMetadata {
                    role: Some(role),
                    track_lane_id: None,
                    bus_group_id: None,
                    console_group_id: None,
                    send_return_id: Some(node_id.into()),
                }
            }
            GraphNodeTopologyRole::ConsoleNode => GraphNodeTopologyMetadata {
                role: Some(role),
                track_lane_id: None,
                bus_group_id: None,
                console_group_id: Some(node_id.into()),
                send_return_id: None,
            },
        };
        GraphNodeSpec {
            buffer_contract: GraphNodeBufferContract {
                input: GraphNodeBusEndpoint::new(input_bus, input_channels),
                output: GraphNodeBusEndpoint::new(output_bus, output_channels),
                ..GraphNodeBufferContract::default()
            },
            topology,
            ..test_node(node_id, GraphNodeExecutionClass::Stateful, 0, stages)
        }
    }

    fn stage_event(
        node_id: &str,
        stage_index: usize,
        parameter: GraphStageParameter,
        sample_offset: usize,
        value: f32,
    ) -> GraphParameterEvent {
        GraphParameterEvent {
            sample_offset,
            target: GraphParameterTarget {
                node_id: node_id.into(),
                stage_index,
                parameter,
            },
            value,
        }
    }

    fn parameter_batch(events: Vec<GraphParameterEvent>) -> GraphParameterBatch {
        GraphParameterBatch {
            epoch: 7,
            strategy: GraphParameterApplicationStrategy::SplitAtEvents { max_sub_blocks: 8 },
            events,
        }
    }

    fn parameter_batch_with_strategy(
        events: Vec<GraphParameterEvent>,
        strategy: GraphParameterApplicationStrategy,
    ) -> GraphParameterBatch {
        GraphParameterBatch {
            epoch: 7,
            strategy,
            events,
        }
    }

    #[test]
    fn mono_mixdown_averages_channels() {
        let audio = AudioBuffer::from_interleaved(
            SampleRate(48_000),
            ChannelLayout::Stereo,
            vec![1.0, -1.0, 0.25, 0.75],
        );

        assert_eq!(audio.to_mono(), vec![0.0, 0.5]);
    }

    #[test]
    fn executable_graph_processes_buffer_and_reports_metrics() {
        let mut buffer = synthetic_stereo_block(SampleRate(48_000), FrameCount(4), 2);
        let graph = ExecutableGraph::new(
            "graph:test",
            vec![
                GraphNodeSpec {
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    ..routed_node(
                        "pre",
                        "main:in",
                        ChannelLayout::Stereo,
                        "bus:pre",
                        ChannelLayout::Stereo,
                        GraphNodeTopologyRole::TrackLane,
                        vec![
                            GraphStageSpec::Gain { linear: 0.5 },
                            GraphStageSpec::Bias { amount: 0.25 },
                            GraphStageSpec::TanhDrive { drive: 1.5 },
                        ],
                    )
                },
                GraphNodeSpec {
                    execution_class: GraphNodeExecutionClass::LatencyBearing,
                    latency_samples: 24,
                    ..routed_node(
                        "post",
                        "bus:pre",
                        ChannelLayout::Stereo,
                        "main:out",
                        ChannelLayout::Stereo,
                        GraphNodeTopologyRole::ConsoleNode,
                        vec![
                            GraphStageSpec::StereoBalance { balance: -0.25 },
                            GraphStageSpec::HardClip { threshold: 0.4 },
                        ],
                    )
                },
            ],
        );

        let report = graph.process(&mut buffer);

        assert_eq!(report.graph_id, "graph:test");
        assert_eq!(report.node_count, 2);
        assert_eq!(report.stateful_node_count, 1);
        assert_eq!(report.latency_node_count, 1);
        assert_eq!(report.contract_issue_count, 0);
        assert_eq!(report.silence_clear_node_count, 0);
        assert_eq!(report.adaptive_channel_node_count, 0);
        assert_eq!(report.resettable_node_count, 0);
        assert_eq!(report.scratch_buffer_count, 0);
        assert_eq!(report.direct_edge_count, 1);
        assert_eq!(report.fan_in_bus_count, 0);
        assert_eq!(report.fan_out_bus_count, 0);
        assert_eq!(report.phase_count, 2);
        assert_eq!(report.anticipative_phase_count, 0);
        assert_eq!(report.lane_count, 1);
        assert_eq!(report.anticipative_lane_count, 0);
        assert_eq!(report.lane_order, vec![GraphExecutionLane::Realtime]);
        assert_eq!(report.dispatch_count, 1);
        assert_eq!(report.dispatch_boundary_count, 0);
        assert_eq!(report.dispatch_order, vec![GraphExecutionLane::Realtime]);
        assert_eq!(report.prepared_dispatch_count, 0);
        assert_eq!(report.realtime_dispatch_count, 1);
        assert_eq!(report.dispatch_handoff_count, 0);
        assert_eq!(report.prework_output_peak, None);
        assert!(report.realtime_input_peak.is_some());
        assert_eq!(
            report.phase_order,
            vec![
                GraphNodePlanningGroup::InlineRealtime,
                GraphNodePlanningGroup::StatefulRealtime,
            ]
        );
        assert_eq!(report.stage_count, 5);
        assert_eq!(report.dynamic_kernel_stage_count, 0);
        assert_eq!(
            report.dynamic_stage_state_model,
            GraphDynamicStageStateModel::RebuiltPerBlock
        );
        assert_eq!(report.total_latency_samples, 24);
        assert_eq!(report.max_node_latency_samples, 24);
        assert_eq!(report.total_tail_samples, 0);
        assert_eq!(report.max_node_tail_samples, 0);
        assert_eq!(report.output_latency_samples, 24);
        assert_eq!(report.max_bus_latency_samples, 24);
        assert_eq!(report.frame_count, 4);
        assert_eq!(report.channel_count, 2);
        assert!(report.output_peak <= 0.4);
        assert!(report.output_rms > 0.0);
        assert!(report.first_output_sample.is_some());
    }

    #[test]
    fn stereo_balance_stage_scales_channels_as_expected() {
        let mut buffer = AudioBuffer::from_interleaved(
            SampleRate(48_000),
            ChannelLayout::Stereo,
            vec![1.0, 1.0, 0.5, 0.5],
        );
        let graph = ExecutableGraph::new(
            "graph:stereo-balance",
            vec![test_node(
                "balance",
                GraphNodeExecutionClass::PureTransform,
                0,
                vec![GraphStageSpec::StereoBalance { balance: 0.5 }],
            )],
        );

        let report = graph.process(&mut buffer);

        assert_eq!(buffer.samples(), &[0.5, 1.0, 0.25, 0.5]);
        assert_eq!(report.node_count, 1);
        assert_eq!(report.phase_count, 1);
        assert_eq!(report.lane_count, 1);
        assert_eq!(report.dispatch_count, 1);
        assert_eq!(report.prepared_dispatch_count, 0);
        assert_eq!(report.realtime_dispatch_count, 1);
        assert_eq!(report.dispatch_handoff_count, 0);
        assert_eq!(report.stage_count, 1);
    }

    #[test]
    fn executable_graph_carries_execution_context() {
        let graph = ExecutableGraph::new(
            "graph:context",
            vec![test_node(
                "gain",
                GraphNodeExecutionClass::Stateful,
                0,
                vec![GraphStageSpec::Gain { linear: 0.5 }],
            )],
        );
        let context = GraphExecutionContext {
            processing_epoch: 3,
            block_sequence: 17,
            projection_epoch: 2,
            parameter_epoch: 23,
            configured_block_size: 256,
            anticipative_enabled: true,
            transport_playing: true,
            transport_tempo_bpm: 128.0,
            timeline_position_samples: 512,
        };

        let (_buffer, report) = graph.execute(GraphExecutionRequest {
            context: context.clone(),
            buffer: synthetic_stereo_block(SampleRate(48_000), FrameCount(4), 4),
            parameter_batch: None,
        });

        assert_eq!(report.context, context);
        assert_eq!(report.graph_id, "graph:context");
        assert_eq!(report.node_count, 1);
        assert_eq!(report.stateful_node_count, 1);
        assert_eq!(report.prepared_dispatch_count, 0);
        assert_eq!(report.realtime_dispatch_count, 1);
        assert_eq!(report.dispatch_handoff_count, 0);
    }

    #[test]
    fn gain_parameter_events_split_block_and_update_report() {
        let graph = ExecutableGraph::new(
            "graph:param-gain",
            vec![routed_node(
                "gain",
                "main:in",
                ChannelLayout::Mono,
                "main:out",
                ChannelLayout::Mono,
                GraphNodeTopologyRole::Utility,
                vec![GraphStageSpec::Gain { linear: 0.0 }],
            )],
        );
        let mut buffer =
            AudioBuffer::from_interleaved(SampleRate(48_000), ChannelLayout::Mono, vec![1.0; 6]);
        let batch = parameter_batch(vec![
            stage_event("gain", 0, GraphStageParameter::GainLinear, 2, 0.5),
            stage_event("gain", 0, GraphStageParameter::GainLinear, 4, 1.0),
        ]);

        let report = graph.process_with_parameter_batch(
            &mut buffer,
            GraphExecutionContext::default(),
            Some(&batch),
        );

        assert_eq!(buffer.samples(), &[0.0, 0.0, 0.5, 0.5, 1.0, 1.0]);
        assert_eq!(report.parameter_epoch, Some(7));
        assert_eq!(report.parameter_event_count, 2);
        assert_eq!(report.parameter_targeted_node_count, 1);
        assert_eq!(report.parameter_ignored_event_count, 0);
        assert_eq!(report.parameter_sub_block_count, 3);
        assert_eq!(report.parameter_coalesced_event_count, 0);
    }

    #[test]
    fn low_pass_parameter_events_follow_bounded_sub_blocks() {
        let graph = ExecutableGraph::new(
            "graph:param-lowpass",
            vec![routed_node(
                "filter",
                "main:in",
                ChannelLayout::Mono,
                "main:out",
                ChannelLayout::Mono,
                GraphNodeTopologyRole::Utility,
                vec![GraphStageSpec::LowPass { cutoff_hz: 200.0 }],
            )],
        );
        let mut buffer = AudioBuffer::from_interleaved(
            SampleRate(48_000),
            ChannelLayout::Mono,
            vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        );
        let batch = parameter_batch(vec![stage_event(
            "filter",
            0,
            GraphStageParameter::LowPassCutoffHz,
            3,
            4_000.0,
        )]);

        let report = graph.process_with_parameter_batch(
            &mut buffer,
            GraphExecutionContext::default(),
            Some(&batch),
        );

        let mut expected = vec![1.0, 0.0, 0.0];
        let mut filter =
            OnePoleLowPass::new(SampleRate(48_000), signal_primitives::FrequencyHz(200.0));
        process_low_pass_with_cutoff_control(&mut filter, &mut expected, &[200.0; 3]);
        let mut tail = vec![0.0, 0.0, 0.0];
        process_low_pass_with_cutoff_control(&mut filter, &mut tail, &[4_000.0; 3]);
        expected.extend(tail);

        assert_eq!(buffer.samples().len(), expected.len());
        for (actual, expected) in buffer.samples().iter().zip(expected.iter()) {
            assert!((actual - expected).abs() < 1.0e-5);
        }
        assert_eq!(report.parameter_sub_block_count, 2);
    }

    #[test]
    fn dense_parameter_batches_are_coalesced_by_max_sub_block_budget() {
        let graph = ExecutableGraph::new(
            "graph:param-coalesced",
            vec![routed_node(
                "gain",
                "main:in",
                ChannelLayout::Mono,
                "main:out",
                ChannelLayout::Mono,
                GraphNodeTopologyRole::Utility,
                vec![GraphStageSpec::Gain { linear: 0.0 }],
            )],
        );
        let mut buffer =
            AudioBuffer::from_interleaved(SampleRate(48_000), ChannelLayout::Mono, vec![1.0; 6]);
        let batch = parameter_batch_with_strategy(
            vec![
                stage_event("gain", 0, GraphStageParameter::GainLinear, 1, 0.2),
                stage_event("gain", 0, GraphStageParameter::GainLinear, 2, 0.4),
                stage_event("gain", 0, GraphStageParameter::GainLinear, 3, 0.6),
                stage_event("gain", 0, GraphStageParameter::GainLinear, 4, 0.8),
            ],
            GraphParameterApplicationStrategy::SplitAtEvents { max_sub_blocks: 3 },
        );

        let report = graph.process_with_parameter_batch(
            &mut buffer,
            GraphExecutionContext::default(),
            Some(&batch),
        );

        assert_eq!(buffer.samples(), &[0.0, 0.2, 0.8, 0.8, 0.8, 0.8]);
        assert_eq!(report.parameter_sub_block_count, 3);
        assert_eq!(report.parameter_coalesced_event_count, 2);
    }

    #[test]
    fn delay_parameter_events_drive_feedback_changes_within_block() {
        let graph = ExecutableGraph::new(
            "graph:param-delay",
            vec![GraphNodeSpec {
                execution_class: GraphNodeExecutionClass::LatencyBearing,
                ..routed_node(
                    "delay",
                    "main:in",
                    ChannelLayout::Mono,
                    "main:out",
                    ChannelLayout::Mono,
                    GraphNodeTopologyRole::Send,
                    vec![GraphStageSpec::Delay {
                        delay_samples: 2,
                        feedback: 0.0,
                    }],
                )
            }],
        );
        let mut buffer = AudioBuffer::from_interleaved(
            SampleRate(48_000),
            ChannelLayout::Mono,
            vec![1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        );
        let batch = parameter_batch(vec![stage_event(
            "delay",
            0,
            GraphStageParameter::DelayFeedback,
            3,
            0.5,
        )]);

        let report = graph.process_with_parameter_batch(
            &mut buffer,
            GraphExecutionContext::default(),
            Some(&batch),
        );

        let mut expected = vec![1.0, 0.0, 0.0];
        let mut delay = DelayLine::with_max_delay(2);
        delay.set_delay_samples(2);
        process_delay_with_feedback_control(&mut delay, &mut expected, &[0.0; 3]);
        let mut tail = vec![0.0, 0.0, 0.0];
        process_delay_with_feedback_control(&mut delay, &mut tail, &[0.5; 3]);
        expected.extend(tail);

        assert_eq!(buffer.samples().len(), expected.len());
        for (actual, expected) in buffer.samples().iter().zip(expected.iter()) {
            assert!((actual - expected).abs() < 1.0e-6);
        }
        assert_eq!(report.parameter_sub_block_count, 2);
    }

    #[test]
    fn low_pass_dynamic_stage_rebuilds_state_across_blocks_currently() {
        let graph = ExecutableGraph::new(
            "graph:lowpass-reset",
            vec![routed_node(
                "filter",
                "main:in",
                ChannelLayout::Mono,
                "main:out",
                ChannelLayout::Mono,
                GraphNodeTopologyRole::Utility,
                vec![GraphStageSpec::LowPass { cutoff_hz: 200.0 }],
            )],
        );

        let mut first = AudioBuffer::from_interleaved(
            SampleRate(48_000),
            ChannelLayout::Mono,
            vec![1.0, 0.0, 0.0, 0.0],
        );
        let mut second = AudioBuffer::from_interleaved(
            SampleRate(48_000),
            ChannelLayout::Mono,
            vec![0.0, 0.0, 0.0, 0.0],
        );

        let first_report = graph.process(&mut first);
        let second_report = graph.process(&mut second);

        assert!(first.samples().iter().any(|sample| sample.abs() > 0.0));
        assert_eq!(second.samples(), &[0.0, 0.0, 0.0, 0.0]);
        assert_eq!(first_report.dynamic_kernel_stage_count, 1);
        assert_eq!(
            second_report.dynamic_stage_state_model,
            GraphDynamicStageStateModel::RebuiltPerBlock
        );
    }

    #[test]
    fn delay_dynamic_stage_rebuilds_state_across_blocks_currently() {
        let graph = ExecutableGraph::new(
            "graph:delay-reset",
            vec![routed_node(
                "delay",
                "main:in",
                ChannelLayout::Mono,
                "main:out",
                ChannelLayout::Mono,
                GraphNodeTopologyRole::Send,
                vec![GraphStageSpec::Delay {
                    delay_samples: 2,
                    feedback: 0.5,
                }],
            )],
        );

        let mut first = AudioBuffer::from_interleaved(
            SampleRate(48_000),
            ChannelLayout::Mono,
            vec![1.0, 0.0, 0.0, 0.0],
        );
        let mut second = AudioBuffer::from_interleaved(
            SampleRate(48_000),
            ChannelLayout::Mono,
            vec![0.0, 0.0, 0.0, 0.0],
        );

        graph.process(&mut first);
        let second_report = graph.process(&mut second);

        assert_eq!(second.samples(), &[0.0, 0.0, 0.0, 0.0]);
        assert_eq!(second_report.dynamic_kernel_stage_count, 1);
        assert_eq!(
            second_report.dynamic_stage_state_model,
            GraphDynamicStageStateModel::RebuiltPerBlock
        );
    }

    #[test]
    fn latency_nodes_become_anticipative_candidates_when_enabled() {
        let graph = ExecutableGraph::new(
            "graph:planning",
            vec![
                test_node(
                    "inline",
                    GraphNodeExecutionClass::PureTransform,
                    0,
                    vec![GraphStageSpec::Gain { linear: 1.0 }],
                ),
                test_node(
                    "latency",
                    GraphNodeExecutionClass::LatencyBearing,
                    32,
                    vec![GraphStageSpec::HardClip { threshold: 0.8 }],
                ),
            ],
        );

        let anticipative = graph.planning_summary(true);
        let realtime_only = graph.planning_summary(false);

        assert_eq!(anticipative.inline_realtime_node_count, 1);
        assert_eq!(anticipative.anticipative_eligible_node_count, 1);
        assert_eq!(anticipative.plugin_backed_node_count, 0);
        assert_eq!(anticipative.phase_count, 2);
        assert_eq!(anticipative.anticipative_phase_count, 1);
        assert_eq!(anticipative.lane_count, 2);
        assert_eq!(anticipative.anticipative_lane_count, 1);
        assert_eq!(
            anticipative.lane_order,
            vec![
                GraphExecutionLane::Anticipative,
                GraphExecutionLane::Realtime
            ]
        );
        assert_eq!(anticipative.dispatch_count, 2);
        assert_eq!(anticipative.dispatch_boundary_count, 1);
        assert_eq!(
            anticipative.phase_order,
            vec![
                GraphNodePlanningGroup::InlineRealtime,
                GraphNodePlanningGroup::AnticipativeEligible,
            ]
        );
        assert_eq!(
            anticipative.planned_nodes[1].group,
            GraphNodePlanningGroup::AnticipativeEligible
        );
        assert_eq!(realtime_only.stateful_realtime_node_count, 1);
        assert_eq!(realtime_only.lane_count, 1);
        assert_eq!(realtime_only.anticipative_lane_count, 0);
        assert_eq!(realtime_only.lane_order, vec![GraphExecutionLane::Realtime]);
        assert_eq!(realtime_only.dispatch_count, 1);
        assert_eq!(realtime_only.dispatch_boundary_count, 0);
        assert_eq!(
            realtime_only.phase_order,
            vec![
                GraphNodePlanningGroup::InlineRealtime,
                GraphNodePlanningGroup::StatefulRealtime,
            ]
        );
        assert_eq!(
            realtime_only.planned_nodes[1].group,
            GraphNodePlanningGroup::StatefulRealtime
        );
    }

    #[test]
    fn plugin_backed_nodes_remain_realtime_and_are_counted_in_planning() {
        let graph = ExecutableGraph::new(
            "graph:planning:plugin-backed",
            vec![
                test_node(
                    "inline",
                    GraphNodeExecutionClass::PureTransform,
                    0,
                    vec![GraphStageSpec::Gain { linear: 1.0 }],
                ),
                test_node(
                    "plugin",
                    GraphNodeExecutionClass::PluginBacked,
                    0,
                    vec![GraphStageSpec::HardClip { threshold: 0.8 }],
                ),
                test_node(
                    "latency",
                    GraphNodeExecutionClass::LatencyBearing,
                    32,
                    vec![GraphStageSpec::HardClip { threshold: 0.4 }],
                ),
            ],
        );

        let planning = graph.planning_summary(true);

        assert_eq!(planning.plugin_backed_node_count, 1);
        assert_eq!(planning.stateful_realtime_node_count, 1);
        assert_eq!(planning.anticipative_eligible_node_count, 1);
        assert_eq!(
            planning.planned_nodes[1].execution_class,
            GraphNodeExecutionClass::PluginBacked
        );
        assert_eq!(
            planning.planned_nodes[1].group,
            GraphNodePlanningGroup::StatefulRealtime
        );
    }

    #[test]
    fn plugin_render_override_injects_external_output_and_updates_routing_metrics() {
        let graph = ExecutableGraph::new(
            "graph:plugin:override",
            vec![GraphNodeSpec {
                execution_class: GraphNodeExecutionClass::PluginBacked,
                ..routed_node(
                    "plugin",
                    "main:in",
                    ChannelLayout::Stereo,
                    "main:out",
                    ChannelLayout::Stereo,
                    GraphNodeTopologyRole::TrackLane,
                    vec![GraphStageSpec::HardClip { threshold: 0.25 }],
                )
            }],
        );
        let mut buffer = AudioBuffer::new(SampleRate(48_000), ChannelLayout::Stereo, FrameCount(4));
        let override_buffer = AudioBuffer::from_interleaved(
            SampleRate(48_000),
            ChannelLayout::Stereo,
            vec![0.75, -0.5, 0.5, -0.25, 0.25, -0.125, 0.125, -0.0625],
        );

        let report = graph.process_with_parameter_batch_and_node_overrides(
            &mut buffer,
            GraphExecutionContext::default(),
            None,
            &[GraphNodeRenderOverride {
                node_id: "plugin".into(),
                buffer: override_buffer.clone(),
                latency_samples: 32,
                tail_samples: 48,
                bypassed: false,
            }],
        );

        assert_eq!(buffer, override_buffer);
        assert_eq!(report.output_latency_samples, 32);
        assert_eq!(report.max_bus_latency_samples, 32);
        assert_eq!(report.output_tail_samples, 48);
        assert_eq!(report.max_bus_tail_samples, 48);
    }

    #[test]
    fn plugin_render_override_bypass_uses_graph_input_instead_of_fallback_stage() {
        let graph = ExecutableGraph::new(
            "graph:plugin:bypass",
            vec![GraphNodeSpec {
                execution_class: GraphNodeExecutionClass::PluginBacked,
                ..routed_node(
                    "plugin",
                    "main:in",
                    ChannelLayout::Mono,
                    "main:out",
                    ChannelLayout::Mono,
                    GraphNodeTopologyRole::TrackLane,
                    vec![GraphStageSpec::HardClip { threshold: 0.5 }],
                )
            }],
        );
        let input =
            AudioBuffer::from_interleaved(SampleRate(48_000), ChannelLayout::Mono, vec![0.9; 4]);
        let override_buffer =
            AudioBuffer::from_interleaved(SampleRate(48_000), ChannelLayout::Mono, vec![0.1; 4]);

        let mut bypassed = input.clone();
        graph.process_with_parameter_batch_and_node_overrides(
            &mut bypassed,
            GraphExecutionContext::default(),
            None,
            &[GraphNodeRenderOverride {
                node_id: "plugin".into(),
                buffer: override_buffer,
                latency_samples: 0,
                tail_samples: 0,
                bypassed: true,
            }],
        );

        let mut fallback = input.clone();
        graph.process(&mut fallback);

        assert_eq!(bypassed.samples(), &[0.9, 0.9, 0.9, 0.9]);
        assert_eq!(fallback.samples(), &[0.5, 0.5, 0.5, 0.5]);
    }

    #[test]
    fn anticipative_dispatch_prepares_buffer_before_realtime_pass() {
        let mut buffer = synthetic_stereo_block(SampleRate(48_000), FrameCount(4), 7);
        let graph = ExecutableGraph::new(
            "graph:prework",
            vec![
                GraphNodeSpec {
                    execution_class: GraphNodeExecutionClass::LatencyBearing,
                    latency_samples: 16,
                    ..routed_node(
                        "anticipative",
                        "main:in",
                        ChannelLayout::Stereo,
                        "bus:preworked",
                        ChannelLayout::Stereo,
                        GraphNodeTopologyRole::Bus,
                        vec![GraphStageSpec::HardClip { threshold: 0.2 }],
                    )
                },
                routed_node(
                    "realtime",
                    "bus:preworked",
                    ChannelLayout::Stereo,
                    "main:out",
                    ChannelLayout::Stereo,
                    GraphNodeTopologyRole::ConsoleNode,
                    vec![GraphStageSpec::Gain { linear: 0.5 }],
                ),
            ],
        );

        let report = graph.process_with_context(
            &mut buffer,
            GraphExecutionContext {
                anticipative_enabled: true,
                ..GraphExecutionContext::default()
            },
        );

        assert_eq!(report.prepared_dispatch_count, 1);
        assert_eq!(report.realtime_dispatch_count, 1);
        assert_eq!(report.dispatch_handoff_count, 1);
        assert_eq!(
            report.dispatch_order,
            vec![
                GraphExecutionLane::Anticipative,
                GraphExecutionLane::Realtime
            ]
        );
        assert!(report.prework_output_peak.is_some());
        assert_eq!(report.prework_output_peak, report.realtime_input_peak);
        assert!(report.output_peak <= report.prework_output_peak.unwrap_or_default());
    }

    #[test]
    fn contract_summary_surfaces_topology_and_adaptation_metadata() {
        let graph = ExecutableGraph::new(
            "graph:contract",
            vec![
                GraphNodeSpec {
                    buffer_contract: GraphNodeBufferContract {
                        input: GraphNodeBusEndpoint::new("main:in", ChannelLayout::Mono),
                        output: GraphNodeBusEndpoint::new("bus:mix", ChannelLayout::Stereo),
                        scratch_buffers: 2,
                        silence_policy: GraphNodeSilencePolicy::ClearOutput,
                        channel_adaptation: GraphChannelAdaptationMode::AdaptiveMonoStereo,
                        reset_policy: GraphNodeResetPolicy::ResetOnTransportStop,
                    },
                    topology: GraphNodeTopologyMetadata {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: Some("track:vox".into()),
                        bus_group_id: Some("mix".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                    ..test_node(
                        "track",
                        GraphNodeExecutionClass::Stateful,
                        0,
                        vec![GraphStageSpec::Gain { linear: 1.0 }],
                    )
                },
                GraphNodeSpec {
                    buffer_contract: GraphNodeBufferContract {
                        input: GraphNodeBusEndpoint::new("bus:mix", ChannelLayout::Stereo),
                        output: GraphNodeBusEndpoint::new("console:main", ChannelLayout::Stereo),
                        ..GraphNodeBufferContract::default()
                    },
                    topology: GraphNodeTopologyMetadata {
                        role: Some(GraphNodeTopologyRole::ConsoleNode),
                        track_lane_id: None,
                        bus_group_id: None,
                        console_group_id: Some("console:main".into()),
                        send_return_id: None,
                    },
                    ..test_node(
                        "console",
                        GraphNodeExecutionClass::PureTransform,
                        0,
                        vec![GraphStageSpec::HardClip { threshold: 0.8 }],
                    )
                },
            ],
        );

        let summary = graph.contract_summary();

        assert_eq!(summary.issue_count, 0);
        assert_eq!(summary.silence_clear_node_count, 1);
        assert_eq!(summary.adaptive_channel_node_count, 1);
        assert_eq!(summary.resettable_node_count, 1);
        assert_eq!(summary.scratch_buffer_count, 2);
        assert_eq!(summary.track_lane_node_count, 1);
        assert_eq!(summary.console_node_count, 1);
        assert_eq!(
            summary.node_contracts[0].adaptation_result,
            GraphChannelAdaptationResult::MonoToStereo
        );
        assert_eq!(graph.routing_summary().output_tail_samples, 0);
    }

    #[test]
    fn contract_summary_rejects_send_nodes_that_loop_back_to_same_bus() {
        let graph = ExecutableGraph::new(
            "graph:send-loop",
            vec![
                routed_node(
                    "track",
                    "main:in",
                    ChannelLayout::Stereo,
                    "bus:drums",
                    ChannelLayout::Stereo,
                    GraphNodeTopologyRole::TrackLane,
                    vec![],
                ),
                GraphNodeSpec {
                    buffer_contract: GraphNodeBufferContract {
                        input: GraphNodeBusEndpoint::new("bus:drums", ChannelLayout::Stereo),
                        output: GraphNodeBusEndpoint::new("bus:drums", ChannelLayout::Stereo),
                        ..GraphNodeBufferContract::default()
                    },
                    topology: GraphNodeTopologyMetadata {
                        role: Some(GraphNodeTopologyRole::Send),
                        track_lane_id: None,
                        bus_group_id: None,
                        console_group_id: None,
                        send_return_id: Some("fx".into()),
                    },
                    ..test_node(
                        "send",
                        GraphNodeExecutionClass::Stateful,
                        0,
                        vec![GraphStageSpec::Gain { linear: 0.5 }],
                    )
                },
            ],
        );

        let summary = graph.contract_summary();

        assert_eq!(summary.issue_count, 1);
        assert!(summary
            .issues
            .iter()
            .any(|issue| matches!(issue, GraphContractIssue::SendRequiresDistinctBuses { .. })));
    }

    #[test]
    fn contract_summary_requires_explicit_mixer_topology_ids() {
        let graph = ExecutableGraph::new(
            "graph:topology-ids",
            vec![
                GraphNodeSpec {
                    topology: GraphNodeTopologyMetadata {
                        role: Some(GraphNodeTopologyRole::TrackLane),
                        track_lane_id: None,
                        bus_group_id: Some("mix:tracks".into()),
                        console_group_id: None,
                        send_return_id: None,
                    },
                    ..routed_node(
                        "track",
                        "main:in",
                        ChannelLayout::Stereo,
                        "bus:tracks",
                        ChannelLayout::Stereo,
                        GraphNodeTopologyRole::TrackLane,
                        vec![],
                    )
                },
                GraphNodeSpec {
                    topology: GraphNodeTopologyMetadata {
                        role: Some(GraphNodeTopologyRole::Bus),
                        track_lane_id: None,
                        bus_group_id: None,
                        console_group_id: None,
                        send_return_id: None,
                    },
                    ..routed_node(
                        "bus",
                        "bus:tracks",
                        ChannelLayout::Stereo,
                        "bus:master",
                        ChannelLayout::Stereo,
                        GraphNodeTopologyRole::Bus,
                        vec![],
                    )
                },
                GraphNodeSpec {
                    topology: GraphNodeTopologyMetadata {
                        role: Some(GraphNodeTopologyRole::Send),
                        track_lane_id: None,
                        bus_group_id: None,
                        console_group_id: None,
                        send_return_id: None,
                    },
                    ..routed_node(
                        "send",
                        "bus:tracks",
                        ChannelLayout::Stereo,
                        "bus:fx",
                        ChannelLayout::Stereo,
                        GraphNodeTopologyRole::Send,
                        vec![],
                    )
                },
                GraphNodeSpec {
                    topology: GraphNodeTopologyMetadata {
                        role: Some(GraphNodeTopologyRole::ConsoleNode),
                        track_lane_id: None,
                        bus_group_id: None,
                        console_group_id: None,
                        send_return_id: None,
                    },
                    ..routed_node(
                        "console",
                        "bus:master",
                        ChannelLayout::Stereo,
                        "main:out",
                        ChannelLayout::Stereo,
                        GraphNodeTopologyRole::ConsoleNode,
                        vec![],
                    )
                },
            ],
        );

        let summary = graph.contract_summary();

        assert!(summary.issues.iter().any(|issue| matches!(
            issue,
            GraphContractIssue::MissingTrackLaneId { node_id } if node_id == "track"
        )));
        assert!(summary.issues.iter().any(|issue| matches!(
            issue,
            GraphContractIssue::MissingBusGroupId { node_id } if node_id == "bus"
        )));
        assert!(summary.issues.iter().any(|issue| matches!(
            issue,
            GraphContractIssue::MissingSendReturnId { node_id } if node_id == "send"
        )));
        assert!(summary.issues.iter().any(|issue| matches!(
            issue,
            GraphContractIssue::MissingConsoleGroupId { node_id } if node_id == "console"
        )));
    }

    #[test]
    fn silence_clear_policy_keeps_silent_blocks_zeroed() {
        let mut buffer = AudioBuffer::new(SampleRate(48_000), ChannelLayout::Stereo, FrameCount(8));
        let graph = ExecutableGraph::new(
            "graph:silence-clear",
            vec![GraphNodeSpec {
                buffer_contract: GraphNodeBufferContract {
                    silence_policy: GraphNodeSilencePolicy::ClearOutput,
                    ..GraphNodeBufferContract::default()
                },
                ..test_node(
                    "bias",
                    GraphNodeExecutionClass::Stateful,
                    0,
                    vec![GraphStageSpec::Bias { amount: 0.75 }],
                )
            }],
        );

        let report = graph.process(&mut buffer);

        assert!(buffer.samples().iter().all(|sample| *sample == 0.0));
        assert_eq!(report.silence_clear_node_count, 1);
    }

    #[test]
    fn direct_edge_routing_chains_track_bus_and_console_output() {
        let mut buffer = AudioBuffer::from_interleaved(
            SampleRate(48_000),
            ChannelLayout::Stereo,
            vec![1.0, -1.0, 0.5, -0.5],
        );
        let graph = ExecutableGraph::new(
            "graph:direct-edge",
            vec![
                routed_node(
                    "track",
                    "main:in",
                    ChannelLayout::Stereo,
                    "bus:track",
                    ChannelLayout::Stereo,
                    GraphNodeTopologyRole::TrackLane,
                    vec![GraphStageSpec::Gain { linear: 0.5 }],
                ),
                routed_node(
                    "console",
                    "bus:track",
                    ChannelLayout::Stereo,
                    "main:out",
                    ChannelLayout::Stereo,
                    GraphNodeTopologyRole::ConsoleNode,
                    vec![GraphStageSpec::Bias { amount: 0.25 }],
                ),
            ],
        );

        let report = graph.process(&mut buffer);

        assert_eq!(buffer.samples(), &[0.75, -0.25, 0.5, 0.0]);
        assert_eq!(report.direct_edge_count, 1);
        assert_eq!(report.fan_in_bus_count, 0);
        assert_eq!(report.fan_out_bus_count, 0);
    }

    #[test]
    fn fan_in_routing_sums_multiple_producers_into_mix_bus() {
        let mut buffer = AudioBuffer::from_interleaved(
            SampleRate(48_000),
            ChannelLayout::Stereo,
            vec![1.0, 1.0, 0.5, 0.5],
        );
        let graph = ExecutableGraph::new(
            "graph:fan-in",
            vec![
                routed_node(
                    "track:a",
                    "main:in",
                    ChannelLayout::Stereo,
                    "bus:mix",
                    ChannelLayout::Stereo,
                    GraphNodeTopologyRole::TrackLane,
                    vec![GraphStageSpec::Gain { linear: 0.5 }],
                ),
                routed_node(
                    "track:b",
                    "main:in",
                    ChannelLayout::Stereo,
                    "bus:mix",
                    ChannelLayout::Stereo,
                    GraphNodeTopologyRole::TrackLane,
                    vec![GraphStageSpec::Gain { linear: 0.25 }],
                ),
                routed_node(
                    "console",
                    "bus:mix",
                    ChannelLayout::Stereo,
                    "main:out",
                    ChannelLayout::Stereo,
                    GraphNodeTopologyRole::ConsoleNode,
                    vec![],
                ),
            ],
        );

        let report = graph.process(&mut buffer);

        assert_eq!(buffer.samples(), &[0.75, 0.75, 0.375, 0.375]);
        assert_eq!(report.fan_in_bus_count, 1);
        assert_eq!(report.mixed_bus_count, 1);
    }

    #[test]
    fn fan_out_routing_splits_single_bus_into_multiple_consumers() {
        let mut buffer = AudioBuffer::from_interleaved(
            SampleRate(48_000),
            ChannelLayout::Stereo,
            vec![1.0, 1.0, 0.5, 0.5],
        );
        let graph = ExecutableGraph::new(
            "graph:fan-out",
            vec![
                routed_node(
                    "track",
                    "main:in",
                    ChannelLayout::Stereo,
                    "bus:source",
                    ChannelLayout::Stereo,
                    GraphNodeTopologyRole::TrackLane,
                    vec![],
                ),
                routed_node(
                    "bus:a",
                    "bus:source",
                    ChannelLayout::Stereo,
                    "main:out",
                    ChannelLayout::Stereo,
                    GraphNodeTopologyRole::Bus,
                    vec![GraphStageSpec::Gain { linear: 0.5 }],
                ),
                routed_node(
                    "bus:b",
                    "bus:source",
                    ChannelLayout::Stereo,
                    "main:out",
                    ChannelLayout::Stereo,
                    GraphNodeTopologyRole::Bus,
                    vec![GraphStageSpec::Gain { linear: 0.25 }],
                ),
            ],
        );

        let report = graph.process(&mut buffer);

        assert_eq!(buffer.samples(), &[0.75, 0.75, 0.375, 0.375]);
        assert_eq!(report.fan_out_bus_count, 1);
        assert_eq!(report.fan_in_bus_count, 1);
    }

    #[test]
    fn send_return_routing_keeps_dry_and_wet_paths_deterministic() {
        let mut buffer = AudioBuffer::from_interleaved(
            SampleRate(48_000),
            ChannelLayout::Stereo,
            vec![1.0, 1.0, 0.5, 0.5],
        );
        let graph = ExecutableGraph::new(
            "graph:send-return",
            vec![
                routed_node(
                    "track",
                    "main:in",
                    ChannelLayout::Stereo,
                    "bus:track",
                    ChannelLayout::Stereo,
                    GraphNodeTopologyRole::TrackLane,
                    vec![],
                ),
                routed_node(
                    "dry",
                    "bus:track",
                    ChannelLayout::Stereo,
                    "bus:mix",
                    ChannelLayout::Stereo,
                    GraphNodeTopologyRole::Bus,
                    vec![GraphStageSpec::Gain { linear: 0.5 }],
                ),
                routed_node(
                    "send",
                    "bus:track",
                    ChannelLayout::Stereo,
                    "bus:fx",
                    ChannelLayout::Stereo,
                    GraphNodeTopologyRole::Send,
                    vec![GraphStageSpec::Gain { linear: 0.25 }],
                ),
                routed_node(
                    "return",
                    "bus:fx",
                    ChannelLayout::Stereo,
                    "bus:mix",
                    ChannelLayout::Stereo,
                    GraphNodeTopologyRole::Return,
                    vec![GraphStageSpec::Bias { amount: 0.1 }],
                ),
                routed_node(
                    "console",
                    "bus:mix",
                    ChannelLayout::Stereo,
                    "main:out",
                    ChannelLayout::Stereo,
                    GraphNodeTopologyRole::ConsoleNode,
                    vec![],
                ),
            ],
        );

        let report = graph.process(&mut buffer);

        assert_eq!(buffer.samples(), &[0.85, 0.85, 0.475, 0.475]);
        assert_eq!(report.fan_out_bus_count, 1);
        assert_eq!(report.fan_in_bus_count, 1);
        assert_eq!(report.send_return_node_count, 2);
    }

    #[test]
    fn forward_references_are_classified_as_unsupported_routing() {
        let graph = ExecutableGraph::new(
            "graph:forward-reference",
            vec![
                routed_node(
                    "return",
                    "bus:fx",
                    ChannelLayout::Stereo,
                    "main:out",
                    ChannelLayout::Stereo,
                    GraphNodeTopologyRole::Return,
                    vec![],
                ),
                routed_node(
                    "send",
                    "main:in",
                    ChannelLayout::Stereo,
                    "bus:fx",
                    ChannelLayout::Stereo,
                    GraphNodeTopologyRole::Send,
                    vec![],
                ),
            ],
        );

        let summary = graph.contract_summary();

        assert!(summary.issues.iter().any(|issue| matches!(
            issue,
            GraphContractIssue::UnsupportedForwardReference { node_id, bus_id }
            if node_id == "return" && bus_id == "bus:fx"
        )));
    }

    #[test]
    fn tail_propagates_across_direct_chain_to_output_bus() {
        let graph = ExecutableGraph::new(
            "graph:tail:direct",
            vec![
                GraphNodeSpec {
                    tail_samples: 32,
                    ..routed_node(
                        "track",
                        "main:in",
                        ChannelLayout::Stereo,
                        "bus:track",
                        ChannelLayout::Stereo,
                        GraphNodeTopologyRole::TrackLane,
                        vec![],
                    )
                },
                GraphNodeSpec {
                    latency_samples: 16,
                    tail_samples: 24,
                    ..routed_node(
                        "console",
                        "bus:track",
                        ChannelLayout::Stereo,
                        "main:out",
                        ChannelLayout::Stereo,
                        GraphNodeTopologyRole::ConsoleNode,
                        vec![],
                    )
                },
            ],
        );

        let routing = graph.routing_summary();
        let mut buffer = AudioBuffer::new(SampleRate(48_000), ChannelLayout::Stereo, FrameCount(8));
        let report = graph.process(&mut buffer);

        assert_eq!(routing.output_latency_samples, 16);
        assert_eq!(routing.output_tail_samples, 56);
        assert_eq!(report.total_tail_samples, 56);
        assert_eq!(report.max_node_tail_samples, 32);
        assert_eq!(report.output_tail_samples, 56);
        assert_eq!(report.max_bus_tail_samples, 56);
    }

    #[test]
    fn fan_in_tail_uses_longest_contributing_path() {
        let graph = ExecutableGraph::new(
            "graph:tail:fan-in",
            vec![
                GraphNodeSpec {
                    tail_samples: 12,
                    ..routed_node(
                        "track:a",
                        "main:in",
                        ChannelLayout::Stereo,
                        "bus:mix",
                        ChannelLayout::Stereo,
                        GraphNodeTopologyRole::TrackLane,
                        vec![],
                    )
                },
                GraphNodeSpec {
                    tail_samples: 48,
                    ..routed_node(
                        "track:b",
                        "main:in",
                        ChannelLayout::Stereo,
                        "bus:mix",
                        ChannelLayout::Stereo,
                        GraphNodeTopologyRole::TrackLane,
                        vec![],
                    )
                },
                routed_node(
                    "console",
                    "bus:mix",
                    ChannelLayout::Stereo,
                    "main:out",
                    ChannelLayout::Stereo,
                    GraphNodeTopologyRole::ConsoleNode,
                    vec![],
                ),
            ],
        );

        let routing = graph.routing_summary();

        assert_eq!(routing.output_tail_samples, 48);
        assert_eq!(routing.max_bus_tail_samples, 48);
    }

    #[test]
    fn send_return_tail_accumulates_along_wet_path() {
        let graph = ExecutableGraph::new(
            "graph:tail:send-return",
            vec![
                GraphNodeSpec {
                    tail_samples: 8,
                    ..routed_node(
                        "track",
                        "main:in",
                        ChannelLayout::Stereo,
                        "bus:track",
                        ChannelLayout::Stereo,
                        GraphNodeTopologyRole::TrackLane,
                        vec![],
                    )
                },
                routed_node(
                    "dry",
                    "bus:track",
                    ChannelLayout::Stereo,
                    "bus:mix",
                    ChannelLayout::Stereo,
                    GraphNodeTopologyRole::Bus,
                    vec![],
                ),
                GraphNodeSpec {
                    tail_samples: 40,
                    ..routed_node(
                        "send",
                        "bus:track",
                        ChannelLayout::Stereo,
                        "bus:fx",
                        ChannelLayout::Stereo,
                        GraphNodeTopologyRole::Send,
                        vec![],
                    )
                },
                GraphNodeSpec {
                    tail_samples: 24,
                    ..routed_node(
                        "return",
                        "bus:fx",
                        ChannelLayout::Stereo,
                        "bus:mix",
                        ChannelLayout::Stereo,
                        GraphNodeTopologyRole::Return,
                        vec![],
                    )
                },
                routed_node(
                    "console",
                    "bus:mix",
                    ChannelLayout::Stereo,
                    "main:out",
                    ChannelLayout::Stereo,
                    GraphNodeTopologyRole::ConsoleNode,
                    vec![],
                ),
            ],
        );

        let routing = graph.routing_summary();

        assert_eq!(routing.output_tail_samples, 72);
        assert_eq!(routing.max_bus_tail_samples, 72);
    }
}
