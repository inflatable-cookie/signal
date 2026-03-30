use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct GraphNodeProjection {
    pub node_id: String,
    pub execution_class: GraphNodeExecutionClass,
    pub latency_samples: u32,
    pub stages: Vec<GraphStageSpec>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphProjection {
    pub graph_id: String,
    pub node_count: usize,
    pub nodes: Vec<GraphNodeProjection>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphNodeBusEndpointProjection {
    pub bus_id: String,
    pub channels: ChannelLayout,
}

impl Default for GraphNodeBusEndpointProjection {
    fn default() -> Self {
        Self {
            bus_id: "main:in".into(),
            channels: ChannelLayout::Stereo,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphNodeBufferContractProjection {
    pub input: GraphNodeBusEndpointProjection,
    pub output: GraphNodeBusEndpointProjection,
    pub secondary_input: Option<RuntimeSecondaryInputContractProjection>,
    pub scratch_buffers: usize,
    pub silence_policy: GraphNodeSilencePolicy,
    pub channel_adaptation: GraphChannelAdaptationMode,
    pub reset_policy: GraphNodeResetPolicy,
}

impl Default for GraphNodeBufferContractProjection {
    fn default() -> Self {
        Self {
            input: GraphNodeBusEndpointProjection::default(),
            output: GraphNodeBusEndpointProjection {
                bus_id: "main:out".into(),
                channels: ChannelLayout::Stereo,
            },
            secondary_input: None,
            scratch_buffers: 0,
            silence_policy: GraphNodeSilencePolicy::Process,
            channel_adaptation: GraphChannelAdaptationMode::AdaptiveMonoStereo,
            reset_policy: GraphNodeResetPolicy::RetainAcrossBlocks,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GraphNodeTopologyProjection {
    pub role: Option<GraphNodeTopologyRole>,
    pub track_lane_id: Option<String>,
    pub bus_group_id: Option<String>,
    pub console_group_id: Option<String>,
    pub send_return_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphNodeContractProjection {
    pub node_id: String,
    pub buffer_contract: GraphNodeBufferContractProjection,
    pub topology: GraphNodeTopologyProjection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphContractProjection {
    pub graph_id: String,
    pub contract_count: usize,
    pub nodes: Vec<GraphNodeContractProjection>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginBackedNodeBinding {
    pub node_id: String,
    pub sandbox_id: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePluginIsolationOutcome {
    InProcess,
    SharedSandbox,
    #[default]
    IsolatedSandbox,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimePluginPlacementRuleMatcher {
    Any,
    PluginFormat(PluginFormat),
    PluginTypeId(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginPlacementRule {
    pub rule_id: String,
    pub matcher: RuntimePluginPlacementRuleMatcher,
    pub outcome: RuntimePluginIsolationOutcome,
    pub sandbox_group_key: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginPlacementPolicy {
    pub default_outcome: RuntimePluginIsolationOutcome,
    pub rules: Vec<RuntimePluginPlacementRule>,
}

impl Default for RuntimePluginPlacementPolicy {
    fn default() -> Self {
        Self {
            default_outcome: RuntimePluginIsolationOutcome::IsolatedSandbox,
            rules: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginBackedNodeBindingProjection {
    pub graph_id: String,
    pub bindings: Vec<PluginBackedNodeBinding>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PluginNodeRender {
    pub node_id: String,
    pub sandbox_id: String,
    pub output: AudioBuffer,
    pub latency_samples: u32,
    pub tail_samples: u32,
    pub bypassed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PluginNodeRenderBatch {
    pub graph_id: String,
    pub processing_epoch: u64,
    pub block_sequence: u64,
    pub renders: Vec<PluginNodeRender>,
}
