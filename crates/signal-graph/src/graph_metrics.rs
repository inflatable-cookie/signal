// Graph metrics and introspection for ExecutableGraph
use crate::{ExecutableGraph, GraphNodeExecutionClass};

impl ExecutableGraph {
    /// Returns the number of nodes in the graph.
    pub fn node_count(&self) -> usize {
        self.plan.nodes.len()
    }

    /// Returns the total number of stages across all nodes.
    pub fn stage_count(&self) -> usize {
        self.plan.nodes.iter().map(|node| node.stages.len()).sum()
    }

    /// Returns the number of stateful nodes.
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

    /// Returns the number of latency-bearing nodes.
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

    /// Returns the total latency across all nodes in samples.
    pub fn total_latency_samples(&self) -> u32 {
        self.plan
            .nodes
            .iter()
            .map(|node| node.latency_samples)
            .sum()
    }

    /// Returns the maximum latency of any node in samples.
    pub fn max_node_latency_samples(&self) -> u32 {
        self.plan
            .nodes
            .iter()
            .map(|node| node.latency_samples)
            .max()
            .unwrap_or(0)
    }

    /// Returns the total tail across all nodes in samples.
    pub fn total_tail_samples(&self) -> u32 {
        self.plan.nodes.iter().map(|node| node.tail_samples).sum()
    }

    /// Returns the maximum tail of any node in samples.
    pub fn max_node_tail_samples(&self) -> u32 {
        self.plan
            .nodes
            .iter()
            .map(|node| node.tail_samples)
            .max()
            .unwrap_or(0)
    }

    /// Returns the number of plugin-backed nodes.
    pub fn plugin_backed_node_count(&self) -> usize {
        self.plan
            .nodes
            .iter()
            .filter(|node| matches!(node.execution_class, GraphNodeExecutionClass::PluginBacked))
            .count()
    }
}
