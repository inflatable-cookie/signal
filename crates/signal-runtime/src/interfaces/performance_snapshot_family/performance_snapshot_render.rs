use super::super::*;

impl RuntimePerformanceSnapshot {
    /// Renders a JSON object containing all performance snapshot fields.
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

/// Builds per-lane instrumentation summaries from the planned nodes in a block snapshot.
pub fn runtime_worker_lane_instrumentation_summaries(
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

/// Returns the display name string for a graph node planning group.
pub fn runtime_graph_node_planning_group_name(group: GraphNodePlanningGroup) -> &'static str {
    match group {
        GraphNodePlanningGroup::InlineRealtime => "InlineRealtime",
        GraphNodePlanningGroup::StatefulRealtime => "StatefulRealtime",
        GraphNodePlanningGroup::AnticipativeEligible => "AnticipativeEligible",
    }
}

/// Returns the display name string for a graph node topology role.
pub fn runtime_graph_node_topology_role_name(role: GraphNodeTopologyRole) -> &'static str {
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

/// Returns the display name string for a prework backlog class.
pub fn runtime_prework_backlog_class_name(value: RuntimePreworkBacklogClass) -> &'static str {
    match value {
        RuntimePreworkBacklogClass::Immediate => "Immediate",
        RuntimePreworkBacklogClass::NearTerm => "NearTerm",
        RuntimePreworkBacklogClass::Deferred => "Deferred",
    }
}

impl RuntimeObservationReport {
    /// Captures a performance snapshot from this observation report.
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
    /// Captures a performance snapshot from the observation within this supervisor report.
    pub fn performance_snapshot(&self) -> RuntimePerformanceSnapshot {
        self.observation.performance_snapshot()
    }
}
