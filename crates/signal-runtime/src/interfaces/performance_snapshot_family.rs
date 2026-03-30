use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimePerformanceSnapshot {
    pub sample_rate_hz: u32,
    pub block_size: usize,
    pub processed_block_count: u64,
    pub last_block_sequence: Option<u64>,
    pub cpu_load_percent: f32,
    pub graph_latency_ms: f32,
    pub last_block_execution_time_ns: Option<u64>,
    pub last_block_deadline_budget_ns: Option<u64>,
    pub last_block_budget_utilization_percent: Option<f32>,
    pub last_block_budget_overrun_ns: Option<u64>,
    pub last_block_deadline_pressure: RuntimeBlockDeadlinePressure,
    pub budget_overrun_count: u64,
    pub peak_block_execution_time_ns: u64,
    pub peak_block_budget_utilization_percent: f32,
    pub peak_block_budget_overrun_ns: u64,
    pub xrun_count: u64,
    pub scheduler_phase_count: usize,
    pub scheduler_lane_count: usize,
    pub scheduler_dispatch_count: usize,
    pub scheduler_prepared_dispatch_count: usize,
    pub scheduler_realtime_dispatch_count: usize,
    pub scheduler_dispatch_handoff_count: usize,
    pub scheduler_topology_compatible: bool,
    pub scheduler_topology_requires_host_reinterpretation: bool,
    pub scheduler_topology_issue_count: usize,
    pub prework_service_state: RuntimePreworkServiceState,
    pub prework_service_pressure: RuntimePreworkServicePressure,
    pub prework_service_semantic_policy: RuntimePreworkServiceSemanticPolicy,
    pub pending_prework_target_count: usize,
    pub pending_prework_deferred_target_count: usize,
    pub prework_queue_depth: usize,
    pub prework_peak_queue_depth: usize,
    pub prework_service_cycle_count: u64,
    pub prework_service_starvation_count: u64,
    pub prework_service_throttle_count: u64,
    pub prework_service_yield_count: u64,
    pub last_prework_service_effective_cycles: usize,
    pub last_prework_service_budget_per_cycle: Option<usize>,
    pub last_prework_service_effective_budget_per_cycle: Option<usize>,
    pub last_prework_serviced_backlog_class: Option<String>,
    pub transport_gate_active: bool,
    pub plugin_gate_active: bool,
    pub hot_latency_node_id: Option<String>,
    pub hot_latency_node_group: Option<String>,
    pub hot_latency_node_topology_role: Option<String>,
    pub hot_latency_node_plugin_sandbox_id: Option<String>,
    pub hot_latency_node_samples: u32,
    pub hot_latency_group: Option<String>,
    pub hot_latency_group_node_count: usize,
    pub hot_latency_group_total_samples: u32,
    pub critical_path_lane: Option<String>,
    pub critical_path_lane_node_count: usize,
    pub critical_path_lane_plugin_backed_node_count: usize,
    pub critical_path_lane_planning_group_count: usize,
    pub critical_path_lane_total_latency_samples: u32,
    pub worker_lane_summaries: Vec<RuntimeWorkerLaneInstrumentationSummary>,
    pub background_service_class: Option<RuntimeDeferredServiceClass>,
    pub background_service_decision: Option<RuntimeDeferredServiceDecision>,
    pub background_service_reason: Option<RuntimeDeferredServiceReason>,
    pub background_service_priority_band: Option<RuntimeDeferredServicePriorityBand>,
    pub background_service_blocking_priority_band: Option<RuntimeDeferredServicePriorityBand>,
    pub background_service_backpressure_source: Option<RuntimeDeferredServiceBackpressureSource>,
    pub background_service_starvation_risk: bool,
    pub background_service_starved_work_item_count: usize,
    pub background_service_cancellation_cause: Option<RuntimeDeferredServiceCancellationCause>,
    pub background_service_cancelled_work_item_count: usize,
    pub background_queued_work_item_count: usize,
    pub background_deferred_work_item_count: usize,
    pub background_pending_cleanup_work_item_count: usize,
    pub background_pending_retry_work_item_count: usize,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeWorkerLaneInstrumentationSummary {
    pub lane: GraphExecutionLane,
    pub node_count: usize,
    pub plugin_backed_node_count: usize,
    pub planning_group_count: usize,
    pub total_latency_samples: u32,
    pub max_node_latency_samples: u32,
}

impl RuntimePerformanceSnapshot {
    pub fn capture(
        effective_config: &EffectiveRuntimeConfig,
        diagnostics_snapshot: &RuntimeDiagnosticsSnapshot,
        engine_block_snapshot: &RuntimeEngineBlockSnapshot,
        last_deferred_service_receipt: Option<&RuntimeDeferredServiceReceipt>,
    ) -> Self {
        let worker_lane_summaries =
            runtime_worker_lane_instrumentation_summaries(engine_block_snapshot);
        let hot_latency_node = engine_block_snapshot
            .planned_nodes
            .iter()
            .max_by_key(|node| node.latency_samples)
            .filter(|node| node.latency_samples > 0);
        let mut inline_realtime_group_total_samples = 0u32;
        let mut inline_realtime_group_node_count = 0usize;
        let mut stateful_realtime_group_total_samples = 0u32;
        let mut stateful_realtime_group_node_count = 0usize;
        let mut anticipative_group_total_samples = 0u32;
        let mut anticipative_group_node_count = 0usize;
        for node in &engine_block_snapshot.planned_nodes {
            match node.group {
                GraphNodePlanningGroup::InlineRealtime => {
                    inline_realtime_group_total_samples =
                        inline_realtime_group_total_samples.saturating_add(node.latency_samples);
                    inline_realtime_group_node_count =
                        inline_realtime_group_node_count.saturating_add(1);
                }
                GraphNodePlanningGroup::StatefulRealtime => {
                    stateful_realtime_group_total_samples =
                        stateful_realtime_group_total_samples.saturating_add(node.latency_samples);
                    stateful_realtime_group_node_count =
                        stateful_realtime_group_node_count.saturating_add(1);
                }
                GraphNodePlanningGroup::AnticipativeEligible => {
                    anticipative_group_total_samples =
                        anticipative_group_total_samples.saturating_add(node.latency_samples);
                    anticipative_group_node_count = anticipative_group_node_count.saturating_add(1);
                }
            }
        }
        let hot_latency_group = [
            (
                GraphNodePlanningGroup::InlineRealtime,
                inline_realtime_group_total_samples,
                inline_realtime_group_node_count,
            ),
            (
                GraphNodePlanningGroup::StatefulRealtime,
                stateful_realtime_group_total_samples,
                stateful_realtime_group_node_count,
            ),
            (
                GraphNodePlanningGroup::AnticipativeEligible,
                anticipative_group_total_samples,
                anticipative_group_node_count,
            ),
        ]
        .into_iter()
        .max_by_key(|(_, total_samples, _)| *total_samples)
        .filter(|(_, total_samples, _)| *total_samples > 0);
        let critical_path_lane = worker_lane_summaries
            .iter()
            .max_by_key(|summary| summary.total_latency_samples)
            .filter(|summary| summary.total_latency_samples > 0);
        let mut snapshot = Self {
            sample_rate_hz: effective_config.sample_rate.0,
            block_size: effective_config.block_size,
            processed_block_count: engine_block_snapshot.processed_blocks,
            last_block_sequence: engine_block_snapshot.last_block_sequence,
            cpu_load_percent: diagnostics_snapshot.cpu_load_percent,
            graph_latency_ms: diagnostics_snapshot.graph_latency_ms,
            last_block_execution_time_ns: engine_block_snapshot.last_block_execution_time_ns,
            last_block_deadline_budget_ns: engine_block_snapshot.last_block_deadline_budget_ns,
            last_block_budget_utilization_percent: engine_block_snapshot
                .last_block_budget_utilization_percent,
            last_block_budget_overrun_ns: engine_block_snapshot.last_block_budget_overrun_ns,
            last_block_deadline_pressure: engine_block_snapshot.last_block_deadline_pressure,
            budget_overrun_count: engine_block_snapshot.budget_overrun_count,
            peak_block_execution_time_ns: engine_block_snapshot.peak_block_execution_time_ns,
            peak_block_budget_utilization_percent: engine_block_snapshot
                .peak_block_budget_utilization_percent,
            peak_block_budget_overrun_ns: engine_block_snapshot.peak_block_budget_overrun_ns,
            xrun_count: diagnostics_snapshot.xruns,
            scheduler_phase_count: engine_block_snapshot.phase_count,
            scheduler_lane_count: engine_block_snapshot.lane_count,
            scheduler_dispatch_count: engine_block_snapshot.dispatch_count,
            scheduler_prepared_dispatch_count: engine_block_snapshot.prepared_dispatch_count,
            scheduler_realtime_dispatch_count: engine_block_snapshot.realtime_dispatch_count,
            scheduler_dispatch_handoff_count: engine_block_snapshot.dispatch_handoff_count,
            scheduler_topology_compatible: engine_block_snapshot.scheduler_topology.compatible,
            scheduler_topology_requires_host_reinterpretation: engine_block_snapshot
                .scheduler_topology
                .requires_host_reinterpretation,
            scheduler_topology_issue_count: engine_block_snapshot.scheduler_topology.issues.len(),
            prework_service_state: engine_block_snapshot.prework_service_state,
            prework_service_pressure: engine_block_snapshot.prework_service_pressure,
            prework_service_semantic_policy: engine_block_snapshot.prework_service_semantic_policy,
            pending_prework_target_count: engine_block_snapshot.prework_pending_target_count,
            pending_prework_deferred_target_count: engine_block_snapshot
                .prework_pending_deferred_target_count,
            prework_queue_depth: engine_block_snapshot.prework_cache_queue_depth,
            prework_peak_queue_depth: engine_block_snapshot.prework_cache_peak_queue_depth,
            prework_service_cycle_count: engine_block_snapshot.prework_service_cycle_count,
            prework_service_starvation_count: engine_block_snapshot
                .prework_service_starvation_count,
            prework_service_throttle_count: engine_block_snapshot.prework_service_throttle_count,
            prework_service_yield_count: engine_block_snapshot.prework_service_yield_count,
            last_prework_service_effective_cycles: engine_block_snapshot
                .last_prework_service_effective_cycles,
            last_prework_service_budget_per_cycle: engine_block_snapshot
                .last_prework_service_budget_per_cycle,
            last_prework_service_effective_budget_per_cycle: engine_block_snapshot
                .last_prework_service_effective_budget_per_cycle,
            last_prework_serviced_backlog_class: engine_block_snapshot
                .last_prework_serviced_backlog_class
                .map(|value| runtime_prework_backlog_class_name(value).to_string()),
            transport_gate_active: engine_block_snapshot.prework_service_transport_gate_active,
            plugin_gate_active: engine_block_snapshot.prework_service_plugin_gate_active,
            hot_latency_node_id: hot_latency_node.map(|node| node.node_id.clone()),
            hot_latency_node_group: hot_latency_node
                .map(|node| runtime_graph_node_planning_group_name(node.group).to_string()),
            hot_latency_node_topology_role: hot_latency_node
                .map(|node| runtime_graph_node_topology_role_name(node.topology_role).to_string()),
            hot_latency_node_plugin_sandbox_id: hot_latency_node
                .and_then(|node| node.plugin_sandbox_id.clone()),
            hot_latency_node_samples: hot_latency_node.map_or(0, |node| node.latency_samples),
            hot_latency_group: hot_latency_group
                .map(|(group, _, _)| runtime_graph_node_planning_group_name(group).to_string()),
            hot_latency_group_node_count: hot_latency_group
                .map_or(0, |(_, _, node_count)| node_count),
            hot_latency_group_total_samples: hot_latency_group
                .map_or(0, |(_, total_samples, _)| total_samples),
            critical_path_lane: critical_path_lane
                .map(|summary| runtime_execution_lane_name(summary.lane).to_string()),
            critical_path_lane_node_count: critical_path_lane
                .map_or(0, |summary| summary.node_count),
            critical_path_lane_plugin_backed_node_count: critical_path_lane
                .map_or(0, |summary| summary.plugin_backed_node_count),
            critical_path_lane_planning_group_count: critical_path_lane
                .map_or(0, |summary| summary.planning_group_count),
            critical_path_lane_total_latency_samples: critical_path_lane
                .map_or(0, |summary| summary.total_latency_samples),
            worker_lane_summaries,
            background_service_class: last_deferred_service_receipt
                .map(|receipt| receipt.work_class),
            background_service_decision: last_deferred_service_receipt
                .map(|receipt| receipt.decision),
            background_service_reason: last_deferred_service_receipt.map(|receipt| receipt.reason),
            background_service_priority_band: last_deferred_service_receipt
                .map(|receipt| receipt.priority_band),
            background_service_blocking_priority_band: last_deferred_service_receipt
                .and_then(|receipt| receipt.blocking_priority_band),
            background_service_backpressure_source: last_deferred_service_receipt
                .and_then(|receipt| receipt.backpressure_source),
            background_service_starvation_risk: last_deferred_service_receipt
                .is_some_and(|receipt| receipt.starvation_risk),
            background_service_starved_work_item_count: last_deferred_service_receipt
                .map(|receipt| receipt.starved_work_item_count)
                .unwrap_or(0),
            background_service_cancellation_cause: last_deferred_service_receipt
                .and_then(|receipt| receipt.cancellation_cause),
            background_service_cancelled_work_item_count: last_deferred_service_receipt
                .map(|receipt| receipt.cancelled_work_item_count)
                .unwrap_or(0),
            background_queued_work_item_count: last_deferred_service_receipt
                .map(|receipt| receipt.queued_work_item_count)
                .unwrap_or(0),
            background_deferred_work_item_count: last_deferred_service_receipt
                .map(|receipt| receipt.deferred_work_item_count)
                .unwrap_or(0),
            background_pending_cleanup_work_item_count: last_deferred_service_receipt
                .map(|receipt| receipt.pending_cleanup_work_items)
                .unwrap_or(0),
            background_pending_retry_work_item_count: last_deferred_service_receipt
                .map(|receipt| receipt.pending_deferred_retry_work_items)
                .unwrap_or(0),
            summary: String::new(),
        };
        let dispatch_summary = format!(
            "{}/{}/{}",
            snapshot.scheduler_dispatch_count,
            snapshot.scheduler_prepared_dispatch_count,
            snapshot.scheduler_realtime_dispatch_count
        );
        let topology_summary = format!(
            "{}/{}/{}",
            snapshot.scheduler_topology_compatible,
            snapshot.scheduler_topology_requires_host_reinterpretation,
            snapshot.scheduler_topology_issue_count
        );
        let prework_summary = format!(
            "{:?}/{:?}/{:?}",
            snapshot.prework_service_state,
            snapshot.prework_service_pressure,
            snapshot.prework_service_semantic_policy,
        );
        let service_summary = format!(
            "{}/{}/{}/{}",
            snapshot.prework_service_starvation_count,
            snapshot.prework_service_throttle_count,
            snapshot.prework_service_yield_count,
            snapshot.last_prework_service_effective_cycles,
        );
        let hot_node_summary = format!(
            "{:?}/{:?}/{:?}/{}",
            snapshot.hot_latency_node_id,
            snapshot.hot_latency_node_group,
            snapshot.hot_latency_node_topology_role,
            snapshot.hot_latency_node_samples,
        );
        let hot_group_summary = format!(
            "{:?}/{}/{}",
            snapshot.hot_latency_group,
            snapshot.hot_latency_group_node_count,
            snapshot.hot_latency_group_total_samples
        );
        let critical_lane_summary = format!(
            "{:?}/{}/{}/{}/{}",
            snapshot.critical_path_lane,
            snapshot.critical_path_lane_node_count,
            snapshot.critical_path_lane_plugin_backed_node_count,
            snapshot.critical_path_lane_planning_group_count,
            snapshot.critical_path_lane_total_latency_samples
        );
        let worker_lane_summary = snapshot
            .worker_lane_summaries
            .iter()
            .map(|summary| {
                format!(
                    "{}:{}/{}/{}/{}/{}",
                    runtime_execution_lane_name(summary.lane),
                    summary.node_count,
                    summary.plugin_backed_node_count,
                    summary.planning_group_count,
                    summary.total_latency_samples,
                    summary.max_node_latency_samples,
                )
            })
            .collect::<Vec<_>>()
            .join("|");
        let background_summary = format!(
            "{:?}/{:?}/{:?}/{:?}/{:?}/{:?}/{}/{}",
            snapshot.background_service_class,
            snapshot.background_service_decision,
            snapshot.background_service_reason,
            snapshot.background_service_priority_band,
            snapshot.background_service_blocking_priority_band,
            snapshot.background_service_backpressure_source,
            snapshot.background_service_starvation_risk,
            snapshot.background_service_cancelled_work_item_count,
        );
        snapshot.summary = format!(
            "sample_rate={} block_size={} blocks={} cpu_load={:.3} graph_latency_ms={:.3} timing={:?}/{:?}/{:?}/{:?}/{:?}/{} xruns={} phases={} lanes={} dispatches={} handoff={} topology={} prework={} pending_targets={}/{} queue={}/{} service={} cycles={} budget={:?}/{:?} backlog={:?} gates={}/{} hot_node={} hot_group={} critical_lane={} worker_lanes={} background={} policy={:?}/{:?}/{:?}/{}/{} items={}/{}/{}/{}",
            snapshot.sample_rate_hz,
            snapshot.block_size,
            snapshot.processed_block_count,
            snapshot.cpu_load_percent,
            snapshot.graph_latency_ms,
            snapshot.last_block_execution_time_ns,
            snapshot.last_block_deadline_budget_ns,
            snapshot.last_block_budget_utilization_percent,
            snapshot.last_block_budget_overrun_ns,
            snapshot.last_block_deadline_pressure,
            snapshot.budget_overrun_count,
            snapshot.xrun_count,
            snapshot.scheduler_phase_count,
            snapshot.scheduler_lane_count,
            dispatch_summary,
            snapshot.scheduler_dispatch_handoff_count,
            topology_summary,
            prework_summary,
            snapshot.pending_prework_target_count,
            snapshot.pending_prework_deferred_target_count,
            snapshot.prework_queue_depth,
            snapshot.prework_peak_queue_depth,
            service_summary,
            snapshot.prework_service_cycle_count,
            snapshot.last_prework_service_budget_per_cycle,
            snapshot.last_prework_service_effective_budget_per_cycle,
            snapshot.last_prework_serviced_backlog_class,
            snapshot.transport_gate_active,
            snapshot.plugin_gate_active,
            hot_node_summary,
            hot_group_summary,
            critical_lane_summary,
            worker_lane_summary,
            background_summary,
            snapshot.background_service_priority_band,
            snapshot.background_service_blocking_priority_band,
            snapshot.background_service_backpressure_source,
            snapshot.background_service_starved_work_item_count,
            snapshot.background_service_cancelled_work_item_count,
            snapshot.background_queued_work_item_count,
            snapshot.background_deferred_work_item_count,
            snapshot.background_pending_cleanup_work_item_count,
            snapshot.background_pending_retry_work_item_count,
        );
        snapshot
    }

    pub fn render_json(&self) -> String {
        format!(
            concat!(
                "{{",
                "\"sample_rate_hz\":{},",
                "\"block_size\":{},",
                "\"processed_block_count\":{},",
                "\"last_block_sequence\":{},",
                "\"cpu_load_percent\":{},",
                "\"graph_latency_ms\":{},",
                "\"last_block_execution_time_ns\":{},",
                "\"last_block_deadline_budget_ns\":{},",
                "\"last_block_budget_utilization_percent\":{},",
                "\"last_block_budget_overrun_ns\":{},",
                "\"last_block_deadline_pressure\":\"{:?}\",",
                "\"budget_overrun_count\":{},",
                "\"peak_block_execution_time_ns\":{},",
                "\"peak_block_budget_utilization_percent\":{},",
                "\"peak_block_budget_overrun_ns\":{},",
                "\"xrun_count\":{},",
                "\"scheduler_phase_count\":{},",
                "\"scheduler_lane_count\":{},",
                "\"scheduler_dispatch_count\":{},",
                "\"scheduler_prepared_dispatch_count\":{},",
                "\"scheduler_realtime_dispatch_count\":{},",
                "\"scheduler_dispatch_handoff_count\":{},",
                "\"scheduler_topology_compatible\":{},",
                "\"scheduler_topology_requires_host_reinterpretation\":{},",
                "\"scheduler_topology_issue_count\":{},",
                "\"prework_service_state\":\"{:?}\",",
                "\"prework_service_pressure\":\"{:?}\",",
                "\"prework_service_semantic_policy\":\"{:?}\",",
                "\"pending_prework_target_count\":{},",
                "\"pending_prework_deferred_target_count\":{},",
                "\"prework_queue_depth\":{},",
                "\"prework_peak_queue_depth\":{},",
                "\"prework_service_cycle_count\":{},",
                "\"prework_service_starvation_count\":{},",
                "\"prework_service_throttle_count\":{},",
                "\"prework_service_yield_count\":{},",
                "\"last_prework_service_effective_cycles\":{},",
                "\"last_prework_service_budget_per_cycle\":{},",
                "\"last_prework_service_effective_budget_per_cycle\":{},",
                "\"last_prework_serviced_backlog_class\":{},",
                "\"transport_gate_active\":{},",
                "\"plugin_gate_active\":{},",
                "\"hot_latency_node_id\":{},",
                "\"hot_latency_node_group\":{},",
                "\"hot_latency_node_topology_role\":{},",
                "\"hot_latency_node_plugin_sandbox_id\":{},",
                "\"hot_latency_node_samples\":{},",
                "\"hot_latency_group\":{},",
                "\"hot_latency_group_node_count\":{},",
                "\"hot_latency_group_total_samples\":{},",
                "\"critical_path_lane\":{},",
                "\"critical_path_lane_node_count\":{},",
                "\"critical_path_lane_plugin_backed_node_count\":{},",
                "\"critical_path_lane_planning_group_count\":{},",
                "\"critical_path_lane_total_latency_samples\":{},",
                "\"worker_lane_summaries\":{},",
                "\"background_service_class\":{},",
                "\"background_service_decision\":{},",
                "\"background_service_reason\":{},",
                "\"background_service_priority_band\":{},",
                "\"background_service_blocking_priority_band\":{},",
                "\"background_service_backpressure_source\":{},",
                "\"background_service_starvation_risk\":{},",
                "\"background_service_starved_work_item_count\":{},",
                "\"background_service_cancellation_cause\":{},",
                "\"background_service_cancelled_work_item_count\":{},",
                "\"background_queued_work_item_count\":{},",
                "\"background_deferred_work_item_count\":{},",
                "\"background_pending_cleanup_work_item_count\":{},",
                "\"background_pending_retry_work_item_count\":{},",
                "\"summary\":{}}}",
            ),
            self.sample_rate_hz,
            self.block_size,
            self.processed_block_count,
            json_option_u64(self.last_block_sequence),
            self.cpu_load_percent,
            self.graph_latency_ms,
            json_option_u64(self.last_block_execution_time_ns),
            json_option_u64(self.last_block_deadline_budget_ns),
            json_option_f32(self.last_block_budget_utilization_percent),
            json_option_u64(self.last_block_budget_overrun_ns),
            self.last_block_deadline_pressure,
            self.budget_overrun_count,
            self.peak_block_execution_time_ns,
            self.peak_block_budget_utilization_percent,
            self.peak_block_budget_overrun_ns,
            self.xrun_count,
            self.scheduler_phase_count,
            self.scheduler_lane_count,
            self.scheduler_dispatch_count,
            self.scheduler_prepared_dispatch_count,
            self.scheduler_realtime_dispatch_count,
            self.scheduler_dispatch_handoff_count,
            self.scheduler_topology_compatible,
            self.scheduler_topology_requires_host_reinterpretation,
            self.scheduler_topology_issue_count,
            self.prework_service_state,
            self.prework_service_pressure,
            self.prework_service_semantic_policy,
            self.pending_prework_target_count,
            self.pending_prework_deferred_target_count,
            self.prework_queue_depth,
            self.prework_peak_queue_depth,
            self.prework_service_cycle_count,
            self.prework_service_starvation_count,
            self.prework_service_throttle_count,
            self.prework_service_yield_count,
            self.last_prework_service_effective_cycles,
            json_option_u64(
                self.last_prework_service_budget_per_cycle
                    .map(|value| value as u64)
            ),
            json_option_u64(
                self.last_prework_service_effective_budget_per_cycle
                    .map(|value| value as u64),
            ),
            json_option_string(self.last_prework_serviced_backlog_class.as_deref()),
            self.transport_gate_active,
            self.plugin_gate_active,
            json_option_string(self.hot_latency_node_id.as_deref()),
            json_option_string(self.hot_latency_node_group.as_deref()),
            json_option_string(self.hot_latency_node_topology_role.as_deref()),
            json_option_string(self.hot_latency_node_plugin_sandbox_id.as_deref()),
            self.hot_latency_node_samples,
            json_option_string(self.hot_latency_group.as_deref()),
            self.hot_latency_group_node_count,
            self.hot_latency_group_total_samples,
            json_option_string(self.critical_path_lane.as_deref()),
            self.critical_path_lane_node_count,
            self.critical_path_lane_plugin_backed_node_count,
            self.critical_path_lane_planning_group_count,
            self.critical_path_lane_total_latency_samples,
            json_runtime_worker_lane_instrumentation_summaries(&self.worker_lane_summaries),
            json_option_string(
                self.background_service_class
                    .as_ref()
                    .map(|value| match value {
                        RuntimeDeferredServiceClass::OfflineRenderQueue => "OfflineRenderQueue",
                        RuntimeDeferredServiceClass::OfflineRenderPurge => "OfflineRenderPurge",
                    }),
            ),
            json_option_string(self.background_service_decision.as_ref().map(
                |value| match value {
                    RuntimeDeferredServiceDecision::Run => "Run",
                    RuntimeDeferredServiceDecision::Defer => "Defer",
                    RuntimeDeferredServiceDecision::Throttle => "Throttle",
                    RuntimeDeferredServiceDecision::Abort => "Abort",
                }
            ),),
            json_option_string(
                self.background_service_reason
                    .as_ref()
                    .map(|value| match value {
                        RuntimeDeferredServiceReason::Ready => "Ready",
                        RuntimeDeferredServiceReason::RealtimeActive => "RealtimeActive",
                        RuntimeDeferredServiceReason::PendingCleanup => "PendingCleanup",
                        RuntimeDeferredServiceReason::RecoveryDegraded => "RecoveryDegraded",
                        RuntimeDeferredServiceReason::SafeMode => "SafeMode",
                        RuntimeDeferredServiceReason::InvalidRequest => "InvalidRequest",
                    }),
            ),
            json_option_string(
                self.background_service_priority_band
                    .as_ref()
                    .map(|value| format!("{value:?}"))
                    .as_deref(),
            ),
            json_option_string(
                self.background_service_blocking_priority_band
                    .as_ref()
                    .map(|value| format!("{value:?}"))
                    .as_deref(),
            ),
            json_option_string(
                self.background_service_backpressure_source
                    .as_ref()
                    .map(|value| format!("{value:?}"))
                    .as_deref(),
            ),
            self.background_service_starvation_risk,
            self.background_service_starved_work_item_count,
            json_option_string(
                self.background_service_cancellation_cause
                    .as_ref()
                    .map(|value| format!("{value:?}"))
                    .as_deref(),
            ),
            self.background_service_cancelled_work_item_count,
            self.background_queued_work_item_count,
            self.background_deferred_work_item_count,
            self.background_pending_cleanup_work_item_count,
            self.background_pending_retry_work_item_count,
            json_option_string(Some(self.summary.as_str())),
        )
    }
}

fn runtime_worker_lane_instrumentation_summaries(
    engine_block_snapshot: &RuntimeEngineBlockSnapshot,
) -> Vec<RuntimeWorkerLaneInstrumentationSummary> {
    let mut lane_order = engine_block_snapshot.lane_order.clone();
    for node in &engine_block_snapshot.planned_nodes {
        let lane = runtime_lane_for_group(node.group);
        if !lane_order.contains(&lane) {
            lane_order.push(lane);
        }
    }

    let mut summaries = Vec::new();
    for lane in lane_order {
        let mut node_count = 0usize;
        let mut plugin_backed_node_count = 0usize;
        let mut planning_groups = Vec::new();
        let mut total_latency_samples = 0u32;
        let mut max_node_latency_samples = 0u32;

        for node in engine_block_snapshot
            .planned_nodes
            .iter()
            .filter(|node| runtime_lane_for_group(node.group) == lane)
        {
            node_count = node_count.saturating_add(1);
            if matches!(node.execution_class, GraphNodeExecutionClass::PluginBacked) {
                plugin_backed_node_count = plugin_backed_node_count.saturating_add(1);
            }
            if !planning_groups.contains(&node.group) {
                planning_groups.push(node.group);
            }
            total_latency_samples = total_latency_samples.saturating_add(node.latency_samples);
            max_node_latency_samples = max_node_latency_samples.max(node.latency_samples);
        }

        if node_count > 0 {
            summaries.push(RuntimeWorkerLaneInstrumentationSummary {
                lane,
                node_count,
                plugin_backed_node_count,
                planning_group_count: planning_groups.len(),
                total_latency_samples,
                max_node_latency_samples,
            });
        }
    }

    summaries
}

fn runtime_graph_node_planning_group_name(group: GraphNodePlanningGroup) -> &'static str {
    match group {
        GraphNodePlanningGroup::InlineRealtime => "InlineRealtime",
        GraphNodePlanningGroup::StatefulRealtime => "StatefulRealtime",
        GraphNodePlanningGroup::AnticipativeEligible => "AnticipativeEligible",
    }
}

