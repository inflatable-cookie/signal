use signal_primitives::ChannelLayout;

use super::{
    GraphChannelAdaptationMode, GraphChannelAdaptationResult, GraphNodeResetPolicy,
    GraphNodeSilencePolicy, GraphNodeTopologyRole,
};

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
