use super::*;

/// Host-scoped supervisor report: [`RuntimeHostObservationReport`] plus the
/// accumulated event stream.  Extends [`RuntimeSupervisorReport`] with hardware
/// I/O context; built via `RuntimeHostSupervisorReport::new()`.
pub struct RuntimeHostSupervisorReport {
    /// The combined observation and host I/O data for this cycle.
    pub observation: RuntimeHostObservationReport,
    /// Accumulated runtime events captured during this observation window.
    pub events: Vec<RuntimeEvent>,
}

impl RuntimeHostSupervisorReport {
    /// Constructs a host supervisor report from a base supervisor report and a host I/O summary.
    pub fn new(supervisor: RuntimeSupervisorReport, host_io: RuntimeHostIoSummary) -> Self {
        Self {
            observation: RuntimeHostObservationReport::new(supervisor.observation, host_io),
            events: supervisor.events,
        }
    }
}
