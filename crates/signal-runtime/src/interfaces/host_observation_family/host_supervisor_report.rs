use super::*;

/// Host-scoped supervisor report: [`RuntimeHostObservationReport`] plus the
/// accumulated event stream.  Extends [`RuntimeSupervisorReport`] with hardware
/// I/O context; built via `RuntimeHostSupervisorReport::new()`.  Provides
/// `profiling_receipt()` and `soak_receipt()` that include host hardware fields.
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

    /// Returns a profiling receipt including host hardware fields.
    pub fn profiling_receipt(&self) -> RuntimeProfilingReceipt {
        build_runtime_profiling_receipt(
            &self.observation.observation,
            Some(&self.observation.host_io),
        )
    }

    /// Returns a soak receipt derived from this host supervisor report.
    pub fn soak_receipt(&self) -> RuntimeSoakReceipt {
        build_runtime_soak_receipt(RuntimeSoakReceiptInput {
            observation: &self.observation.observation,
            event_stream_count: self.events.len(),
        })
    }
}
