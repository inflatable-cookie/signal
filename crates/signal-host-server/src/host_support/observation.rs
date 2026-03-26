use signal_hardware::{HardwareBackend, HardwareStreamRequest, SimulatedHardwareBackend};
use signal_runtime::{
    RuntimeExternalMidiEndpointGraphSnapshot, RuntimeHostAudioPumpSummary,
    RuntimeHostAudioStreamState, RuntimeHostAudioTransferPolicy,
    RuntimeHostClockDiscontinuityState, RuntimeHostClockDomain, RuntimeHostClockDriftState,
    RuntimeHostClockFallbackState, RuntimeHostClockTransitionState, RuntimeHostClockingSummary,
    RuntimeHostDuplexMismatchState, RuntimeHostEndpointTopology, RuntimeHostHardwareSummary,
    RuntimeHostIoSummary, RuntimeHostLatencySummary, RuntimeObservationDiagnostics,
    RuntimeObservationReport, RuntimeSupervisorReport,
};

use super::super::ServerRuntimeHost;
use super::{
    runtime_host_clock_source, runtime_host_lifecycle_ownership, runtime_host_restart_policy,
};

impl ServerRuntimeHost {
    #[allow(dead_code)]
    pub fn observation_diagnostics(&self) -> RuntimeObservationDiagnostics {
        self.events.diagnostics()
    }

    #[allow(dead_code)]
    pub fn observation_report(&self) -> RuntimeObservationReport {
        let observation = RuntimeObservationReport::capture(&self.runtime, &self.events);
        let host_io = self.host_io_summary(&observation);
        let jack_host_io = self.jack_host_io_summary(&observation);
        observation
            .with_linux_backend_session_snapshot(&host_io)
            .with_pipewire_alsa_parity_snapshot(&host_io)
            .with_jack_coordination_snapshot(&jack_host_io)
            .with_external_midi_snapshot(RuntimeExternalMidiEndpointGraphSnapshot::empty(
                "signal-host-server",
            ))
    }

    pub fn supervisor_report(&self) -> RuntimeSupervisorReport {
        let mut report = RuntimeSupervisorReport::capture(&self.runtime, &self.events);
        let host_io = self.host_io_summary(&report.observation);
        let jack_host_io = self.jack_host_io_summary(&report.observation);
        report.observation = report
            .observation
            .with_linux_backend_session_snapshot(&host_io)
            .with_pipewire_alsa_parity_snapshot(&host_io)
            .with_jack_coordination_snapshot(&jack_host_io)
            .with_external_midi_snapshot(RuntimeExternalMidiEndpointGraphSnapshot::empty(
                "signal-host-server",
            ));
        report
    }

    pub(crate) fn host_io_summary(
        &self,
        observation: &RuntimeObservationReport,
    ) -> RuntimeHostIoSummary {
        self.simulated_linux_host_io_summary(
            observation,
            SimulatedHardwareBackend::linux_pipewire_duplex(),
            HardwareStreamRequest::new_output("pipewire:default-graph", 48_000, 256)
                .with_input_channels(2)
                .with_output_channels(2),
        )
    }

    pub(crate) fn jack_host_io_summary(
        &self,
        observation: &RuntimeObservationReport,
    ) -> RuntimeHostIoSummary {
        self.simulated_linux_host_io_summary(
            observation,
            SimulatedHardwareBackend::linux_jack_duplex(),
            HardwareStreamRequest::new_output("jack:graph-main", 48_000, 256)
                .with_input_channels(2)
                .with_output_channels(2),
        )
    }

