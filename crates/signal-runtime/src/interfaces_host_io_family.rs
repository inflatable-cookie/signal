use signal_hardware::{
    AudioSampleFormat, BackendHealth, HardwareBackendIdentity, HardwareClockSource,
    HardwareClockTopology, HardwareLifecycleOwnership, HardwareRestartPolicy,
};

/// State of the host audio pump stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostAudioStreamState {
    /// The audio stream has been stopped.
    Stopped,
    /// The audio stream is actively running.
    Running,
    /// The audio stream encountered a fault.
    Faulted,
}

/// Who drives the audio callback lifecycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostLifecycleOwnership {
    /// The host drives the audio callback directly.
    HostDrivenCallback,
    /// The audio backend manages the callback lifecycle.
    BackendManagedCallback,
}

/// Who is responsible for restarting the audio backend after a fault.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostRestartPolicy {
    /// The host is responsible for restarting the backend after a fault.
    HostMustRestart,
    /// The backend may restart itself without host intervention.
    BackendMayRestart,
}

/// Clock source driving the host audio stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostClockSource {
    /// Clock is derived from the device's internal oscillator.
    Internal,
    /// Clock is locked to an external word clock signal.
    ExternalWordClock,
    /// Clock is derived from a digital input signal (e.g. S/PDIF).
    DigitalInput,
    /// Clock is virtual (no physical clock source).
    Virtual,
}

/// Clock domain relationship between the host and runtime.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostClockDomain {
    /// Host and runtime share the same clock domain.
    SameClock,
    /// Host and runtime are on different clock domains requiring resampling.
    CrossClock,
    /// Clock is sourced from an aggregate device.
    Aggregate,
    /// Clock domain relationship is degraded or unresolvable.
    Degraded,
}

/// Current clock fallback behaviour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostClockFallbackState {
    /// Clock is running directly with no fallback active.
    Direct,
    /// Runtime is resampling to bridge a clock domain mismatch.
    RuntimeResampled,
    /// Clock fallback is constrained to a recovery mode.
    RecoveryConstrained,
    /// Clock fallback state is not yet configured.
    Unconfigured,
}

/// Most recent clock topology transition event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostClockTransitionState {
    /// Clock has just been observed for the first time.
    InitialObservation,
    /// Clock topology is stable with no recent transitions.
    Stable,
    /// Transitioned into an aggregate clock arrangement.
    EnteredAggregateClock,
    /// Transitioned into a cross-clock resampling fallback.
    EnteredCrossClockFallback,
    /// Transitioned into a recovery-constrained fallback.
    EnteredRecoveryFallback,
    /// Returned to direct clocking after a fallback.
    ReturnedToDirect,
    /// Clock configuration was lost.
    LostConfiguration,
    /// Clock has been reconfigured.
    Reconfigured,
}

/// Clock drift management state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostClockDriftState {
    /// Drift is within acceptable bounds and no compensation is active.
    Stable,
    /// Drift is being managed across a cross-clock boundary.
    CrossClockManaged,
    /// Drift is being managed within an aggregate device arrangement.
    AggregateManaged,
    /// Drift compensation is actively resyncing.
    Resyncing,
    /// Drift state is not yet configured.
    Unconfigured,
}

/// Clock continuity state — whether the stream has experienced a discontinuity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostClockDiscontinuityState {
    /// Stream is continuous with no detected discontinuities.
    Continuous,
    /// Stream was reconfigured, introducing a discontinuity.
    Reconfigured,
    /// Stream is recovering from a discontinuity.
    Recovering,
    /// Stream clock configuration was lost.
    LostConfiguration,
    /// Stream encountered a fault-level discontinuity.
    Faulted,
}

/// Input/output duplex alignment state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostDuplexMismatchState {
    /// Duplex alignment is not applicable for this configuration.
    NotApplicable,
    /// Input and output endpoints are aligned on the same clock.
    Aligned,
    /// Input and output have diverged due to a cross-clock boundary.
    CrossClockDiverged,
    /// Only partial duplex availability is present.
    PartialAvailability,
    /// Duplex alignment is degraded.
    Degraded,
}

/// I/O endpoint topology of the active audio backend.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostEndpointTopology {
    /// No endpoint topology is configured.
    Unconfigured,
    /// Only output endpoints are active.
    OutputOnly,
    /// Only input endpoints are active.
    InputOnly,
    /// Both input and output endpoints are active.
    Duplex,
    /// Endpoints come from an aggregate device arrangement.
    Aggregate,
}

/// Full clocking summary for the host audio stream: clock source, domain,
/// and callback interval.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RuntimeHostClockingSummary {
    /// Clock source driving the hardware stream.
    pub clock_source: RuntimeHostClockSource,
    /// Who owns the audio callback lifecycle.
    pub ownership: RuntimeHostLifecycleOwnership,
    /// Who is responsible for backend restarts after a fault.
    pub restart_policy: RuntimeHostRestartPolicy,
    /// Sample rate used by the runtime graph (may differ from hardware).
    pub processing_sample_rate_hz: u32,
    /// Sample rate reported by the hardware device.
    pub hardware_sample_rate_hz: u32,
    /// Clock domain relationship between the host and runtime.
    pub clock_domain: RuntimeHostClockDomain,
    /// Active clock fallback mode.
    pub fallback_state: RuntimeHostClockFallbackState,
    /// Most recent clock topology transition event.
    pub transition_state: RuntimeHostClockTransitionState,
    /// Clock drift management state.
    pub drift_state: RuntimeHostClockDriftState,
    /// Whether the stream has experienced a clock discontinuity.
    pub discontinuity_state: RuntimeHostClockDiscontinuityState,
    /// Duplex alignment state between input and output endpoints.
    pub duplex_mismatch_state: RuntimeHostDuplexMismatchState,
    /// I/O endpoint topology of the active backend.
    pub endpoint_topology: RuntimeHostEndpointTopology,
    /// Whether only partial I/O availability is present.
    pub partial_availability: bool,
    /// Whether clock crossing is required for this configuration.
    pub crossing_required: bool,
    /// Host callback interval in milliseconds.
    pub callback_interval_ms: f32,
}

