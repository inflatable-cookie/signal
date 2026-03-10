//! Graph model and execution semantics for Signal.

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

#[derive(Clone, Debug, PartialEq)]
pub struct GraphNodeSpec {
    pub node_id: String,
    pub execution_class: GraphNodeExecutionClass,
    pub latency_samples: u32,
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
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphPreparedDispatch {
    pub buffer: AudioBuffer,
    pub output_peak: f32,
    pub dispatch_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExecutableGraph {
    plan: GraphExecutionPlan,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphBlockReport {
    pub graph_id: String,
    pub context: GraphExecutionContext,
    pub node_count: usize,
    pub stateful_node_count: usize,
    pub latency_node_count: usize,
    pub plugin_backed_node_count: usize,
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
    pub total_latency_samples: u32,
    pub max_node_latency_samples: u32,
    pub frame_count: usize,
    pub channel_count: usize,
    pub input_peak: f32,
    pub prework_output_peak: Option<f32>,
    pub realtime_input_peak: Option<f32>,
    pub output_peak: f32,
    pub output_rms: f32,
    pub first_output_sample: Option<f32>,
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

    pub fn node_count(&self) -> usize {
        self.plan.nodes.len()
    }

    pub fn stage_count(&self) -> usize {
        self.plan.nodes.iter().map(|node| node.stages.len()).sum()
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

    pub fn plugin_backed_node_count(&self) -> usize {
        self.plan
            .nodes
            .iter()
            .filter(|node| matches!(node.execution_class, GraphNodeExecutionClass::PluginBacked))
            .count()
    }

    pub fn planning_summary(&self, anticipative_enabled: bool) -> GraphPlanningSummary {
        let planned_nodes = self
            .plan
            .nodes
            .iter()
            .map(|node| GraphPlannedNode {
                node_id: node.node_id.clone(),
                execution_class: node.execution_class,
                group: planning_group_for_node(node, anticipative_enabled),
                latency_samples: node.latency_samples,
            })
            .collect::<Vec<_>>();
        let phase_order = planning_phase_order(&planned_nodes);
        let phases = phase_order
            .iter()
            .copied()
            .map(|group| GraphPlannedPhase {
                group,
                node_ids: planned_nodes
                    .iter()
                    .filter(|node| node.group == group)
                    .map(|node| node.node_id.clone())
                    .collect(),
            })
            .collect::<Vec<_>>();
        let lane_order = planning_lane_order(&planned_nodes);
        let dispatches = lane_order
            .iter()
            .copied()
            .map(|lane| GraphLaneDispatch {
                lane,
                phase_order: phase_order
                    .iter()
                    .copied()
                    .filter(|group| planning_lane_for_group(*group) == lane)
                    .collect(),
            })
            .collect::<Vec<_>>();

        GraphPlanningSummary {
            inline_realtime_node_count: planned_nodes
                .iter()
                .filter(|node| node.group == GraphNodePlanningGroup::InlineRealtime)
                .count(),
            stateful_realtime_node_count: planned_nodes
                .iter()
                .filter(|node| node.group == GraphNodePlanningGroup::StatefulRealtime)
                .count(),
            anticipative_eligible_node_count: planned_nodes
                .iter()
                .filter(|node| node.group == GraphNodePlanningGroup::AnticipativeEligible)
                .count(),
            plugin_backed_node_count: planned_nodes
                .iter()
                .filter(|node| node.execution_class == GraphNodeExecutionClass::PluginBacked)
                .count(),
            phase_count: phase_order.len(),
            anticipative_phase_count: phase_order
                .iter()
                .filter(|group| **group == GraphNodePlanningGroup::AnticipativeEligible)
                .count(),
            phase_order,
            lane_count: lane_order.len(),
            anticipative_lane_count: lane_order
                .iter()
                .filter(|lane| **lane == GraphExecutionLane::Anticipative)
                .count(),
            lane_order,
            dispatch_count: dispatches.len(),
            dispatch_boundary_count: dispatches.len().saturating_sub(1),
            dispatches,
            phases,
            planned_nodes,
        }
    }

    pub fn execute(&self, request: GraphExecutionRequest) -> (AudioBuffer, GraphBlockReport) {
        let GraphExecutionRequest {
            context,
            mut buffer,
        } = request;
        let report = self.process_with_context(&mut buffer, context);
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
        let input_peak = peak_abs(buffer.samples());
        let planning = self.planning_summary(context.anticipative_enabled);
        let prepared = self.prepare_anticipative(buffer, &context);
        let (working_buffer, report) =
            self.execute_realtime_from_prepared(buffer, input_peak, prepared, context, &planning);
        *buffer = working_buffer;
        report
    }

    pub fn prepare_anticipative(
        &self,
        buffer: &AudioBuffer,
        context: &GraphExecutionContext,
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

        let mut prepared = buffer.clone();
        self.execute_dispatches(
            &mut prepared,
            &anticipative_dispatches,
            context.anticipative_enabled,
        );

        Some(GraphPreparedDispatch {
            output_peak: peak_abs(prepared.samples()),
            buffer: prepared,
            dispatch_count: anticipative_dispatches.len(),
        })
    }

    pub fn execute_realtime_from_prepared(
        &self,
        input: &AudioBuffer,
        input_peak: f32,
        prepared: Option<GraphPreparedDispatch>,
        context: GraphExecutionContext,
        planning: &GraphPlanningSummary,
    ) -> (AudioBuffer, GraphBlockReport) {
        let realtime_dispatches = planning
            .dispatches
            .iter()
            .filter(|dispatch| dispatch.lane == GraphExecutionLane::Realtime)
            .collect::<Vec<_>>();

        let prework_output_peak = prepared.as_ref().map(|prepared| prepared.output_peak);
        let mut realtime_input_peak = prework_output_peak;
        let mut working_buffer = prepared
            .as_ref()
            .map_or_else(|| input.clone(), |prepared| prepared.buffer.clone());

        if !realtime_dispatches.is_empty() {
            if prework_output_peak.is_none() {
                realtime_input_peak = Some(peak_abs(working_buffer.samples()));
            }
            self.execute_dispatches(
                &mut working_buffer,
                &realtime_dispatches,
                context.anticipative_enabled,
            );
        }

        (
            working_buffer.clone(),
            GraphBlockReport {
                graph_id: self.plan.graph_id.clone(),
                context,
                node_count: self.node_count(),
                stateful_node_count: self.stateful_node_count(),
                latency_node_count: self.latency_node_count(),
                plugin_backed_node_count: self.plugin_backed_node_count(),
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
                total_latency_samples: self.total_latency_samples(),
                max_node_latency_samples: self.max_node_latency_samples(),
                frame_count: working_buffer.frames().0,
                channel_count: working_buffer.channel_count().0,
                input_peak,
                prework_output_peak,
                realtime_input_peak,
                output_peak: peak_abs(working_buffer.samples()),
                output_rms: rms(working_buffer.samples()),
                first_output_sample: working_buffer.samples().first().copied(),
            },
        )
    }

    fn execute_dispatches(
        &self,
        buffer: &mut AudioBuffer,
        dispatches: &[&GraphLaneDispatch],
        anticipative_enabled: bool,
    ) {
        for dispatch in dispatches {
            for phase in &dispatch.phase_order {
                for node in
                    self.plan.nodes.iter().filter(|node| {
                        planning_group_for_node(node, anticipative_enabled) == *phase
                    })
                {
                    for stage in &node.stages {
                        apply_stage(buffer, stage);
                    }
                }
            }
        }
    }
}

fn apply_stage(buffer: &mut AudioBuffer, stage: &GraphStageSpec) {
    match *stage {
        GraphStageSpec::Gain { linear } => {
            for sample in buffer.samples_mut() {
                *sample *= linear;
            }
        }
        GraphStageSpec::Bias { amount } => {
            for sample in buffer.samples_mut() {
                *sample += amount;
            }
        }
        GraphStageSpec::TanhDrive { drive } => {
            let drive = drive.max(0.0);
            for sample in buffer.samples_mut() {
                *sample = (*sample * drive).tanh();
            }
        }
        GraphStageSpec::StereoBalance { balance } => {
            apply_stereo_balance(buffer, balance);
        }
        GraphStageSpec::HardClip { threshold } => {
            let threshold = threshold.abs();
            for sample in buffer.samples_mut() {
                *sample = sample.clamp(-threshold, threshold);
            }
        }
    }
}

fn planning_group_for_node(
    node: &GraphNodeSpec,
    anticipative_enabled: bool,
) -> GraphNodePlanningGroup {
    match node.execution_class {
        GraphNodeExecutionClass::PureTransform => GraphNodePlanningGroup::InlineRealtime,
        GraphNodeExecutionClass::Stateful | GraphNodeExecutionClass::PluginBacked => {
            GraphNodePlanningGroup::StatefulRealtime
        }
        GraphNodeExecutionClass::LatencyBearing if anticipative_enabled => {
            GraphNodePlanningGroup::AnticipativeEligible
        }
        GraphNodeExecutionClass::LatencyBearing => GraphNodePlanningGroup::StatefulRealtime,
    }
}

fn planning_phase_order(nodes: &[GraphPlannedNode]) -> Vec<GraphNodePlanningGroup> {
    [
        GraphNodePlanningGroup::InlineRealtime,
        GraphNodePlanningGroup::StatefulRealtime,
        GraphNodePlanningGroup::AnticipativeEligible,
    ]
    .into_iter()
    .filter(|group| nodes.iter().any(|node| node.group == *group))
    .collect()
}

fn planning_lane_order(nodes: &[GraphPlannedNode]) -> Vec<GraphExecutionLane> {
    [
        GraphExecutionLane::Anticipative,
        GraphExecutionLane::Realtime,
    ]
    .into_iter()
    .filter(|lane| {
        nodes
            .iter()
            .any(|node| planning_lane_for_group(node.group) == *lane)
    })
    .collect()
}

fn planning_lane_for_group(group: GraphNodePlanningGroup) -> GraphExecutionLane {
    match group {
        GraphNodePlanningGroup::AnticipativeEligible => GraphExecutionLane::Anticipative,
        GraphNodePlanningGroup::InlineRealtime | GraphNodePlanningGroup::StatefulRealtime => {
            GraphExecutionLane::Realtime
        }
    }
}

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

fn apply_stereo_balance(buffer: &mut AudioBuffer, balance: f32) {
    if buffer.channel_count().0 != 2 {
        return;
    }

    let balance = balance.clamp(-1.0, 1.0);
    let left_gain = if balance >= 0.0 { 1.0 - balance } else { 1.0 };
    let right_gain = if balance <= 0.0 { 1.0 + balance } else { 1.0 };

    for frame in buffer.samples_mut().chunks_exact_mut(2) {
        frame[0] *= left_gain;
        frame[1] *= right_gain;
    }
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
    use super::{
        synthetic_stereo_block, AudioBuffer, ChannelLayout, ExecutableGraph, FrameCount,
        GraphExecutionContext, GraphExecutionLane, GraphExecutionRequest, GraphNodeExecutionClass,
        GraphNodePlanningGroup, GraphNodeSpec, GraphStageSpec, SampleRate,
    };

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
                    node_id: "pre".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![
                        GraphStageSpec::Gain { linear: 0.5 },
                        GraphStageSpec::Bias { amount: 0.25 },
                        GraphStageSpec::TanhDrive { drive: 1.5 },
                    ],
                },
                GraphNodeSpec {
                    node_id: "post".into(),
                    execution_class: GraphNodeExecutionClass::LatencyBearing,
                    latency_samples: 24,
                    stages: vec![
                        GraphStageSpec::StereoBalance { balance: -0.25 },
                        GraphStageSpec::HardClip { threshold: 0.4 },
                    ],
                },
            ],
        );

        let report = graph.process(&mut buffer);

        assert_eq!(report.graph_id, "graph:test");
        assert_eq!(report.node_count, 2);
        assert_eq!(report.stateful_node_count, 1);
        assert_eq!(report.latency_node_count, 1);
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
        assert_eq!(report.total_latency_samples, 24);
        assert_eq!(report.max_node_latency_samples, 24);
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
            vec![GraphNodeSpec {
                node_id: "balance".into(),
                execution_class: GraphNodeExecutionClass::PureTransform,
                latency_samples: 0,
                stages: vec![GraphStageSpec::StereoBalance { balance: 0.5 }],
            }],
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
            vec![GraphNodeSpec {
                node_id: "gain".into(),
                execution_class: GraphNodeExecutionClass::Stateful,
                latency_samples: 0,
                stages: vec![GraphStageSpec::Gain { linear: 0.5 }],
            }],
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
    fn latency_nodes_become_anticipative_candidates_when_enabled() {
        let graph = ExecutableGraph::new(
            "graph:planning",
            vec![
                GraphNodeSpec {
                    node_id: "inline".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 1.0 }],
                },
                GraphNodeSpec {
                    node_id: "latency".into(),
                    execution_class: GraphNodeExecutionClass::LatencyBearing,
                    latency_samples: 32,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.8 }],
                },
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
                GraphNodeSpec {
                    node_id: "inline".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 1.0 }],
                },
                GraphNodeSpec {
                    node_id: "plugin".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.8 }],
                },
                GraphNodeSpec {
                    node_id: "latency".into(),
                    execution_class: GraphNodeExecutionClass::LatencyBearing,
                    latency_samples: 32,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.4 }],
                },
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
    fn anticipative_dispatch_prepares_buffer_before_realtime_pass() {
        let mut buffer = synthetic_stereo_block(SampleRate(48_000), FrameCount(4), 7);
        let graph = ExecutableGraph::new(
            "graph:prework",
            vec![
                GraphNodeSpec {
                    node_id: "anticipative".into(),
                    execution_class: GraphNodeExecutionClass::LatencyBearing,
                    latency_samples: 16,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.2 }],
                },
                GraphNodeSpec {
                    node_id: "realtime".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.5 }],
                },
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
}
