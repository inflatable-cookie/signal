//! Graph model and offline execution semantics for Signal.
//!
//! This crate is an offline/simulation graph execution engine for the control
//! plane and diagnostic harnesses (notably signal-runtime's engine-block
//! path). It models node contracts, routed buses, planning groups, execution
//! lanes, and block-local parameter-event application, and produces detailed
//! per-block reports for inspection.
//!
//! It is NOT suitable for the audio callback: execution allocates per block
//! (summary recomputation, buffer clones, per-stage event collection). The
//! production realtime audio path is `signal-render-plane`, which does not use
//! this crate.
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

#![warn(missing_docs)]

mod bus;
mod execution;
mod execution_support;
mod graph_metrics;
#[path = "graph_summary.rs"]
mod graph_summary;
mod parameter_events;
mod stage_processor;
mod types;

pub use execution_support::GraphBlockReport;
pub(crate) use execution_support::{build_block_report, GraphBusState};
pub use types::*;

pub use parameter_events::GraphStageParameterExt;
use parameter_events::{parameter_application_report, stage_parameter_events_for_node};

use graph_summary::{classify_channel_adaptation, planning_group_for_node};
use signal_primitives::{AudioBuffer, ChannelCount, ChannelLayout, FrameCount, SampleRate};
use stage_processor::StageParameterEvent;

/// A compiled, executable audio processing graph for offline/simulation use.
///
/// Wraps a [`GraphExecutionPlan`] and exposes the block-processing entry
/// points. Build one from a graph ID and a list of [`GraphNodeSpec`]s, then
/// call [`ExecutableGraph::process_with_context`] per simulated block.
///
/// Not realtime-safe: each call allocates (working-buffer clones, per-stage
/// event vectors, report construction). Use it from the control plane or
/// diagnostic harnesses, never from the audio callback — the production audio
/// path is `signal-render-plane`.
#[derive(Clone, Debug, PartialEq)]
pub struct ExecutableGraph {
    plan: GraphExecutionPlan,
}

impl ExecutableGraph {
    /// Construct a graph from a unique `graph_id` and an ordered node list.
    ///
    /// Nodes are processed in slice order unless the planner reorders them for
    /// lane dispatch. An empty `nodes` list is valid and produces a pass-through
    /// graph.
    pub fn new(graph_id: impl Into<String>, nodes: Vec<GraphNodeSpec>) -> Self {
        Self {
            plan: GraphExecutionPlan {
                graph_id: graph_id.into(),
                nodes,
            },
        }
    }

    /// Returns the graph's unique identifier.
    pub fn graph_id(&self) -> &str {
        self.plan.graph_id.as_str()
    }

    /// Returns a reference to the underlying execution plan.
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
    /// Nominal processing block size in frames.
    pub block_size: usize,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self { block_size: 512 }
    }
}

/// Create a deterministic stereo [`AudioBuffer`] for testing and demos.
///
/// Fills a stereo interleaved buffer with a simple ramp-derived waveform. The
/// `seed` shifts the amplitude range so different calls produce distinguishable
/// signals. The left channel ramps from `seed * 0.03125 - 1` to
/// `seed * 0.03125 + 1`; the right channel mirrors at half amplitude.
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
