use super::*;
use signal_hardware::{
    AudioSampleFormat, BackendHealth, HardwareBackendIdentity, HardwareClockSource,
    HardwareClockTopology, HardwareLifecycleOwnership, HardwareRestartPolicy,
    LinuxAudioBackendKind,
};

#[path = "interfaces_host_io_family/parity.rs"]
mod parity;

pub use parity::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostAudioStreamState {
    Stopped,
    Running,
    Faulted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeHostAudioTransferPolicy {
    pub max_callback_frames: usize,
    pub max_transfer_channels: u16,
    pub zero_fill_unwritten_output: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostLifecycleOwnership {
    HostDrivenCallback,
    BackendManagedCallback,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostRestartPolicy {
    HostMustRestart,
    BackendMayRestart,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostClockSource {
    Internal,
    ExternalWordClock,
    DigitalInput,
    Virtual,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostClockDomain {
    SameClock,
    CrossClock,
    Aggregate,
    Degraded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostClockFallbackState {
    Direct,
    RuntimeResampled,
    RecoveryConstrained,
    Unconfigured,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostClockTransitionState {
    InitialObservation,
    Stable,
    EnteredAggregateClock,
    EnteredCrossClockFallback,
    EnteredRecoveryFallback,
    ReturnedToDirect,
    LostConfiguration,
    Reconfigured,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostClockDriftState {
    Stable,
    CrossClockManaged,
    AggregateManaged,
    Resyncing,
    Unconfigured,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostClockDiscontinuityState {
    Continuous,
    Reconfigured,
    Recovering,
    LostConfiguration,
    Faulted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostDuplexMismatchState {
    NotApplicable,
    Aligned,
    CrossClockDiverged,
    PartialAvailability,
    Degraded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostEndpointTopology {
    Unconfigured,
    OutputOnly,
    InputOnly,
    Duplex,
    Aggregate,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RuntimeHostClockingSummary {
    pub clock_source: RuntimeHostClockSource,
    pub ownership: RuntimeHostLifecycleOwnership,
    pub restart_policy: RuntimeHostRestartPolicy,
    pub processing_sample_rate_hz: u32,
    pub hardware_sample_rate_hz: u32,
    pub clock_domain: RuntimeHostClockDomain,
    pub fallback_state: RuntimeHostClockFallbackState,
    pub transition_state: RuntimeHostClockTransitionState,
    pub drift_state: RuntimeHostClockDriftState,
    pub discontinuity_state: RuntimeHostClockDiscontinuityState,
    pub duplex_mismatch_state: RuntimeHostDuplexMismatchState,
    pub endpoint_topology: RuntimeHostEndpointTopology,
    pub linux_clocking_parity: RuntimeLinuxAudioBackendClockingParityBand,
    pub linux_duplex_parity: RuntimeLinuxAudioBackendDuplexParityState,
    pub linux_endpoint_topology_parity: RuntimeLinuxAudioBackendEndpointTopologyParityState,
    pub partial_availability: bool,
    pub crossing_required: bool,
    pub callback_interval_ms: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RuntimeHostLatencySummary {
    pub input_latency_samples: Option<u32>,
    pub output_latency_samples: u32,
    pub round_trip_latency_samples: Option<u32>,
    pub graph_latency_samples: u32,
    pub estimated_output_latency_samples: u32,
    pub estimated_round_trip_latency_samples: Option<u32>,
    pub output_latency_ms: f32,
    pub graph_latency_ms: f32,
    pub estimated_output_latency_ms: f32,
    pub estimated_round_trip_latency_ms: Option<f32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeHostHardwareSummary {
    pub backend_identity: HardwareBackendIdentity,
    pub backend_name: String,
    pub linux_backend_identity: RuntimeLinuxAudioBackendIdentity,
    pub linux_backend_portability: RuntimeLinuxAudioBackendPortabilityBand,
    pub device_id: String,
    pub device_name: String,
    pub sample_rate: u32,
    pub buffer_size: usize,
    pub input_channels: u16,
    pub output_channels: u16,
    pub sample_format: AudioSampleFormat,
    pub simulated: bool,
    pub backend_health: BackendHealth,
    pub xrun_count: u64,
    pub callback_overrun_count: u64,
    pub device_loss_count: u64,
    pub restart_attempt_count: u64,
    pub restart_failure_count: u64,
}

impl RuntimeHostHardwareSummary {
    pub fn classify_linux_backend_identity(
        backend_identity: HardwareBackendIdentity,
    ) -> RuntimeLinuxAudioBackendIdentity {
        match backend_identity {
            HardwareBackendIdentity::CoreAudio => RuntimeLinuxAudioBackendIdentity::NotLinux,
            HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa) => {
                RuntimeLinuxAudioBackendIdentity::Alsa
            }
            HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack) => {
                RuntimeLinuxAudioBackendIdentity::Jack
            }
            HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire) => {
                RuntimeLinuxAudioBackendIdentity::PipeWire
            }
            HardwareBackendIdentity::Unsupported => RuntimeLinuxAudioBackendIdentity::Unsupported,
        }
    }

    pub fn classify_linux_backend_portability(
        backend_identity: HardwareBackendIdentity,
        simulated: bool,
        backend_health: BackendHealth,
        device_loss_count: u64,
        restart_attempt_count: u64,
        restart_failure_count: u64,
    ) -> RuntimeLinuxAudioBackendPortabilityBand {
        match Self::classify_linux_backend_identity(backend_identity) {
            RuntimeLinuxAudioBackendIdentity::Alsa
            | RuntimeLinuxAudioBackendIdentity::Jack
            | RuntimeLinuxAudioBackendIdentity::PipeWire => {
                if simulated
                    || !matches!(backend_health, BackendHealth::Healthy)
                    || device_loss_count > 0
                    || restart_attempt_count > 0
                    || restart_failure_count > 0
                {
                    RuntimeLinuxAudioBackendPortabilityBand::Guarded
                } else {
                    RuntimeLinuxAudioBackendPortabilityBand::Portable
                }
            }
            RuntimeLinuxAudioBackendIdentity::NotLinux
            | RuntimeLinuxAudioBackendIdentity::Unavailable
            | RuntimeLinuxAudioBackendIdentity::Unsupported => {
                RuntimeLinuxAudioBackendPortabilityBand::Unsupported
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeHostAudioPumpSummary {
    pub stream_state: RuntimeHostAudioStreamState,
    pub transfer_policy: RuntimeHostAudioTransferPolicy,
    pub callback_count: u64,
    pub total_callback_frames: u64,
    pub total_runtime_output_frames: u64,
    pub copied_output_samples: u64,
    pub zero_filled_output_samples: u64,
    pub dropped_output_samples: u64,
    pub last_callback_output_peak: Option<f32>,
    pub last_runtime_graph_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeHostIoSummary {
    pub hardware: RuntimeHostHardwareSummary,
    pub audio_pump: RuntimeHostAudioPumpSummary,
    pub clocking: RuntimeHostClockingSummary,
    pub latency: RuntimeHostLatencySummary,
    pub runtime_graph_id_matches_pump: bool,
}

impl From<HardwareLifecycleOwnership> for RuntimeHostLifecycleOwnership {
    fn from(value: HardwareLifecycleOwnership) -> Self {
        match value {
            HardwareLifecycleOwnership::HostDrivenCallback => Self::HostDrivenCallback,
            HardwareLifecycleOwnership::BackendManagedCallback => Self::BackendManagedCallback,
        }
    }
}

impl From<HardwareRestartPolicy> for RuntimeHostRestartPolicy {
    fn from(value: HardwareRestartPolicy) -> Self {
        match value {
            HardwareRestartPolicy::HostMustRestart => Self::HostMustRestart,
            HardwareRestartPolicy::BackendMayRestart => Self::BackendMayRestart,
        }
    }
}

impl From<HardwareClockSource> for RuntimeHostClockSource {
    fn from(value: HardwareClockSource) -> Self {
        match value {
            HardwareClockSource::Internal => Self::Internal,
            HardwareClockSource::ExternalWordClock => Self::ExternalWordClock,
            HardwareClockSource::DigitalInput => Self::DigitalInput,
            HardwareClockSource::Virtual => Self::Virtual,
        }
    }
}

impl From<HardwareClockTopology> for RuntimeHostClockDomain {
    fn from(value: HardwareClockTopology) -> Self {
        match value {
            HardwareClockTopology::SingleEndpoint => Self::SameClock,
            HardwareClockTopology::Aggregate => Self::Aggregate,
        }
    }
}
