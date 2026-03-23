use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeDegradationSummary {
    pub readiness_degraded: bool,
    pub safe_mode_enabled: bool,
    pub xrun_count: u64,
    pub plugin_fault_count: usize,
    pub transport_fault_event_count: usize,
    pub broker_failure_event_count: usize,
    pub sandbox_operation_failure_event_count: usize,
    pub recovery_event_count: usize,
    pub active_plugin_sandboxes: u32,
    pub degraded_bound_plugin_sandboxes: usize,
    pub missing_bound_plugin_sandboxes: usize,
    pub recovery_overlap_sessions: usize,
    pub lingering_sessions: usize,
    pub detach_faulted_sessions: usize,
    pub transport_gate_active: bool,
    pub plugin_gate_active: bool,
    pub last_watchdog_trigger: Option<RuntimeWatchdogTrigger>,
}

impl RuntimeDegradationSummary {
    pub fn capture(
        readiness: &RuntimeReadiness,
        diagnostics_snapshot: RuntimeDiagnosticsSnapshot,
        supervision_snapshot: &RuntimeSupervisionSnapshot,
        engine_block_snapshot: &RuntimeEngineBlockSnapshot,
        transport_concurrency_snapshot: &RuntimeTransportConcurrencySnapshot,
        observation: &RuntimeObservationDiagnostics,
    ) -> Self {
        Self {
            readiness_degraded: matches!(readiness, RuntimeReadiness::Degraded { .. }),
            safe_mode_enabled: supervision_snapshot.safe_mode_enabled,
            xrun_count: diagnostics_snapshot.xruns,
            plugin_fault_count: observation.plugin_fault_count(),
            transport_fault_event_count: observation.transport_fault_event_count(),
            broker_failure_event_count: observation.broker_failure_event_count(),
            sandbox_operation_failure_event_count: observation
                .sandbox_operation_failure_event_count(),
            recovery_event_count: observation.recovery_event_count(),
            active_plugin_sandboxes: diagnostics_snapshot.active_plugin_sandboxes,
            degraded_bound_plugin_sandboxes: engine_block_snapshot
                .prework_service_degraded_bound_plugin_sandboxes,
            missing_bound_plugin_sandboxes: engine_block_snapshot
                .prework_service_missing_bound_plugin_sandboxes,
            recovery_overlap_sessions: engine_block_snapshot
                .prework_service_recovery_overlap_sessions,
            lingering_sessions: engine_block_snapshot.prework_service_lingering_sessions,
            detach_faulted_sessions: transport_concurrency_snapshot.current_detach_faulted_sessions,
            transport_gate_active: engine_block_snapshot.prework_service_transport_gate_active,
            plugin_gate_active: engine_block_snapshot.prework_service_plugin_gate_active,
            last_watchdog_trigger: observation
                .last_supervision_update()
                .and_then(|snapshot| snapshot.last_watchdog_trigger)
                .or(supervision_snapshot.last_watchdog_trigger),
        }
    }
}
