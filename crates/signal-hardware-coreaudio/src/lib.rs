//! CoreAudio backend shell for Signal.

use signal_hardware::{
    AudioDeviceDescriptor, BackendHealth, BackendPolicyRecord, BackendPolicyTier, HardwareBackend,
    HardwareClockSource, HardwareClockTopology, HardwareConfigRequest, HardwareDiagnosticEvent,
    HardwareDiagnosticKind, HardwareDiagnosticSeverity, HardwareDiagnosticsSnapshot,
    HardwareLatencyProfile, HardwareLifecycleContract, HardwareLifecycleOwnership,
    HardwareNegotiationError, HardwareRestartPolicy, HardwareStreamConfig, HardwareStreamRequest,
    SampleRate,
};

const DEFAULT_OUTPUT_DEVICE_ID: &str = "coreaudio:default-output";
const DEFAULT_OUTPUT_DEVICE_NAME: &str = "CoreAudio Default Output";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoreAudioBackend {
    policy_tier: BackendPolicyTier,
    diagnostics: HardwareDiagnosticsSnapshot,
}

impl Default for CoreAudioBackend {
    fn default() -> Self {
        Self {
            policy_tier: BackendPolicyTier::Tier0InHost,
            diagnostics: HardwareDiagnosticsSnapshot::healthy(),
        }
    }
}

impl CoreAudioBackend {
    pub fn policy_tier(&self) -> BackendPolicyTier {
        self.policy_tier
    }

    fn default_output_descriptor(&self) -> AudioDeviceDescriptor {
        AudioDeviceDescriptor {
            backend_name: self.backend_name(),
            device_id: DEFAULT_OUTPUT_DEVICE_ID.into(),
            name: DEFAULT_OUTPUT_DEVICE_NAME.into(),
            default_input: false,
            default_output: true,
            max_input_channels: 0,
            max_output_channels: 2,
            nominal_sample_rate: SampleRate(48_000),
            preferred_buffer_sizes: vec![128, 256, 512],
        }
    }

    pub fn default_output_stream(
        &self,
        sample_rate: u32,
        buffer_size: usize,
    ) -> Result<HardwareStreamConfig, HardwareNegotiationError> {
        let request =
            HardwareStreamRequest::new_output(DEFAULT_OUTPUT_DEVICE_ID, sample_rate, buffer_size);
        self.negotiate_stream(&request)
    }

    pub fn default_output_request(
        &self,
        sample_rate: u32,
        buffer_size: usize,
    ) -> HardwareConfigRequest {
        let stream = self
            .default_output_stream(sample_rate, buffer_size)
            .expect("default CoreAudio output stream should be negotiable");
        HardwareConfigRequest::from_stream(&stream, self.policy_tier)
    }

    pub fn reset_diagnostics(&mut self) {
        self.diagnostics = HardwareDiagnosticsSnapshot::healthy();
    }

    pub fn mark_recovered(&mut self) {
        self.diagnostics.health = BackendHealth::Healthy;
    }

    pub fn simulate_device_loss(&mut self, detail: impl Into<String>) {
        self.diagnostics.health = BackendHealth::Degraded;
        self.diagnostics.device_loss_count = self.diagnostics.device_loss_count.saturating_add(1);
        self.diagnostics.last_event = Some(HardwareDiagnosticEvent {
            kind: HardwareDiagnosticKind::DeviceDisconnected,
            severity: HardwareDiagnosticSeverity::Critical,
            device_id: Some(DEFAULT_OUTPUT_DEVICE_ID.into()),
            callback_index: None,
            detail: detail.into(),
        });
    }

    pub fn simulate_restart_attempt(&mut self, detail: impl Into<String>) {
        self.diagnostics.health = BackendHealth::Recovering;
        self.diagnostics.restart_attempt_count =
            self.diagnostics.restart_attempt_count.saturating_add(1);
        self.diagnostics.last_event = Some(HardwareDiagnosticEvent {
            kind: HardwareDiagnosticKind::RestartAttempted,
            severity: HardwareDiagnosticSeverity::Info,
            device_id: Some(DEFAULT_OUTPUT_DEVICE_ID.into()),
            callback_index: None,
            detail: detail.into(),
        });
    }

    pub fn simulate_restart_failure(&mut self, detail: impl Into<String>) {
        self.diagnostics.health = BackendHealth::Degraded;
        self.diagnostics.restart_failure_count =
            self.diagnostics.restart_failure_count.saturating_add(1);
        self.diagnostics.last_event = Some(HardwareDiagnosticEvent {
            kind: HardwareDiagnosticKind::RestartFailed,
            severity: HardwareDiagnosticSeverity::Critical,
            device_id: Some(DEFAULT_OUTPUT_DEVICE_ID.into()),
            callback_index: None,
            detail: detail.into(),
        });
    }
}

