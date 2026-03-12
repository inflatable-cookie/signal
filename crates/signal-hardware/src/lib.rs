//! Common hardware and device abstractions for Signal.

pub use signal_primitives::SampleRate;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendPolicyTier {
    Tier0InHost,
    Tier1Brokered,
    Tier2StrongContainment,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioStreamDirection {
    Input,
    Output,
    Duplex,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioSampleFormat {
    F32,
    I16,
    I32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HardwareClockSource {
    Internal,
    ExternalWordClock,
    DigitalInput,
    Virtual,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HardwareClockTopology {
    SingleEndpoint,
    Aggregate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HardwareLifecycleOwnership {
    HostDrivenCallback,
    BackendManagedCallback,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HardwareRestartPolicy {
    HostMustRestart,
    BackendMayRestart,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HardwareDiagnosticSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HardwareDiagnosticKind {
    Xrun,
    CallbackOverrun,
    DeviceDisconnected,
    RestartAttempted,
    RestartFailed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioDeviceDescriptor {
    pub backend_name: &'static str,
    pub device_id: String,
    pub name: String,
    pub default_input: bool,
    pub default_output: bool,
    pub max_input_channels: u16,
    pub max_output_channels: u16,
    pub nominal_sample_rate: SampleRate,
    pub preferred_buffer_sizes: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HardwareStreamRequest {
    pub device_id: String,
    pub direction: AudioStreamDirection,
    pub sample_rate: SampleRate,
    pub buffer_size: usize,
    pub input_channels: u16,
    pub output_channels: u16,
    pub sample_format: AudioSampleFormat,
    pub interleaved: bool,
}

impl HardwareStreamRequest {
    pub fn new_output(device_id: impl Into<String>, sample_rate: u32, buffer_size: usize) -> Self {
        Self {
            device_id: device_id.into(),
            direction: AudioStreamDirection::Output,
            sample_rate: SampleRate(sample_rate),
            buffer_size,
            input_channels: 0,
            output_channels: 2,
            sample_format: AudioSampleFormat::F32,
            interleaved: true,
        }
    }

    pub fn with_output_channels(mut self, output_channels: u16) -> Self {
        self.output_channels = output_channels;
        self
    }

    pub fn with_input_channels(mut self, input_channels: u16) -> Self {
        self.input_channels = input_channels;
        self
    }

    pub fn with_sample_format(mut self, sample_format: AudioSampleFormat) -> Self {
        self.sample_format = sample_format;
        self
    }

    pub fn with_interleaved(mut self, interleaved: bool) -> Self {
        self.interleaved = interleaved;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HardwareLifecycleContract {
    pub ownership: HardwareLifecycleOwnership,
    pub restart_policy: HardwareRestartPolicy,
}

impl Default for HardwareLifecycleContract {
    fn default() -> Self {
        Self {
            ownership: HardwareLifecycleOwnership::HostDrivenCallback,
            restart_policy: HardwareRestartPolicy::HostMustRestart,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HardwareLatencyProfile {
    pub input_latency_samples: Option<u32>,
    pub output_latency_samples: u32,
    pub round_trip_latency_samples: Option<u32>,
}

impl HardwareLatencyProfile {
    pub fn output_only(output_latency_samples: u32) -> Self {
        Self {
            input_latency_samples: None,
            output_latency_samples,
            round_trip_latency_samples: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HardwareStreamConfig {
    pub device: AudioDeviceDescriptor,
    pub direction: AudioStreamDirection,
    pub sample_rate: SampleRate,
    pub buffer_size: usize,
    pub input_channels: u16,
    pub output_channels: u16,
    pub sample_format: AudioSampleFormat,
    pub interleaved: bool,
    pub clock_source: HardwareClockSource,
    pub clock_topology: HardwareClockTopology,
    pub lifecycle: HardwareLifecycleContract,
    pub latency: HardwareLatencyProfile,
    pub simulated: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HardwareConfigRequest {
    pub sample_rate: SampleRate,
    pub buffer_size: usize,
    pub backend_policy: BackendPolicyTier,
    pub input_channels: u16,
    pub output_channels: u16,
    pub sample_format: AudioSampleFormat,
    pub interleaved: bool,
}

impl HardwareConfigRequest {
    pub fn new(sample_rate: u32, buffer_size: usize, backend_policy: BackendPolicyTier) -> Self {
        Self {
            sample_rate: SampleRate(sample_rate),
            buffer_size,
            backend_policy,
            input_channels: 0,
            output_channels: 2,
            sample_format: AudioSampleFormat::F32,
            interleaved: true,
        }
    }

    pub fn from_stream(stream: &HardwareStreamConfig, backend_policy: BackendPolicyTier) -> Self {
        Self {
            sample_rate: stream.sample_rate,
            buffer_size: stream.buffer_size,
            backend_policy,
            input_channels: stream.input_channels,
            output_channels: stream.output_channels,
            sample_format: stream.sample_format,
            interleaved: stream.interleaved,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BackendPolicyRecord {
    pub tier: BackendPolicyTier,
    pub in_host_default: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendHealth {
    Healthy,
    Degraded,
    Recovering,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HardwareDiagnosticEvent {
    pub kind: HardwareDiagnosticKind,
    pub severity: HardwareDiagnosticSeverity,
    pub device_id: Option<String>,
    pub callback_index: Option<u64>,
    pub detail: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HardwareDiagnosticsSnapshot {
    pub health: BackendHealth,
    pub xrun_count: u64,
    pub callback_overrun_count: u64,
    pub device_loss_count: u64,
    pub restart_attempt_count: u64,
    pub restart_failure_count: u64,
    pub last_event: Option<HardwareDiagnosticEvent>,
}

impl HardwareDiagnosticsSnapshot {
    pub fn healthy() -> Self {
        Self {
            health: BackendHealth::Healthy,
            xrun_count: 0,
            callback_overrun_count: 0,
            device_loss_count: 0,
            restart_attempt_count: 0,
            restart_failure_count: 0,
            last_event: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HardwareNegotiationErrorKind {
    DeviceUnavailable,
    UnsupportedConfiguration,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HardwareNegotiationError {
    pub kind: HardwareNegotiationErrorKind,
    pub message: String,
}

impl HardwareNegotiationError {
    pub fn new(kind: HardwareNegotiationErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

pub trait HardwareBackend {
    fn backend_name(&self) -> &'static str;
    fn policy_record(&self) -> BackendPolicyRecord;
    fn health(&self) -> BackendHealth;
    fn enumerate_devices(&self) -> Vec<AudioDeviceDescriptor>;
    fn default_output_device(&self) -> Option<AudioDeviceDescriptor>;
    fn negotiate_stream(
        &self,
        request: &HardwareStreamRequest,
    ) -> Result<HardwareStreamConfig, HardwareNegotiationError>;
    fn diagnostics(&self) -> HardwareDiagnosticsSnapshot;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimulatedHardwareBackend {
    backend_name: &'static str,
    policy_tier: BackendPolicyTier,
    devices: Vec<AudioDeviceDescriptor>,
    lifecycle: HardwareLifecycleContract,
    diagnostics: HardwareDiagnosticsSnapshot,
}

impl SimulatedHardwareBackend {
    pub fn new(
        backend_name: &'static str,
        policy_tier: BackendPolicyTier,
        devices: Vec<AudioDeviceDescriptor>,
    ) -> Self {
        Self {
            backend_name,
            policy_tier,
            devices,
            lifecycle: HardwareLifecycleContract::default(),
            diagnostics: HardwareDiagnosticsSnapshot::healthy(),
        }
    }

    pub fn default_stereo_output(backend_name: &'static str, device_id: &str, name: &str) -> Self {
        Self::new(
            backend_name,
            BackendPolicyTier::Tier0InHost,
            vec![AudioDeviceDescriptor {
                backend_name,
                device_id: device_id.into(),
                name: name.into(),
                default_input: false,
                default_output: true,
                max_input_channels: 0,
                max_output_channels: 2,
                nominal_sample_rate: SampleRate(48_000),
                preferred_buffer_sizes: vec![128, 256, 512],
            }],
        )
    }

    pub fn with_lifecycle(mut self, lifecycle: HardwareLifecycleContract) -> Self {
        self.lifecycle = lifecycle;
        self
    }

    pub fn with_diagnostics(mut self, diagnostics: HardwareDiagnosticsSnapshot) -> Self {
        self.diagnostics = diagnostics;
        self
    }
}

impl HardwareBackend for SimulatedHardwareBackend {
    fn backend_name(&self) -> &'static str {
        self.backend_name
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
        self.devices.clone()
    }

    fn default_output_device(&self) -> Option<AudioDeviceDescriptor> {
        self.devices
            .iter()
            .find(|device| device.default_output)
            .cloned()
    }

    fn negotiate_stream(
        &self,
        request: &HardwareStreamRequest,
    ) -> Result<HardwareStreamConfig, HardwareNegotiationError> {
        let device = self
            .devices
            .iter()
            .find(|device| device.device_id == request.device_id)
            .cloned()
            .ok_or_else(|| {
                HardwareNegotiationError::new(
                    HardwareNegotiationErrorKind::DeviceUnavailable,
                    format!("unknown device {}", request.device_id),
                )
            })?;

        if request.buffer_size == 0 || request.sample_rate.0 == 0 {
            return Err(HardwareNegotiationError::new(
                HardwareNegotiationErrorKind::UnsupportedConfiguration,
                "sample_rate and buffer_size must be non-zero",
            ));
        }
        if request.output_channels > device.max_output_channels
            || request.input_channels > device.max_input_channels
        {
            return Err(HardwareNegotiationError::new(
                HardwareNegotiationErrorKind::UnsupportedConfiguration,
                format!(
                    "requested {} input and {} output channels exceeds device capacity",
                    request.input_channels, request.output_channels
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
            clock_source: HardwareClockSource::Virtual,
            clock_topology: HardwareClockTopology::SingleEndpoint,
            lifecycle: self.lifecycle,
            latency: HardwareLatencyProfile::output_only(request.buffer_size as u32),
            simulated: true,
        })
    }

    fn diagnostics(&self) -> HardwareDiagnosticsSnapshot {
        self.diagnostics.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simulated_backend_negotiates_default_output_stream_and_runtime_request() {
        let backend = SimulatedHardwareBackend::default_stereo_output(
            "simulated",
            "sim:default-output",
            "Simulated Output",
        )
        .with_lifecycle(HardwareLifecycleContract {
            ownership: HardwareLifecycleOwnership::HostDrivenCallback,
            restart_policy: HardwareRestartPolicy::BackendMayRestart,
        });

        let device = backend
            .default_output_device()
            .expect("default output device available");
        let request = HardwareStreamRequest::new_output(device.device_id.clone(), 48_000, 256)
            .with_output_channels(2);
        let stream = backend
            .negotiate_stream(&request)
            .expect("negotiate simulated output stream");

        assert_eq!(stream.device.device_id, "sim:default-output");
        assert_eq!(stream.direction, AudioStreamDirection::Output);
        assert_eq!(stream.sample_rate, SampleRate(48_000));
        assert_eq!(stream.buffer_size, 256);
        assert_eq!(stream.output_channels, 2);
        assert_eq!(stream.sample_format, AudioSampleFormat::F32);
        assert!(stream.interleaved);
        assert_eq!(stream.clock_source, HardwareClockSource::Virtual);
        assert_eq!(stream.clock_topology, HardwareClockTopology::SingleEndpoint);
        assert_eq!(
            stream.lifecycle,
            HardwareLifecycleContract {
                ownership: HardwareLifecycleOwnership::HostDrivenCallback,
                restart_policy: HardwareRestartPolicy::BackendMayRestart,
            }
        );
        assert_eq!(stream.latency, HardwareLatencyProfile::output_only(256));
        assert!(stream.simulated);

        let runtime_request =
            HardwareConfigRequest::from_stream(&stream, backend.policy_record().tier);
        assert_eq!(runtime_request.sample_rate, SampleRate(48_000));
        assert_eq!(runtime_request.buffer_size, 256);
        assert_eq!(runtime_request.output_channels, 2);
        assert_eq!(
            runtime_request.backend_policy,
            BackendPolicyTier::Tier0InHost
        );
    }

    #[test]
    fn simulated_backend_surfaces_diagnostics_contract() {
        let diagnostics = HardwareDiagnosticsSnapshot {
            health: BackendHealth::Degraded,
            xrun_count: 3,
            callback_overrun_count: 1,
            device_loss_count: 1,
            restart_attempt_count: 2,
            restart_failure_count: 1,
            last_event: Some(HardwareDiagnosticEvent {
                kind: HardwareDiagnosticKind::RestartFailed,
                severity: HardwareDiagnosticSeverity::Critical,
                device_id: Some("sim:default-output".into()),
                callback_index: Some(42),
                detail: "simulated restart failure".into(),
            }),
        };
        let backend = SimulatedHardwareBackend::default_stereo_output(
            "simulated",
            "sim:default-output",
            "Simulated Output",
        )
        .with_diagnostics(diagnostics.clone());

        assert_eq!(backend.health(), BackendHealth::Degraded);
        assert_eq!(backend.diagnostics(), diagnostics);
    }
}
