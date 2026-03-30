use super::super::*;

impl RuntimeObservationDiagnostics {
    pub fn render_compact(&self) -> String {
        let last_trigger = self
            .last_supervision_update()
            .and_then(|snapshot| snapshot.last_watchdog_trigger)
            .map(|trigger| format!("{trigger:?}"))
            .unwrap_or_else(|| "none".into());
        let last_fault = self
            .plugin_faults
            .last()
            .map(|fault| format!("{}:{:?}", fault.sandbox_id, fault.kind))
            .unwrap_or_else(|| "none".into());
        let last_recovery = self
            .last_recovery_event()
            .map(|recovery| {
                format!(
                    "{}:{:?}:{:?}@{:?}",
                    recovery.sandbox_id,
                    recovery.intent,
                    recovery.stop_reason,
                    recovery.processing_epoch
                )
            })
            .unwrap_or_else(|| "none".into());
        let last_plugin_instance_state = self
            .last_plugin_instance_state()
            .map(|state| {
                format!(
                    "{}:{}:{}/{}/active={}@{:?}",
                    state.sandbox_id,
                    state.instance_id,
                    state.lifecycle_state,
                    state.readiness_state,
                    state.active,
                    state.processing_epoch
                )
            })
            .unwrap_or_else(|| "none".into());
        let last_lifecycle = self
            .last_lifecycle_event()
            .map(|lifecycle| {
                format!(
                    "{}:{:?}@{:?}",
                    lifecycle.sandbox_id, lifecycle.stage, lifecycle.processing_epoch
                )
            })
            .unwrap_or_else(|| "none".into());
        let last_transport = self
            .last_transport_event()
            .map(|transport| {
                format!(
                    "{}:{}:{}:{:?}@{:?}",
                    transport.sandbox_id,
                    transport.lease_id,
                    transport.region_id,
                    transport.stage,
                    transport.processing_epoch
                )
            })
            .unwrap_or_else(|| "none".into());
        let last_heartbeat = self
            .last_heartbeat_event()
            .map(|heartbeat| {
                format!(
                    "{}:{:?}@{:?}/block={:?}",
                    heartbeat.sandbox_id,
                    heartbeat.stage,
                    heartbeat.processing_epoch,
                    heartbeat.block_sequence
                )
            })
            .unwrap_or_else(|| "none".into());
        let last_dispatch = self
            .last_block_dispatch_event()
            .map(|dispatch| {
                format!(
                    "{}:{}:{:?}/block={}@{}",
                    dispatch.sandbox_id,
                    dispatch.lease_id,
                    dispatch.stage,
                    dispatch.block_sequence,
                    dispatch.processing_epoch
                )
            })
            .unwrap_or_else(|| "none".into());
        let last_rollover = self
            .last_lease_rollover_event()
            .map(|rollover| {
                format!(
                    "{}:{}->{}@{}/block={}",
                    rollover.sandbox_id,
                    rollover.previous_lease_id,
                    rollover.lease_id,
                    rollover.processing_epoch,
                    rollover.first_block_sequence
                )
            })
            .unwrap_or_else(|| "none".into());
        let last_invalidation = self
            .last_invalidation_event()
            .map(|invalidation| {
                format!(
                    "{}:{}:{:?}@{}/block={:?}",
                    invalidation.sandbox_id,
                    invalidation.lease_id,
                    invalidation.stage,
                    invalidation.processing_epoch,
                    invalidation.block_sequence
                )
            })
            .unwrap_or_else(|| "none".into());
        let last_completion_slot = self
            .last_completion_slot_event()
            .map(|completion| {
                format!(
                    "{}:{}:{:?}@{}/block={}",
                    completion.sandbox_id,
                    completion.lease_id,
                    completion.stage,
                    completion.processing_epoch,
                    completion.block_sequence
                )
            })
            .unwrap_or_else(|| "none".into());
        let last_transport_fault = self
            .last_transport_fault_event()
            .map(|failure| {
                format!(
                    "{}:{:?}:{:?}:{:?}:{:?}:lease={:?}@{:?}/block={:?}",
                    failure.sandbox_id,
                    failure.source,
                    failure.stage,
                    failure.phase,
                    failure.resource,
                    failure.lease_id,
                    failure.processing_epoch,
                    failure.block_sequence
                )
            })
            .unwrap_or_else(|| "none".into());
        let last_broker_failure = self
            .last_broker_failure_event()
            .map(|failure| {
                format!(
                    "{}:{:?}:lease={:?}@{:?}/block={:?}",
                    failure.sandbox_id,
                    failure.stage,
                    failure.lease_id,
                    failure.processing_epoch,
                    failure.block_sequence
                )
            })
            .unwrap_or_else(|| "none".into());
        let last_sandbox_operation_failure = self
            .last_sandbox_operation_failure_event()
            .map(|failure| {
                format!(
                    "{}:{}:{:?}:lease={:?}@{:?}",
                    failure.sandbox_id,
                    failure.operation,
                    failure.stage,
                    failure.lease_id,
                    failure.processing_epoch
                )
            })
            .unwrap_or_else(|| "none".into());

        format!(
            "events={} supervision_updates={} plugin_faults={} plugin_instance_states={} recovery_events={} lifecycle_events={} transport_events={} heartbeat_events={} block_dispatch_events={} lease_rollover_events={} invalidation_events={} completion_slot_events={} transport_fault_events={} broker_failure_events={} sandbox_operation_failure_events={} last_watchdog={} last_fault={} last_plugin_instance_state={} last_recovery={} last_lifecycle={} last_transport={} last_heartbeat={} last_dispatch={} last_rollover={} last_invalidation={} last_completion_slot={} last_transport_fault={} last_broker_failure={} last_sandbox_operation_failure={}",
            self.total_events,
            self.supervision_update_count(),
            self.plugin_fault_count(),
            self.plugin_instance_state_event_count(),
            self.recovery_event_count(),
            self.lifecycle_event_count(),
            self.transport_event_count(),
            self.heartbeat_event_count(),
            self.block_dispatch_event_count(),
            self.lease_rollover_event_count(),
            self.invalidation_event_count(),
            self.completion_slot_event_count(),
            self.transport_fault_event_count(),
            self.broker_failure_event_count(),
            self.sandbox_operation_failure_event_count(),
            last_trigger,
            last_fault,
            last_plugin_instance_state,
            last_recovery,
            last_lifecycle,
            last_transport,
            last_heartbeat,
            last_dispatch,
            last_rollover,
            last_invalidation,
            last_completion_slot,
            last_transport_fault,
            last_broker_failure,
            last_sandbox_operation_failure,
        )
    }
}