fn runtime_graph_node_topology_role_name(role: GraphNodeTopologyRole) -> &'static str {
    match role {
        GraphNodeTopologyRole::Utility => "Utility",
        GraphNodeTopologyRole::TrackLane => "TrackLane",
        GraphNodeTopologyRole::Bus => "Bus",
        GraphNodeTopologyRole::Send => "Send",
        GraphNodeTopologyRole::Return => "Return",
        GraphNodeTopologyRole::ConsoleNode => "ConsoleNode",
    }
}

pub(crate) fn runtime_execution_lane_name(lane: GraphExecutionLane) -> &'static str {
    match lane {
        GraphExecutionLane::Realtime => "Realtime",
        GraphExecutionLane::Anticipative => "Anticipative",
    }
}

fn runtime_prework_backlog_class_name(value: RuntimePreworkBacklogClass) -> &'static str {
    match value {
        RuntimePreworkBacklogClass::Immediate => "Immediate",
        RuntimePreworkBacklogClass::NearTerm => "NearTerm",
        RuntimePreworkBacklogClass::Deferred => "Deferred",
    }
}
impl RuntimeObservationReport {
    pub fn performance_snapshot(&self) -> RuntimePerformanceSnapshot {
        RuntimePerformanceSnapshot::capture(
            &self.effective_config,
            &self.diagnostics_snapshot,
            &self.engine_block_snapshot,
            self.last_deferred_service_receipt.as_ref(),
        )
    }
}

impl RuntimeSupervisorReport {
    pub fn performance_snapshot(&self) -> RuntimePerformanceSnapshot {
        self.observation.performance_snapshot()
    }
}
