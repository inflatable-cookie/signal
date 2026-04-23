use signal_primitives::ChannelLayout;

use super::{
    GraphChannelAdaptationMode, GraphChannelAdaptationResult, GraphNodeResetPolicy,
    GraphNodeSilencePolicy, GraphNodeTopologyRole,
};

/// A validation problem found during graph contract checking.
///
/// Contract issues are non-fatal: the graph still executes, but the relevant
/// nodes may produce silence or be skipped. The full list is surfaced in
/// [`GraphContractSummary::issues`] and the count in
/// [`GraphBlockReport::contract_issue_count`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GraphContractIssue {
    /// A node declared an empty string as its input bus ID.
    EmptyInputBusId {
        /// The offending node's ID.
        node_id: String,
    },
    /// A node declared an empty string as its output bus ID.
    EmptyOutputBusId {
        /// The offending node's ID.
        node_id: String,
    },
    /// The bus this node reads from has no upstream writer.
    MissingInputBusProducer {
        /// The node that requires the missing producer.
        node_id: String,
        /// The bus with no upstream writer.
        bus_id: String,
    },
    /// The node references an output bus that appears later in the node list
    /// (forward references are not supported).
    UnsupportedForwardReference {
        /// The node with the forward reference.
        node_id: String,
        /// The forward-referenced bus.
        bus_id: String,
    },
    /// The input/output channel layouts cannot be adapted under the node's
    /// declared [`GraphChannelAdaptationMode`].
    UnsupportedChannelAdaptation {
        /// The offending node's ID.
        node_id: String,
        /// Actual input channel layout.
        input: ChannelLayout,
        /// Desired output channel layout.
        output: ChannelLayout,
        /// Adaptation mode that was in effect.
        mode: GraphChannelAdaptationMode,
    },
    /// Two nodes write to the same output bus with different channel layouts,
    /// which is forbidden.
    InconsistentOutputBusChannels {
        /// The second writer that caused the inconsistency.
        node_id: String,
        /// The shared bus.
        bus_id: String,
        /// Channel layout established by the first writer.
        expected: ChannelLayout,
        /// Channel layout declared by this node.
        actual: ChannelLayout,
    },
    /// A `Send` node must use distinct input and output buses.
    SendRequiresDistinctBuses {
        /// The offending node's ID.
        node_id: String,
    },
    /// A `Return` node must use distinct input and output buses.
    ReturnRequiresDistinctBuses {
        /// The offending node's ID.
        node_id: String,
    },
    /// A [`GraphNodeTopologyRole::TrackLane`] node is missing its
    /// `track_lane_id` annotation.
    MissingTrackLaneId {
        /// The offending node's ID.
        node_id: String,
    },
    /// A [`GraphNodeTopologyRole::Bus`] node is missing its `bus_group_id`
    /// annotation.
    MissingBusGroupId {
        /// The offending node's ID.
        node_id: String,
    },
    /// A [`GraphNodeTopologyRole::ConsoleNode`] node is missing its
    /// `console_group_id` annotation.
    MissingConsoleGroupId {
        /// The offending node's ID.
        node_id: String,
    },
    /// A `Send` or `Return` node is missing its `send_return_id` annotation.
    MissingSendReturnId {
        /// The offending node's ID.
        node_id: String,
    },
}

/// Resolved contract information for a single node after validation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphNodeContractSummary {
    /// The node this summary describes.
    pub node_id: String,
    /// Resolved input bus identifier.
    pub input_bus_id: String,
    /// Resolved output bus identifier.
    pub output_bus_id: String,
    /// Channel layout on the input bus.
    pub input_channels: ChannelLayout,
    /// Channel layout on the output bus.
    pub output_channels: ChannelLayout,
    /// Silence handling policy for this node.
    pub silence_policy: GraphNodeSilencePolicy,
    /// Channel adaptation mode declared by the node.
    pub channel_adaptation: GraphChannelAdaptationMode,
    /// Outcome of resolving the channel adaptation for this node.
    pub adaptation_result: GraphChannelAdaptationResult,
    /// Number of scratch buffers allocated for this node.
    pub scratch_buffers: usize,
    /// State reset policy for this node.
    pub reset_policy: GraphNodeResetPolicy,
    /// Resolved topology role (defaults to `Utility` when unset).
    pub topology_role: GraphNodeTopologyRole,
    /// Track lane identifier, if applicable.
    pub track_lane_id: Option<String>,
    /// Bus group identifier, if applicable.
    pub bus_group_id: Option<String>,
    /// Console group identifier, if applicable.
    pub console_group_id: Option<String>,
    /// Send/return pair identifier, if applicable.
    pub send_return_id: Option<String>,
}

/// Aggregate result of validating all node contracts in the graph.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GraphContractSummary {
    /// Total number of contract issues found.
    pub issue_count: usize,
    /// Full list of contract issues (see [`GraphContractIssue`]).
    pub issues: Vec<GraphContractIssue>,
    /// Number of nodes with [`GraphNodeSilencePolicy::ClearOutput`].
    pub silence_clear_node_count: usize,
    /// Number of nodes that require channel adaptation.
    pub adaptive_channel_node_count: usize,
    /// Number of nodes with a non-`RetainAcrossBlocks` reset policy.
    pub resettable_node_count: usize,
    /// Total scratch buffers allocated across all nodes.
    pub scratch_buffer_count: usize,
    /// Number of nodes assigned the `TrackLane` topology role.
    pub track_lane_node_count: usize,
    /// Number of nodes assigned the `Bus` topology role.
    pub bus_node_count: usize,
    /// Number of nodes assigned the `Send` or `Return` topology role.
    pub send_return_node_count: usize,
    /// Number of nodes assigned the `ConsoleNode` topology role.
    pub console_node_count: usize,
    /// Per-node resolved contracts, in graph node order.
    pub node_contracts: Vec<GraphNodeContractSummary>,
}

/// Aggregate result of analysing the routing topology for the graph.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GraphRoutingSummary {
    /// Total number of distinct buses in the graph.
    pub routed_bus_count: usize,
    /// Number of buses where exactly one node writes and one node reads
    /// (simple serial edges).
    pub direct_edge_count: usize,
    /// Number of buses with more than one reader (fan-out from one writer).
    pub fan_in_bus_count: usize,
    /// Number of buses with more than one writer (fan-in to a summing bus).
    pub fan_out_bus_count: usize,
    /// Number of buses that are both fan-in and fan-out.
    pub mixed_bus_count: usize,
    /// Accumulated latency on the primary output bus (`"main:out"`), in samples.
    pub output_latency_samples: u32,
    /// Maximum latency across all buses in the graph, in samples.
    pub max_bus_latency_samples: u32,
    /// Tail time on the primary output bus, in samples.
    pub output_tail_samples: u32,
    /// Maximum tail time across all buses in the graph, in samples.
    pub max_bus_tail_samples: u32,
}