/// Latency summary for the host audio stream: input, output, round-trip,
/// and graph latency in both samples and milliseconds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RuntimeHostLatencySummary {
    /// Reported input latency in samples, if available.
    pub input_latency_samples: Option<u32>,
    /// Reported output latency in samples.
    pub output_latency_samples: u32,
    /// Round-trip latency in samples, if both input and output are present.
    pub round_trip_latency_samples: Option<u32>,
    /// Latency introduced by the runtime processing graph in samples.
    pub graph_latency_samples: u32,
    /// Estimated output latency including graph latency, in samples.
    pub estimated_output_latency_samples: u32,
    /// Estimated round-trip latency including graph latency, in samples.
    pub estimated_round_trip_latency_samples: Option<u32>,
    /// Reported output latency in milliseconds.
    pub output_latency_ms: f32,
    /// Graph latency in milliseconds.
    pub graph_latency_ms: f32,
    /// Estimated output latency including graph latency, in milliseconds.
    pub estimated_output_latency_ms: f32,
    /// Estimated round-trip latency including graph latency, in milliseconds.
    pub estimated_round_trip_latency_ms: Option<f32>,
}

/// Hardware summary for the active audio backend: device identity, format,
/// channel counts, and fault counters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeHostHardwareSummary {
    /// Underlying hardware backend identity.
    pub backend_identity: HardwareBackendIdentity,
    /// Human-readable name of the active audio backend.
    pub backend_name: String,
    /// Stable identifier for the active audio device.
    pub device_id: String,
    /// Human-readable name of the active audio device.
    pub device_name: String,
    /// Hardware sample rate in Hz.
    pub sample_rate: u32,
    /// Hardware buffer size in frames.
    pub buffer_size: usize,
    /// Number of available input channels.
    pub input_channels: u16,
    /// Number of available output channels.
    pub output_channels: u16,
    /// Sample format used by the hardware device.
    pub sample_format: AudioSampleFormat,
    /// Whether the backend is operating in simulated mode.
    pub simulated: bool,
    /// Health state reported by the audio backend.
    pub backend_health: BackendHealth,
    /// Total number of xruns (buffer under/overruns) observed.
    pub xrun_count: u64,
    /// Number of callback overruns where the host missed its deadline.
    pub callback_overrun_count: u64,
    /// Number of times the audio device was lost and had to be reacquired.
    pub device_loss_count: u64,
    /// Number of backend restart attempts.
    pub restart_attempt_count: u64,
    /// Number of backend restart attempts that failed.
    pub restart_failure_count: u64,
}

/// Output stream state reported by the host control surface.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeHostAudioPumpSummary {
    /// Current state of the host audio stream.
    pub stream_state: RuntimeHostAudioStreamState,
}

/// Complete host I/O summary: hardware, audio pump, clocking, and latency
/// sub-summaries.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeHostIoSummary {
    /// Hardware device and backend summary.
    pub hardware: RuntimeHostHardwareSummary,
    /// Audio pump statistics and transfer state.
    pub audio_pump: RuntimeHostAudioPumpSummary,
    /// Clocking topology and fallback state.
    pub clocking: RuntimeHostClockingSummary,
    /// Latency measurements for input, output, and round-trip.
    pub latency: RuntimeHostLatencySummary,
}

impl From<HardwareLifecycleOwnership> for RuntimeHostLifecycleOwnership {
    /// Converts a hardware lifecycle ownership value to the runtime equivalent.
    fn from(value: HardwareLifecycleOwnership) -> Self {
        match value {
            HardwareLifecycleOwnership::HostDrivenCallback => Self::HostDrivenCallback,
            HardwareLifecycleOwnership::BackendManagedCallback => Self::BackendManagedCallback,
        }
    }
}

impl From<HardwareRestartPolicy> for RuntimeHostRestartPolicy {
    /// Converts a hardware restart policy value to the runtime equivalent.
    fn from(value: HardwareRestartPolicy) -> Self {
        match value {
            HardwareRestartPolicy::HostMustRestart => Self::HostMustRestart,
            HardwareRestartPolicy::BackendMayRestart => Self::BackendMayRestart,
        }
    }
}

impl From<HardwareClockSource> for RuntimeHostClockSource {
    /// Converts a hardware clock source value to the runtime equivalent.
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
    /// Converts a hardware clock topology value to the runtime clock domain equivalent.
    fn from(value: HardwareClockTopology) -> Self {
        match value {
            HardwareClockTopology::SingleEndpoint => Self::SameClock,
            HardwareClockTopology::Aggregate => Self::Aggregate,
        }
    }
}
