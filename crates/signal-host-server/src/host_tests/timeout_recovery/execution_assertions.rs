use signal_runtime::{
    RuntimePreworkCacheState, RuntimePreworkForecastMode, RuntimePreworkFreshnessState,
    RuntimePreworkServiceSemanticPolicy, RuntimeSupervisorReport,
};

use crate::host_support::ServerRuntimeHostSummary;

pub(super) fn assert_timeout_recovery_execution(
    summary: &ServerRuntimeHostSummary,
    supervisor: &RuntimeSupervisorReport,
) {
    let plugin_state = summary
        .execution
        .last_plugin_state
        .as_ref()
        .expect("plugin instance state should be projected into server summary");
    assert_eq!(plugin_state.plugin_type_id, "plugin:clap:server");
    assert_eq!(plugin_state.instance_id, "instance:server:default");
    assert_eq!(plugin_state.lifecycle_state, "Active");
    assert_eq!(plugin_state.readiness_state, "Ready");
    assert!(plugin_state.active);
    assert_eq!(plugin_state.processing_sample_rate_hz, Some(48_000));
    assert_eq!(plugin_state.processing_max_block_frames, Some(512));
    assert!(plugin_state.last_fault.is_none());

    let observed_plugin_state = supervisor
        .observation
        .observation
        .last_plugin_instance_state()
        .expect("runtime observation should retain typed plugin state");
    assert_eq!(observed_plugin_state.instance_id, "instance:server:default");
    assert_eq!(observed_plugin_state.lifecycle_state, "Active");
    assert_eq!(observed_plugin_state.readiness_state, "Ready");
    assert!(supervisor
        .render_json()
        .contains("\"plugin_instance_state_events\":"));
    assert!(summary.execution.last_engine_output_peak.unwrap_or_default() <= 0.7);
    assert!(summary.execution.last_engine_output_rms.unwrap_or_default() > 0.0);

    let snapshot = &supervisor.observation.engine_block_snapshot;
    assert_eq!(
        snapshot
            .last_execution_context
            .as_ref()
            .map(|context| context.projection_epoch),
        Some(1)
    );
    assert_eq!(
        snapshot
            .last_execution_context
            .as_ref()
            .map(|context| context.transport_playing),
        Some(true)
    );
    assert!(
        snapshot
            .last_execution_context
            .as_ref()
            .map(|context| context.timeline_position_samples)
            .unwrap_or_default()
            > 0
    );
    assert_eq!(snapshot.node_count, 3);
    assert_eq!(snapshot.stateful_node_count, 2);
    assert_eq!(snapshot.latency_node_count, 1);
    assert_eq!(snapshot.plugin_backed_node_count, 1);
    assert!(!snapshot.anticipative_planning_enabled);
    assert_eq!(snapshot.inline_realtime_node_count, 1);
    assert_eq!(snapshot.stateful_realtime_node_count, 2);
    assert_eq!(snapshot.anticipative_eligible_node_count, 0);
    assert_eq!(
        snapshot.prework_service_semantic_policy,
        RuntimePreworkServiceSemanticPolicy::Balanced
    );
    assert_eq!(snapshot.prework_service_active_plugin_sandboxes, 1);
    assert_eq!(snapshot.prework_service_bound_plugin_sandboxes, 1);
    assert_eq!(snapshot.prework_service_active_bound_plugin_sandboxes, 1);
    assert_eq!(snapshot.prework_service_degraded_bound_plugin_sandboxes, 0);
    assert_eq!(snapshot.prework_service_missing_bound_plugin_sandboxes, 0);
    assert!(snapshot.planned_nodes.iter().any(|node| {
        node.node_id == "drive"
            && node.plugin_sandbox_id.as_deref() == Some("server-default-sandbox")
    }));
    assert_eq!(snapshot.phase_count, 2);
    assert_eq!(snapshot.anticipative_phase_count, 0);
    assert_eq!(snapshot.lane_count, 1);
    assert_eq!(snapshot.anticipative_lane_count, 0);
    assert_eq!(snapshot.dispatch_count, 1);
    assert_eq!(snapshot.dispatch_boundary_count, 0);
    assert_eq!(snapshot.prepared_dispatch_count, 0);
    assert_eq!(snapshot.realtime_dispatch_count, 1);
    assert_eq!(snapshot.dispatch_handoff_count, 0);
    assert!(!snapshot.prework_cache_enabled);
    assert_eq!(
        snapshot.prework_forecast_requested_mode,
        RuntimePreworkForecastMode::RuntimeRoleDefault
    );
    assert_eq!(snapshot.prework_forecast_mode, RuntimePreworkForecastMode::Disabled);
    assert_eq!(snapshot.prework_cache_state, RuntimePreworkCacheState::Disabled);
    assert_eq!(
        snapshot.prework_cache_freshness_state,
        RuntimePreworkFreshnessState::Disabled
    );
    assert_eq!(snapshot.prework_cache_admissions, 0);
    assert_eq!(snapshot.prework_cache_consumptions, 0);
    assert_eq!(snapshot.prework_cache_retirement_count, 0);
    assert_eq!(snapshot.prework_cache_hits, 0);
    assert_eq!(snapshot.prework_cache_misses, 0);
    assert_eq!(snapshot.last_prework_output_peak, None);
    assert_eq!(snapshot.last_prework_admission_processing_epoch, None);
    assert_eq!(snapshot.last_prework_admission_block_sequence, None);
    assert_eq!(snapshot.last_prework_consumption_processing_epoch, None);
    assert_eq!(snapshot.last_prework_consumption_block_sequence, None);
    assert_eq!(snapshot.last_prework_retirement_reason, None);
    assert_eq!(snapshot.last_prework_retired_unconsumed, None);
    assert_eq!(snapshot.prework_cache_valid_until_block_sequence, None);
    assert!(snapshot.last_realtime_input_peak.is_some());
    assert_eq!(snapshot.total_latency_samples, 32);
}
