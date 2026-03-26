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

mod bus;
mod execution;
mod graph_metrics;
#[path = "graph_summary.rs"]
mod graph_summary;
mod parameter_events;
mod stage_processor;
mod types;

use std::collections::{BTreeMap, BTreeSet};

pub use types::*;

pub use parameter_events::GraphStageParameterExt;
use parameter_events::{parameter_application_report, stage_parameter_events_for_node};

use graph_summary::{classify_channel_adaptation, planning_group_for_node};
use signal_primitives::{AudioBuffer, ChannelCount, ChannelLayout, FrameCount, SampleRate};
use stage_processor::StageParameterEvent;

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

#[cfg(test)]
mod tests;
