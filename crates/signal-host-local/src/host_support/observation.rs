use signal_runtime::{
    RuntimeHostIoSummary, RuntimeObservationDiagnostics, RuntimeObservationReport,
    RuntimeSupervisorReport,
};

use super::super::LocalRuntimeHost;

impl LocalRuntimeHost {
    #[allow(dead_code)]
    pub fn observation_diagnostics(&self) -> RuntimeObservationDiagnostics {
        self.events.diagnostics()
    }

    #[allow(dead_code)]
    pub fn observation_report(&self) -> RuntimeObservationReport {
        self.observation_with_host_io().0
    }

    pub fn supervisor_report(&self) -> RuntimeSupervisorReport {
        self.supervisor_with_host_io().0
    }

    pub(crate) fn observation_with_host_io(
        &self,
    ) -> (RuntimeObservationReport, RuntimeHostIoSummary) {
        let observation = RuntimeObservationReport::capture(&self.runtime, &self.events);
        let host_io = self.host_io_summary(&observation);
        let external_midi_snapshot =
            signal_runtime::RuntimeExternalMidiEndpointGraphSnapshot::empty("signal-host-local");
        let observation = observation
            .with_host_device_supervision(&host_io)
            .with_host_external_io(&host_io)
            .with_linux_backend_session_snapshot(&host_io)
            .with_pipewire_alsa_parity_snapshot(&host_io)
            .with_jack_coordination_snapshot(&host_io)
            .with_external_midi_snapshot(external_midi_snapshot);
        (observation, host_io)
    }

    pub(crate) fn supervisor_with_host_io(
        &self,
    ) -> (RuntimeSupervisorReport, RuntimeHostIoSummary) {
        let mut supervisor = RuntimeSupervisorReport::capture(&self.runtime, &self.events);
        let host_io = self.host_io_summary(&supervisor.observation);
        let external_midi_snapshot =
            signal_runtime::RuntimeExternalMidiEndpointGraphSnapshot::empty("signal-host-local");
        supervisor.observation = supervisor
            .observation
            .clone()
            .with_host_device_supervision(&host_io)
            .with_host_external_io(&host_io)
            .with_linux_backend_session_snapshot(&host_io)
            .with_pipewire_alsa_parity_snapshot(&host_io)
            .with_jack_coordination_snapshot(&host_io)
            .with_external_midi_snapshot(external_midi_snapshot);
        (supervisor, host_io)
    }
}
