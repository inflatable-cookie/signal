use super::*;

#[derive(Clone, Debug, PartialEq)]
/// Combines a [`RuntimeObservationReport`] with the host audio I/O summary
/// captured in the same tick.  Used when the host pump is in scope and
/// hardware metrics should be included alongside runtime diagnostics.
pub struct RuntimeHostObservationReport {
    /// Runtime observation report captured in this tick.
    pub observation: RuntimeObservationReport,
    /// Host audio I/O summary captured alongside the observation report.
    pub host_io: RuntimeHostIoSummary,
}

impl RuntimeHostObservationReport {
    /// Constructs a report from separate observation and host I/O summaries.
    pub fn new(observation: RuntimeObservationReport, host_io: RuntimeHostIoSummary) -> Self {
        Self {
            observation,
            host_io,
        }
    }
}