    pub(crate) fn simulated_linux_host_io_summary(
        &self,
        observation: &RuntimeObservationReport,
        backend: SimulatedHardwareBackend,
        request: HardwareStreamRequest,
    ) -> RuntimeHostIoSummary {
        let diagnostics = backend.diagnostics();
        let stream = backend
            .negotiate_stream(&request)
            .expect("server host linux baseline should negotiate");
        let graph_latency_samples = observation.engine_block_snapshot.total_latency_samples;
        let output_latency_samples = stream.latency.output_latency_samples;
        let input_latency_samples = stream.latency.input_latency_samples;
        let round_trip_latency_samples = stream.latency.round_trip_latency_samples;
        let estimated_output_latency_samples =
            output_latency_samples.saturating_add(graph_latency_samples);
        let estimated_round_trip_latency_samples =
            match (input_latency_samples, round_trip_latency_samples) {
                (_, Some(round_trip)) => Some(round_trip.saturating_add(graph_latency_samples)),
                (Some(input_latency), None) => Some(
                    input_latency
                        .saturating_add(output_latency_samples)
                        .saturating_add(graph_latency_samples),
                ),
                (None, None) => None,
            };
        let endpoint_topology = RuntimeHostEndpointTopology::Duplex;
        RuntimeHostIoSummary {
            hardware: RuntimeHostHardwareSummary {
                backend_identity: stream.device.backend_identity,
                backend_name: stream.device.backend_name.into(),
                linux_backend_identity: RuntimeHostHardwareSummary::classify_linux_backend_identity(
                    stream.device.backend_identity,
                ),
                linux_backend_portability:
                    RuntimeHostHardwareSummary::classify_linux_backend_portability(
                        stream.device.backend_identity,
                        stream.simulated,
                        diagnostics.health,
                        diagnostics.device_loss_count,
                        diagnostics.restart_attempt_count,
                        diagnostics.restart_failure_count,
                    ),
                device_id: stream.device.device_id.clone(),
                device_name: stream.device.name.clone(),
                sample_rate: stream.sample_rate.0,
                buffer_size: stream.buffer_size,
                input_channels: stream.input_channels,
                output_channels: stream.output_channels,
                sample_format: stream.sample_format,
                simulated: stream.simulated,
                backend_health: diagnostics.health,
                xrun_count: diagnostics.xrun_count,
                callback_overrun_count: diagnostics.callback_overrun_count,
                device_loss_count: diagnostics.device_loss_count,
                restart_attempt_count: diagnostics.restart_attempt_count,
                restart_failure_count: diagnostics.restart_failure_count,
            },
            audio_pump: RuntimeHostAudioPumpSummary {
                stream_state: RuntimeHostAudioStreamState::Running,
                transfer_policy: RuntimeHostAudioTransferPolicy {
                    max_callback_frames: stream.buffer_size,
                    max_transfer_channels: stream.input_channels.max(stream.output_channels),
                    zero_fill_unwritten_output: true,
                },
                callback_count: 0,
                total_callback_frames: 0,
                total_runtime_output_frames: 0,
                copied_output_samples: 0,
                zero_filled_output_samples: 0,
                dropped_output_samples: 0,
                last_callback_output_peak: None,
                last_runtime_graph_id: None,
            },
            clocking: RuntimeHostClockingSummary {
                clock_source: runtime_host_clock_source(stream.clock_source),
                ownership: runtime_host_lifecycle_ownership(stream.lifecycle.ownership),
                restart_policy: runtime_host_restart_policy(stream.lifecycle.restart_policy),
                processing_sample_rate_hz: observation.effective_config.sample_rate.0,
                hardware_sample_rate_hz: stream.sample_rate.0,
                clock_domain: RuntimeHostClockDomain::Aggregate,
                fallback_state: RuntimeHostClockFallbackState::Direct,
                transition_state: RuntimeHostClockTransitionState::Stable,
                drift_state: RuntimeHostClockDriftState::AggregateManaged,
                discontinuity_state: RuntimeHostClockDiscontinuityState::Continuous,
                duplex_mismatch_state: RuntimeHostDuplexMismatchState::Aligned,
                endpoint_topology,
                linux_clocking_parity:
                    signal_runtime::RuntimeHostIoSummary::classify_linux_clocking_parity(
                        RuntimeHostHardwareSummary::classify_linux_backend_identity(
                            stream.device.backend_identity,
                        ),
                        diagnostics.health,
                        RuntimeHostAudioStreamState::Running,
                        RuntimeHostClockDomain::Aggregate,
                        RuntimeHostClockFallbackState::Direct,
                        RuntimeHostClockTransitionState::Stable,
                        RuntimeHostClockDriftState::AggregateManaged,
                        RuntimeHostClockDiscontinuityState::Continuous,
                    ),
                linux_duplex_parity:
                    signal_runtime::RuntimeHostIoSummary::classify_linux_duplex_parity(
                        RuntimeHostHardwareSummary::classify_linux_backend_identity(
                            stream.device.backend_identity,
                        ),
                        diagnostics.health,
                        RuntimeHostAudioStreamState::Running,
                        RuntimeHostClockDomain::Aggregate,
                        RuntimeHostClockFallbackState::Direct,
                        RuntimeHostClockTransitionState::Stable,
                        RuntimeHostDuplexMismatchState::Aligned,
                        endpoint_topology,
                        false,
                    ),
                linux_endpoint_topology_parity:
                    signal_runtime::RuntimeHostIoSummary::classify_linux_endpoint_topology_parity(
                        RuntimeHostHardwareSummary::classify_linux_backend_identity(
                            stream.device.backend_identity,
                        ),
                        diagnostics.health,
                        RuntimeHostClockTransitionState::Stable,
                        RuntimeHostClockDiscontinuityState::Continuous,
                        RuntimeHostDuplexMismatchState::Aligned,
                        endpoint_topology,
                        false,
                    ),
                partial_availability: false,
                crossing_required: false,
                callback_interval_ms: super::super::samples_to_ms(
                    stream.buffer_size as u32,
                    stream.sample_rate.0,
                ),
            },
            latency: RuntimeHostLatencySummary {
                input_latency_samples,
                output_latency_samples,
                round_trip_latency_samples,
                graph_latency_samples,
                estimated_output_latency_samples,
                estimated_round_trip_latency_samples,
                output_latency_ms: super::super::samples_to_ms(
                    output_latency_samples,
                    stream.sample_rate.0,
                ),
                graph_latency_ms: super::super::samples_to_ms(
                    graph_latency_samples,
                    stream.sample_rate.0,
                ),
                estimated_output_latency_ms: super::super::samples_to_ms(
                    estimated_output_latency_samples,
                    stream.sample_rate.0,
                ),
                estimated_round_trip_latency_ms: estimated_round_trip_latency_samples
                    .map(|value| super::super::samples_to_ms(value, stream.sample_rate.0)),
            },
            runtime_graph_id_matches_pump: false,
        }
    }
}