impl HardwareBackend for CoreAudioBackend {
    fn backend_name(&self) -> &'static str {
        "coreaudio"
    }

    fn policy_record(&self) -> BackendPolicyRecord {
        BackendPolicyRecord {
            tier: self.policy_tier,
            in_host_default: true,
        }
    }

    fn health(&self) -> BackendHealth {
        self.diagnostics.health
    }

    fn enumerate_devices(&self) -> Vec<AudioDeviceDescriptor> {
        vec![self.default_output_descriptor()]
    }

    fn default_output_device(&self) -> Option<AudioDeviceDescriptor> {
        Some(self.default_output_descriptor())
    }

    fn negotiate_stream(
        &self,
        request: &HardwareStreamRequest,
    ) -> Result<HardwareStreamConfig, HardwareNegotiationError> {
        let device = self
            .enumerate_devices()
            .into_iter()
            .find(|device| device.device_id == request.device_id)
            .ok_or_else(|| {
                signal_hardware::HardwareNegotiationError::new(
                    signal_hardware::HardwareNegotiationErrorKind::DeviceUnavailable,
                    format!("unknown CoreAudio device {}", request.device_id),
                )
            })?;

        if request.buffer_size == 0 || request.sample_rate.0 == 0 {
            return Err(signal_hardware::HardwareNegotiationError::new(
                signal_hardware::HardwareNegotiationErrorKind::UnsupportedConfiguration,
                "sample_rate and buffer_size must be non-zero",
            ));
        }
        if request.output_channels == 0 || request.output_channels > device.max_output_channels {
            return Err(signal_hardware::HardwareNegotiationError::new(
                signal_hardware::HardwareNegotiationErrorKind::UnsupportedConfiguration,
                format!(
                    "requested {} output channels exceeds CoreAudio shell capacity",
                    request.output_channels
                ),
            ));
        }
        if request.input_channels > device.max_input_channels {
            return Err(signal_hardware::HardwareNegotiationError::new(
                signal_hardware::HardwareNegotiationErrorKind::UnsupportedConfiguration,
                format!(
                    "requested {} input channels exceeds CoreAudio shell capacity",
                    request.input_channels
                ),
            ));
        }

        Ok(HardwareStreamConfig {
            device,
            direction: request.direction,
            sample_rate: request.sample_rate,
            buffer_size: request.buffer_size,
            input_channels: request.input_channels,
            output_channels: request.output_channels,
            sample_format: request.sample_format,
            interleaved: request.interleaved,
            clock_source: HardwareClockSource::Internal,
            clock_topology: HardwareClockTopology::SingleEndpoint,
            lifecycle: HardwareLifecycleContract {
                ownership: HardwareLifecycleOwnership::HostDrivenCallback,
                restart_policy: HardwareRestartPolicy::HostMustRestart,
            },
            latency: HardwareLatencyProfile::output_only(request.buffer_size as u32),
            simulated: false,
        })
    }

    fn diagnostics(&self) -> HardwareDiagnosticsSnapshot {
        self.diagnostics.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use signal_hardware::{AudioSampleFormat, AudioStreamDirection};

    #[test]
    fn coreaudio_backend_exposes_default_output_device_and_stream_contract() {
        let backend = CoreAudioBackend::default();

        let device = backend
            .default_output_device()
            .expect("coreaudio default output device");
        assert_eq!(device.device_id, DEFAULT_OUTPUT_DEVICE_ID);
        assert!(device.default_output);

        let stream = backend
            .default_output_stream(48_000, 256)
            .expect("coreaudio default output stream");
        assert_eq!(stream.device.device_id, DEFAULT_OUTPUT_DEVICE_ID);
        assert_eq!(stream.direction, AudioStreamDirection::Output);
        assert_eq!(stream.sample_rate, SampleRate(48_000));
        assert_eq!(stream.buffer_size, 256);
        assert_eq!(stream.output_channels, 2);
        assert_eq!(stream.sample_format, AudioSampleFormat::F32);
        assert_eq!(stream.clock_source, HardwareClockSource::Internal);
        assert_eq!(stream.clock_topology, HardwareClockTopology::SingleEndpoint);
        assert_eq!(stream.latency, HardwareLatencyProfile::output_only(256));
        assert_eq!(
            stream.lifecycle,
            HardwareLifecycleContract {
                ownership: HardwareLifecycleOwnership::HostDrivenCallback,
                restart_policy: HardwareRestartPolicy::HostMustRestart,
            }
        );
        assert!(!stream.simulated);
    }

    #[test]
    fn coreaudio_backend_tracks_simulated_device_loss_and_restart_diagnostics() {
        let mut backend = CoreAudioBackend::default();

        backend.simulate_device_loss("simulated device disconnect");
        backend.simulate_restart_attempt("simulated restart attempt");
        backend.simulate_restart_failure("simulated restart failure");

        let diagnostics = backend.diagnostics();
        assert_eq!(diagnostics.device_loss_count, 1);
        assert_eq!(diagnostics.restart_attempt_count, 1);
        assert_eq!(diagnostics.restart_failure_count, 1);
        assert_eq!(diagnostics.health, BackendHealth::Degraded);
        assert!(diagnostics
            .last_event
            .is_some_and(|event| event.kind == HardwareDiagnosticKind::RestartFailed));

        backend.mark_recovered();
        assert_eq!(backend.health(), BackendHealth::Healthy);
    }
}
