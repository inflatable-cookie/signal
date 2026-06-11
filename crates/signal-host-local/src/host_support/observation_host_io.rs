use signal_hardware::{AudioSampleFormat, HardwareClockSource, HardwareLifecycleContract};
use signal_runtime::{
    RuntimeHostAudioPumpSummary, RuntimeHostClockDomain, RuntimeHostClockSource,
    RuntimeHostClockingSummary, RuntimeHostHardwareSummary, RuntimeHostIoSummary,
    RuntimeHostLatencySummary, RuntimeObservationReport,
};

use super::super::LocalRuntimeHost;
use super::{
    host_clock_discontinuity_state, host_clock_domain, host_clock_drift_state,
    host_clock_fallback_state, host_duplex_mismatch_state, host_endpoint_topology,
    host_partial_availability, samples_to_ms,
};

impl LocalRuntimeHost {
    pub(crate) fn host_io_summary(
        &self,
        observation: &RuntimeObservationReport,
    ) -> RuntimeHostIoSummary {
        let audio_pump = self.audio_pump.summary();
        let backend_diagnostics = self.hardware.diagnostics();
        let active_stream = self.active_output_stream.as_ref();
        let processing_sample_rate_hz = observation.effective_config.sample_rate.0;
        let sample_rate = active_stream
            .map(|stream| stream.sample_rate.0)
            .unwrap_or(processing_sample_rate_hz);
        let buffer_size = active_stream
            .map(|stream| stream.buffer_size)
            .unwrap_or(self.runtime.config().graph.block_size);
        let graph_latency_samples = observation.engine_block_snapshot.total_latency_samples;
        let output_latency_samples = active_stream
            .map(|stream| stream.latency.output_latency_samples)
            .unwrap_or(buffer_size as u32);
        let input_latency_samples =
            active_stream.and_then(|stream| stream.latency.input_latency_samples);
        let round_trip_latency_samples =
            active_stream.and_then(|stream| stream.latency.round_trip_latency_samples);
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
        let clock_domain = host_clock_domain(
            active_stream.map(|stream| stream.clock_topology),
            processing_sample_rate_hz,
            sample_rate,
            backend_diagnostics.health,
        );
        let fallback_state = host_clock_fallback_state(
            active_stream.is_some(),
            clock_domain,
            backend_diagnostics.health,
        );
        let transition_state =
            self.host_clock_transition_state(active_stream.is_some(), clock_domain, fallback_state);
        let endpoint_topology = host_endpoint_topology(active_stream);
        let partial_availability = host_partial_availability(active_stream);
        let drift_state = host_clock_drift_state(
            active_stream.is_some(),
            clock_domain,
            backend_diagnostics.health,
        );
        let discontinuity_state = host_clock_discontinuity_state(
            active_stream.is_some(),
            transition_state,
            backend_diagnostics.health,
            audio_pump.stream_state.into(),
        );
        let duplex_mismatch_state = host_duplex_mismatch_state(
            active_stream,
            clock_domain,
            backend_diagnostics.health,
            audio_pump.stream_state.into(),
            partial_availability,
        );
        let callback_interval_ms = samples_to_ms(buffer_size as u32, sample_rate);
        RuntimeHostIoSummary {
            hardware: RuntimeHostHardwareSummary {
                backend_identity: self.hardware.backend_identity(),
                backend_name: self.hardware.backend_name().into(),
                device_id: active_stream
                    .as_ref()
                    .map(|stream| stream.device.device_id.clone())
                    .unwrap_or_else(|| "coreaudio:unconfigured".into()),
                device_name: active_stream
                    .as_ref()
                    .map(|stream| stream.device.name.clone())
                    .unwrap_or_else(|| "Unconfigured Device".into()),
                sample_rate,
                buffer_size,
                input_channels: active_stream
                    .as_ref()
                    .map(|stream| stream.input_channels)
                    .unwrap_or_default(),
                output_channels: active_stream
                    .as_ref()
                    .map(|stream| stream.output_channels)
                    .unwrap_or_default(),
                sample_format: active_stream
                    .as_ref()
                    .map(|stream| stream.sample_format)
                    .unwrap_or(AudioSampleFormat::F32),
                simulated: active_stream
                    .as_ref()
                    .map(|stream| stream.simulated)
                    .unwrap_or(false),
                backend_health: backend_diagnostics.health,
                xrun_count: backend_diagnostics.xrun_count,
                callback_overrun_count: backend_diagnostics.callback_overrun_count,
                device_loss_count: backend_diagnostics.device_loss_count,
                restart_attempt_count: backend_diagnostics.restart_attempt_count,
                restart_failure_count: backend_diagnostics.restart_failure_count,
            },
            audio_pump: RuntimeHostAudioPumpSummary {
                stream_state: audio_pump.stream_state.into(),
                transfer_policy: audio_pump.transfer_policy.into(),
                callback_count: audio_pump.callback_count,
                total_callback_frames: audio_pump.total_callback_frames,
                total_runtime_output_frames: audio_pump.total_runtime_output_frames,
                copied_output_samples: audio_pump.copied_output_samples,
                zero_filled_output_samples: audio_pump.zero_filled_output_samples,
                dropped_output_samples: audio_pump.dropped_output_samples,
                last_callback_output_peak: audio_pump.last_callback_output_peak,
                last_runtime_graph_id: audio_pump.last_runtime_graph_id.clone(),
            },
            clocking: RuntimeHostClockingSummary {
                clock_source: active_stream
                    .map(|stream| RuntimeHostClockSource::from(stream.clock_source))
                    .unwrap_or(RuntimeHostClockSource::from(HardwareClockSource::Internal)),
                ownership: active_stream
                    .map(|stream| stream.lifecycle.ownership.into())
                    .unwrap_or(HardwareLifecycleContract::default().ownership.into()),
                restart_policy: active_stream
                    .map(|stream| stream.lifecycle.restart_policy.into())
                    .unwrap_or(HardwareLifecycleContract::default().restart_policy.into()),
                processing_sample_rate_hz,
                hardware_sample_rate_hz: sample_rate,
                clock_domain,
                fallback_state,
                transition_state,
                drift_state,
                discontinuity_state,
                duplex_mismatch_state,
                endpoint_topology,
                partial_availability,
                crossing_required: matches!(
                    clock_domain,
                    RuntimeHostClockDomain::CrossClock | RuntimeHostClockDomain::Aggregate
                ),
                callback_interval_ms,
            },
            latency: RuntimeHostLatencySummary {
                input_latency_samples,
                output_latency_samples,
                round_trip_latency_samples,
                graph_latency_samples,
                estimated_output_latency_samples,
                estimated_round_trip_latency_samples,
                output_latency_ms: samples_to_ms(output_latency_samples, sample_rate),
                graph_latency_ms: samples_to_ms(graph_latency_samples, sample_rate),
                estimated_output_latency_ms: samples_to_ms(
                    estimated_output_latency_samples,
                    sample_rate,
                ),
                estimated_round_trip_latency_ms: estimated_round_trip_latency_samples
                    .map(|samples| samples_to_ms(samples, sample_rate)),
            },
            runtime_graph_id_matches_pump: audio_pump.last_runtime_graph_id.as_deref()
                == observation.engine_block_snapshot.graph_id.as_deref(),
        }
    }
}
