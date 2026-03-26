use super::*;

impl RuntimeEventRecorder {
    pub fn diagnostics(&self) -> RuntimeObservationDiagnostics {
        RuntimeObservationDiagnostics {
            total_events: self.count(),
            supervision_updates: self.supervision_updates(),
            plugin_faults: self.plugin_faults(),
            plugin_instance_states: self.plugin_instance_states(),
            recovery_events: self.recovery_events(),
            lifecycle_events: self.lifecycle_events(),
            transport_events: self.transport_events(),
            heartbeat_events: self.heartbeat_events(),
            block_dispatch_events: self.block_dispatch_events(),
            lease_rollover_events: self.lease_rollover_events(),
            invalidation_events: self.invalidation_events(),
            completion_slot_events: self.completion_slot_events(),
            transport_fault_events: self.transport_fault_events(),
            broker_failure_events: self.broker_failure_events(),
            sandbox_operation_failure_events: self.sandbox_operation_failure_events(),
        }
    }
}
