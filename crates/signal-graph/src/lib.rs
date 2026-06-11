//! Graph plan model for Signal's control plane.
//!
//! This crate models the graph plan vocabulary the control surface reports:
//! node specs and contracts, planning groups, execution lanes, and contract
//! summaries. The block-execution simulation that used to live here was
//! deleted in g10.020 — the production realtime audio path is
//! `signal-render-plane`, which does not use this crate.
//!
//! ```
//! use signal_graph::ExecutableGraph;
//!
//! let graph = ExecutableGraph::new("demo", Vec::new());
//! assert_eq!(graph.graph_id(), "demo");
//! assert_eq!(graph.node_count(), 0);
//! ```

#![warn(missing_docs)]

mod graph_metrics;
#[path = "graph_summary.rs"]
mod graph_summary;
mod types;

pub use types::*;

/// A compiled graph plan: node specs, contracts, and scheduling metadata.
///
/// Wraps a [`GraphExecutionPlan`] and exposes the planning and contract
/// summary entry points the control plane reports through.
#[derive(Clone, Debug, PartialEq)]
pub struct ExecutableGraph {
    plan: GraphExecutionPlan,
}

impl ExecutableGraph {
    /// Construct a graph from a unique `graph_id` and an ordered node list.
    ///
    /// An empty `nodes` list is valid and produces a pass-through graph.
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

/// Graph-level config.
///
/// Intentionally narrow: carries only the nominal block size the control
/// plane was configured with.
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

#[cfg(test)]
mod tests;
