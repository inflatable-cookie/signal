use super::*;

/// Single node in an ordered graph projection.
///
/// Carries the execution class and latency that the scheduler needs to build
/// its planning graph.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphNodeProjection {
    /// Unique node identifier.
    pub node_id: String,
    /// Execution class (realtime, anticipative, etc.) for this node.
    pub execution_class: GraphNodeExecutionClass,
    /// Declared latency introduced by this node in samples.
    pub latency_samples: u32,
    /// Stage specs describing the work to be performed at this node.
    pub stages: Vec<GraphStageSpec>,
}

/// Ordered list of [`GraphNodeProjection`]s that the engine will process.
///
/// Pass to `apply_graph_projection()` to commit a new topology to the
/// runtime.  The `graph_id` identifies this particular topology version for
/// prework cache invalidation.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphProjection {
    /// Unique identifier for this graph topology version.
    pub graph_id: String,
    /// Total number of nodes in this projection.
    pub node_count: usize,
    /// Ordered list of nodes in processing order.
    pub nodes: Vec<GraphNodeProjection>,
}

/// Bus endpoint (ID + channel layout) for one side of a graph node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphNodeBusEndpointProjection {
    /// Identifier of the bus at this endpoint.
    pub bus_id: String,
    /// Channel layout of this bus endpoint.
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

/// Buffer contract for a single graph node: buses, silence policy, channel
/// adaptation, and reset policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphNodeBufferContractProjection {
    /// Primary input bus endpoint.
    pub input: GraphNodeBusEndpointProjection,
    /// Primary output bus endpoint.
    pub output: GraphNodeBusEndpointProjection,
    /// Secondary (sidechain/aux) input contract, if applicable.
    pub secondary_input: Option<RuntimeSecondaryInputContractProjection>,
    /// Number of scratch (intermediate) audio buffers required.
    pub scratch_buffers: usize,
    /// How silence is handled at this node.
    pub silence_policy: GraphNodeSilencePolicy,
    /// Channel count adaptation mode between input and output.
    pub channel_adaptation: GraphChannelAdaptationMode,
    /// Whether state is retained across block boundaries.
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

/// Topology metadata attached to a graph node (role, lane membership, group
/// IDs).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GraphNodeTopologyProjection {
    /// Topology role of this node.
    pub role: Option<GraphNodeTopologyRole>,
    /// Track lane this node belongs to, if any.
    pub track_lane_id: Option<String>,
    /// Bus group this node belongs to, if any.
    pub bus_group_id: Option<String>,
    /// Console group this node belongs to, if any.
    pub console_group_id: Option<String>,
    /// Send/return pair this node belongs to, if any.
    pub send_return_id: Option<String>,
}

/// Per-node buffer contract plus topology metadata for the graph contract
/// projection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphNodeContractProjection {
    /// Node identifier this contract applies to.
    pub node_id: String,
    /// Buffer contract for this node.
    pub buffer_contract: GraphNodeBufferContractProjection,
    /// Topology metadata for this node.
    pub topology: GraphNodeTopologyProjection,
}

/// Full buffer contract for an entire graph.
///
/// Pass to `apply_graph_contract_projection()` to give the engine per-node
/// buffer contracts for the new topology.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphContractProjection {
    /// Graph topology version this contract applies to.
    pub graph_id: String,
    /// Number of per-node contracts in this projection.
    pub contract_count: usize,
    /// Per-node contracts.
    pub nodes: Vec<GraphNodeContractProjection>,
}

/// Association between a graph node and the plugin sandbox that processes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginBackedNodeBinding {
    /// Node being bound to a plugin sandbox.
    pub node_id: String,
    /// Sandbox that will process this node.
    pub sandbox_id: String,
}

/// Whether a plugin runs in-process, in a shared sandbox, or fully isolated.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePluginIsolationOutcome {
    /// Plugin runs inside the runtime process.
    InProcess,
    /// Plugin shares a sandbox process with other plugins in the same group.
    SharedSandbox,
    /// Plugin runs in its own dedicated sandbox process.
    #[default]
    IsolatedSandbox,
}

/// Predicate used to match plugins against a placement rule.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimePluginPlacementRuleMatcher {
    /// Matches any plugin.
    Any,
    /// Matches plugins of a specific format (VST3, LV2, etc.).
    PluginFormat(PluginFormat),
    /// Matches a specific plugin type by its ID string.
    PluginTypeId(String),
}

/// Single rule in a [`RuntimePluginPlacementPolicy`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginPlacementRule {
    /// Unique identifier for this rule.
    pub rule_id: String,
    /// Predicate used to match plugins against this rule.
    pub matcher: RuntimePluginPlacementRuleMatcher,
    /// Isolation outcome applied when a plugin matches this rule.
    pub outcome: RuntimePluginIsolationOutcome,
    /// Optional grouping key for shared-sandbox placement.
    pub sandbox_group_key: Option<String>,
}

/// Ordered set of placement rules that determine sandbox isolation for each
/// plugin.  Applied by `apply_plugin_placement_policy()`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginPlacementPolicy {
    /// Isolation outcome used when no rule matches.
    pub default_outcome: RuntimePluginIsolationOutcome,
    /// Ordered list of placement rules evaluated first-match.
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

/// Full set of node→sandbox bindings for a graph.
///
/// Pass to `apply_plugin_backed_node_bindings()` so the engine knows which
/// sandbox owns each plugin-backed node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginBackedNodeBindingProjection {
    /// Graph topology version these bindings apply to.
    pub graph_id: String,
    /// Per-node sandbox bindings.
    pub bindings: Vec<PluginBackedNodeBinding>,
}

/// Rendered output from a single plugin-backed graph node.
#[derive(Clone, Debug, PartialEq)]
pub struct PluginNodeRender {
    /// Node that produced this render.
    pub node_id: String,
    /// Sandbox that processed this node.
    pub sandbox_id: String,
    /// Audio output buffer produced by the plugin.
    pub output: AudioBuffer,
    /// Latency introduced by the plugin in this render in samples.
    pub latency_samples: u32,
    /// Tail length reported by the plugin in samples.
    pub tail_samples: u32,
    /// Whether the plugin was bypassed for this render.
    pub bypassed: bool,
}

/// Batch of per-node plugin renders for one processed block.
#[derive(Clone, Debug, PartialEq)]
pub struct PluginNodeRenderBatch {
    /// Graph topology version these renders correspond to.
    pub graph_id: String,
    /// Processing epoch of the block these renders belong to.
    pub processing_epoch: u64,
    /// Block sequence number.
    pub block_sequence: u64,
    /// Per-node renders for this block.
    pub renders: Vec<PluginNodeRender>,
}
