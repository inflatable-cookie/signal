use signal_runtime::{
    RuntimeHostIoSummary, RuntimeObservationDiagnostics, RuntimeObservationReport,
    RuntimeSupervisorReport,
};

use super::super::LocalRuntimeHost;

impl LocalRuntimeHost {
    #[allow(dead_code)]
    /// Returns raw observation diagnostics captured from the event log.
    pub fn observation_diagnostics(&self) -> RuntimeObservationDiagnostics {
        self.events.diagnostics()
    }

    #[allow(dead_code)]
    /// Returns a full observation report enriched with host I/O state.
    pub fn observation_report(&self) -> RuntimeObservationReport {
        self.observation_with_host_io().0
    }

    /// Returns a supervisor report enriched with host I/O state.
    pub fn supervisor_report(&self) -> RuntimeSupervisorReport {
        self.supervisor_with_host_io().0
    }

    pub(crate) fn observation_with_host_io(
        &self,
    ) -> (RuntimeObservationReport, RuntimeHostIoSummary) {
        let observation = RuntimeObservationReport::capture(&self.runtime, &self.events);
        let host_io = self.host_io_summary(&observation);
        let observation = observation
            .with_host_device_supervision(&host_io)
            .with_host_external_io(&host_io);
        (observation, host_io)
    }

    pub(crate) fn supervisor_with_host_io(
        &self,
    ) -> (RuntimeSupervisorReport, RuntimeHostIoSummary) {
        let mut supervisor = RuntimeSupervisorReport::capture(&self.runtime, &self.events);
        let host_io = self.host_io_summary(&supervisor.observation);
        supervisor.observation = supervisor
            .observation
            .clone()
            .with_host_device_supervision(&host_io)
            .with_host_external_io(&host_io);
        (supervisor, host_io)
    }
}
