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
mod execution_support;
mod graph_metrics;
#[path = "graph_summary.rs"]
mod graph_summary;
mod parameter_events;
mod stage_processor;
mod types;

pub use types::*;
pub use execution_support::GraphBlockReport;
pub(crate) use execution_support::{build_block_report, GraphBusState};

pub use parameter_events::GraphStageParameterExt;
use parameter_events::{parameter_application_report, stage_parameter_events_for_node};

use graph_summary::{classify_channel_adaptation, planning_group_for_node};
use signal_primitives::{AudioBuffer, ChannelCount, ChannelLayout, FrameCount, SampleRate};
use stage_processor::StageParameterEvent;

#[derive(Clone, Debug, PartialEq)]
pub struct ExecutableGraph {
    plan: GraphExecutionPlan,
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
