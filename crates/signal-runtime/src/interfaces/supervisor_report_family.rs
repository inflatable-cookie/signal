use super::*;

/// Complete runtime state snapshot for diagnostics, testing, and logging.
///
/// Combines a full [`RuntimeObservationReport`] (all subsystem snapshots) with
/// the pending [`RuntimeEvent`] stream from a [`RuntimeEventRecorder`].  Use
/// `capture()` to build one from any observation/recorder pair, then inspect
/// with the count helpers.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeSupervisorReport {
    /// Full observation report capturing all subsystem snapshots.
    pub observation: RuntimeObservationReport,
    /// Pending event stream from the event recorder.
    pub events: Vec<RuntimeEvent>,
}

impl RuntimeSupervisorReport {
    /// Captures a full supervisor report from the current runtime and event recorder state.
    pub fn capture(runtime: &impl RuntimeObservationApi, recorder: &RuntimeEventRecorder) -> Self {
        Self {
            observation: RuntimeObservationReport::capture(runtime, recorder),
            events: recorder.snapshot(),
        }
    }

    /// Returns the total number of events in the pending event stream.
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Returns the number of supervision update events recorded.
    pub fn supervision_update_count(&self) -> usize {
        self.observation.observation.supervision_update_count()
    }

    /// Returns the total number of plugin fault events recorded.
    pub fn plugin_fault_count(&self) -> usize {
        self.observation.observation.plugin_fault_count()
    }

    /// Returns the number of plugin instance state events recorded.
    pub fn plugin_instance_state_event_count(&self) -> usize {
        self.observation
            .observation
            .plugin_instance_state_event_count()
    }

    /// Returns the number of recovery cycle events recorded.
    pub fn recovery_event_count(&self) -> usize {
        self.observation.observation.recovery_event_count()
    }

    /// Returns the number of sandbox lifecycle events recorded.
    pub fn lifecycle_event_count(&self) -> usize {
        self.observation.observation.lifecycle_event_count()
    }

    /// Returns the number of sandbox transport events recorded.
    pub fn transport_event_count(&self) -> usize {
        self.observation.observation.transport_event_count()
    }

    /// Returns the number of heartbeat cycle events recorded.
    pub fn heartbeat_event_count(&self) -> usize {
        self.observation.observation.heartbeat_event_count()
    }

    /// Returns the number of block dispatch events recorded.
    pub fn block_dispatch_event_count(&self) -> usize {
        self.observation.observation.block_dispatch_event_count()
    }

    /// Returns the number of lease rollover events recorded.
    pub fn lease_rollover_event_count(&self) -> usize {
        self.observation.observation.lease_rollover_event_count()
    }

    /// Returns the number of broker invalidation events recorded.
    pub fn invalidation_event_count(&self) -> usize {
        self.observation.observation.invalidation_event_count()
    }

    /// Returns the number of completion slot transition events recorded.
    pub fn completion_slot_event_count(&self) -> usize {
        self.observation.observation.completion_slot_event_count()
    }

    /// Returns the number of transport fault events recorded.
    pub fn transport_fault_event_count(&self) -> usize {
        self.observation.observation.transport_fault_event_count()
    }

    /// Returns the number of broker hard-failure events recorded.
    pub fn broker_failure_event_count(&self) -> usize {
        self.observation.observation.broker_failure_event_count()
    }

    /// Returns the number of sandbox operation failure events recorded.
    pub fn sandbox_operation_failure_event_count(&self) -> usize {
        self.observation
            .observation
            .sandbox_operation_failure_event_count()
    }

    /// Returns the most recent watchdog trigger, if any.
    pub fn last_watchdog_trigger(&self) -> Option<RuntimeWatchdogTrigger> {
        self.observation
            .observation
            .last_supervision_update()
            .and_then(|snapshot| snapshot.last_watchdog_trigger)
    }
}
