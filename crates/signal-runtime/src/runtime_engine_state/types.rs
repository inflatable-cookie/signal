use super::super::*;

pub(crate) const PREWORK_CACHE_BLOCK_FRESHNESS_WINDOW: u64 = 2;
pub(crate) const PREWORK_QUEUE_CAPACITY: usize = 3;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct RuntimeEngineState {
    pub(crate) graph: Option<ExecutableGraph>,
    pub(crate) snapshot: RuntimeEngineBlockSnapshot,
    pub(crate) plugin_node_bindings: HashMap<String, String>,
    pub(crate) secondary_input_contracts: HashMap<String, RuntimeSecondaryInputContractProjection>,
    pub(crate) pending_plugin_node_renders: BTreeMap<(u64, u64), PluginNodeRenderBatch>,
    pub(crate) latest_plugin_node_renders: BTreeMap<String, RuntimePluginRenderedNodeState>,
    pub(crate) prework_queue: VecDeque<RuntimeEnginePreworkCache>,
    pub(crate) pending_prework_targets: VecDeque<RuntimePendingPreworkTarget>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RuntimePluginRenderedNodeState {
    pub(crate) sandbox_id: String,
    pub(crate) output: AudioBuffer,
    pub(crate) latency_samples: u32,
    pub(crate) tail_samples: u32,
    pub(crate) bypassed: bool,
    pub(crate) processing_epoch: u64,
    pub(crate) block_sequence: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RuntimeEnginePreworkCache {
    pub(crate) graph_id: String,
    pub(crate) projection_epoch: u64,
    pub(crate) parameter_epoch: u64,
    pub(crate) transport: TransportProjection,
    pub(crate) block_size: usize,
    pub(crate) frame_count: usize,
    pub(crate) channel_count: usize,
    pub(crate) input_signature: u64,
    pub(crate) prepared: GraphPreparedDispatch,
    pub(crate) valid_until_processing_epoch: u64,
    pub(crate) valid_until_block_sequence: u64,
    pub(crate) source_processing_epoch: u64,
    pub(crate) source_block_sequence: u64,
    pub(crate) admitted_from_block_sequence: u64,
    pub(crate) consumption_count: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RuntimePendingPreworkTarget {
    pub(crate) target_block_sequence: u64,
    pub(crate) admitted_from_block_sequence: u64,
    pub(crate) buffer: AudioBuffer,
    pub(crate) input_signature: u64,
    pub(crate) backlog_class: RuntimePreworkBacklogClass,
    pub(crate) parameter_epoch_override: Option<u64>,
    pub(crate) transport_override: Option<TransportProjection>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct RuntimePluginBackedBindingSummary {
    pub(crate) bound_sandbox_ids: Vec<String>,
    pub(crate) active_bound_sandboxes: usize,
    pub(crate) degraded_bound_sandboxes: usize,
    pub(crate) missing_bound_sandboxes: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct RuntimePreworkTransportCondition {
    pub(crate) recovery_overlap_sessions: usize,
    pub(crate) lingering_sessions: usize,
    pub(crate) detach_faulted_sessions: usize,
}
