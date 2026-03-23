use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeSchedulerExportSummary {
    pub phase_count: usize,
    pub anticipative_phase_count: usize,
    pub lane_count: usize,
    pub anticipative_lane_count: usize,
    pub dispatch_count: usize,
    pub prepared_dispatch_count: usize,
    pub realtime_dispatch_count: usize,
    pub dispatch_handoff_count: usize,
    pub prework_service_state: RuntimePreworkServiceState,
    pub prework_service_pressure: RuntimePreworkServicePressure,
    pub prework_service_semantic_policy: RuntimePreworkServiceSemanticPolicy,
    pub prework_pending_target_count: usize,
    pub prework_pending_deferred_target_count: usize,
    pub topology_compatible: bool,
    pub topology_requires_host_reinterpretation: bool,
    pub topology_issue_count: usize,
    pub lane_order: Vec<GraphExecutionLane>,
    pub dispatch_order: Vec<GraphExecutionLane>,
}

impl RuntimeSchedulerExportSummary {
    pub fn from_snapshot(snapshot: &RuntimeEngineBlockSnapshot) -> Self {
        Self {
            phase_count: snapshot.phase_count,
            anticipative_phase_count: snapshot.anticipative_phase_count,
            lane_count: snapshot.lane_count,
            anticipative_lane_count: snapshot.anticipative_lane_count,
            dispatch_count: snapshot.dispatch_count,
            prepared_dispatch_count: snapshot.prepared_dispatch_count,
            realtime_dispatch_count: snapshot.realtime_dispatch_count,
            dispatch_handoff_count: snapshot.dispatch_handoff_count,
            prework_service_state: snapshot.prework_service_state,
            prework_service_pressure: snapshot.prework_service_pressure,
            prework_service_semantic_policy: snapshot.prework_service_semantic_policy,
            prework_pending_target_count: snapshot.prework_pending_target_count,
            prework_pending_deferred_target_count: snapshot.prework_pending_deferred_target_count,
            topology_compatible: snapshot.scheduler_topology.compatible,
            topology_requires_host_reinterpretation: snapshot
                .scheduler_topology
                .requires_host_reinterpretation,
            topology_issue_count: snapshot.scheduler_topology.issues.len(),
            lane_order: snapshot.lane_order.clone(),
            dispatch_order: snapshot.dispatch_order.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeBlockExecutionSummary {
    pub processed_blocks: u64,
    pub last_processing_epoch: Option<u64>,
    pub last_block_sequence: Option<u64>,
    pub last_frame_count: usize,
    pub last_channel_count: usize,
    pub last_block_execution_time_ns: Option<u64>,
    pub last_block_deadline_budget_ns: Option<u64>,
    pub last_block_budget_utilization_percent: Option<f32>,
    pub last_block_budget_overrun_ns: Option<u64>,
    pub last_block_deadline_pressure: RuntimeBlockDeadlinePressure,
    pub budget_overrun_count: u64,
    pub prework_cache_state: RuntimePreworkCacheState,
    pub prework_cache_freshness_state: RuntimePreworkFreshnessState,
    pub last_prework_invalidation_reason: Option<RuntimePreworkInvalidationReason>,
    pub total_latency_samples: u32,
    pub total_tail_samples: u32,
    pub output_tail_samples: u32,
    pub max_bus_tail_samples: u32,
    pub last_input_peak: Option<f32>,
    pub last_output_peak: Option<f32>,
    pub last_output_rms: Option<f32>,
    pub transport_epoch: u64,
    pub transport_transition: Option<RuntimeTransportTransitionKind>,
    pub transport_loop_wrapped: bool,
    pub context_anticipative: Option<bool>,
    pub transport_playing: Option<bool>,
    pub transport_tempo_bpm: Option<f64>,
    pub timeline_position_samples: Option<i64>,
}

impl RuntimeBlockExecutionSummary {
    pub fn from_snapshot(snapshot: &RuntimeEngineBlockSnapshot) -> Self {
        Self {
            processed_blocks: snapshot.processed_blocks,
            last_processing_epoch: snapshot.last_processing_epoch,
            last_block_sequence: snapshot.last_block_sequence,
            last_frame_count: snapshot.last_frame_count,
            last_channel_count: snapshot.last_channel_count,
            last_block_execution_time_ns: snapshot.last_block_execution_time_ns,
            last_block_deadline_budget_ns: snapshot.last_block_deadline_budget_ns,
            last_block_budget_utilization_percent: snapshot.last_block_budget_utilization_percent,
            last_block_budget_overrun_ns: snapshot.last_block_budget_overrun_ns,
            last_block_deadline_pressure: snapshot.last_block_deadline_pressure,
            budget_overrun_count: snapshot.budget_overrun_count,
            prework_cache_state: snapshot.prework_cache_state,
            prework_cache_freshness_state: snapshot.prework_cache_freshness_state,
            last_prework_invalidation_reason: snapshot.last_prework_invalidation_reason,
            total_latency_samples: snapshot.total_latency_samples,
            total_tail_samples: snapshot.total_tail_samples,
            output_tail_samples: snapshot.output_tail_samples,
            max_bus_tail_samples: snapshot.max_bus_tail_samples,
            last_input_peak: snapshot.last_input_peak,
            last_output_peak: snapshot.last_output_peak,
            last_output_rms: snapshot.last_output_rms,
            transport_epoch: snapshot.transport_epoch,
            transport_transition: snapshot.transport_transition,
            transport_loop_wrapped: snapshot.transport_loop_wrapped,
            context_anticipative: snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.anticipative_enabled),
            transport_playing: snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.transport_playing),
            transport_tempo_bpm: snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.transport_tempo_bpm),
            timeline_position_samples: snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.timeline_position_samples),
        }
    }
}
