use super::*;

/// Overall health of the external audio I/O subsystem.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalIoHealthState {
    /// No device is configured or available.
    Unavailable,
    /// Device is running normally.
    Ready,
    /// A clock fallback is active (e.g. cross-clock resampling).
    FallbackActive,
    /// Device is recovering from a transient fault.
    Recovering,
    /// Device has faulted and cannot recover without intervention.
    Faulted,
}

/// Device change / recovery state of the external I/O device.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalIoDeviceChangeState {
    /// No device is configured.
    Unavailable,
    /// Device is running and stable.
    Stable,
    /// Device lost; a restart is queued.
    PendingRestart,
    /// Device is being restarted.
    Recovering,
    /// Restart attempts exhausted; device cannot be recovered.
    Failed,
}

/// Primary audio I/O role of the active device.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalIoPrimaryRole {
    /// No device is available.
    Unavailable,
    /// Device provides program audio output only.
    ProgramOutput,
    /// Device provides external audio input only.
    ExternalInput,
    /// Device provides full-duplex program audio I/O.
    ProgramDuplex,
    /// Device is an aggregate of multiple physical devices.
    AggregateProgram,
}

/// State of the external monitoring path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalIoMonitoringState {
    /// Monitoring path is not available.
    Unavailable,
    /// Monitoring is direct (no resampling or fallback).
    Direct,
    /// Monitoring is active but guarded (e.g. aggregate or partial availability).
    Guarded,
    /// Monitoring is degraded (backend is unhealthy).
    Degraded,
    /// Monitoring has faulted.
    Faulted,
}

/// Where in the signal chain the external monitoring tap is inserted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalIoMonitoringTapPoint {
    /// No tap point is available.
    Unavailable,
    /// Tap is before the graph's first processing node (external input path).
    PreGraphInput,
    /// Tap is after the runtime's final output node.
    PostRuntimeOutput,
    /// Tap is after the hardware output converter.
    PostHardwareOutput,
    /// Tap is after the hardware loopback return.
    PostLoopbackReturn,
}

/// State of the hardware loopback path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalIoLoopbackState {
    /// Loopback is not available (no duplex device).
    Unavailable,
    /// Loopback is ready and low-latency.
    Ready,
    /// Loopback is available but guarded (aggregate or partial availability).
    Guarded,
    /// Loopback is recovering from a transient fault.
    Recovering,
    /// Loopback has faulted.
    Faulted,
}

/// Full snapshot of the external audio I/O subsystem including device, clock,
/// monitoring, loopback, and fault counter fields.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeExternalIoSnapshot {
    /// Overall health classification of the external I/O subsystem.
    pub health_state: RuntimeExternalIoHealthState,
    /// Device change and recovery state.
    pub device_change_state: RuntimeExternalIoDeviceChangeState,
    /// Primary signal flow role of the active device.
    pub primary_role: RuntimeExternalIoPrimaryRole,
    /// State of the external monitoring path.
    pub monitoring_state: RuntimeExternalIoMonitoringState,
    /// Location in the signal chain where the monitoring tap is inserted.
    pub monitoring_tap_point: RuntimeExternalIoMonitoringTapPoint,
    /// State of the hardware loopback path.
    pub loopback_state: RuntimeExternalIoLoopbackState,
    /// Multichannel I/O layout for the active device.
    pub io_layout: RuntimeMultichannelIoSummary,
    /// Name of the active audio backend.
    pub backend_name: String,
    /// Device identifier string of the active output device.
    pub active_output_device_id: String,
    /// Human-readable name of the active output device.
    pub active_output_device_name: String,
    /// State of the host audio stream.
    pub stream_state: RuntimeHostAudioStreamState,
    /// Clock source in use.
    pub clock_source: RuntimeHostClockSource,
    /// Clock domain relationship between input and output.
    pub clock_domain: RuntimeHostClockDomain,
    /// Clock fallback state.
    pub fallback_state: RuntimeHostClockFallbackState,
    /// Clock transition state.
    pub transition_state: RuntimeHostClockTransitionState,
    /// Clock drift management state.
    pub drift_state: RuntimeHostClockDriftState,
    /// Clock discontinuity state.
    pub discontinuity_state: RuntimeHostClockDiscontinuityState,
    /// Duplex sample-rate mismatch state.
    pub duplex_mismatch_state: RuntimeHostDuplexMismatchState,
    /// Endpoint topology (output-only, duplex, aggregate, etc.).
    pub endpoint_topology: RuntimeHostEndpointTopology,
    /// Whether only a subset of the requested I/O channels is available.
    pub partial_availability: bool,
    /// Whether a clock fallback is currently active.
    pub fallback_active: bool,
    /// Whether the runtime's graph ID matches the audio pump's graph ID.
    pub runtime_graph_id_matches_pump: bool,
    /// Output latency measured by the backend in samples.
    pub output_latency_samples: u32,
    /// Estimated output latency when the measured value is unavailable.
    pub estimated_output_latency_samples: u32,
    /// Total number of xruns (underruns/overruns) since startup.
    pub xrun_count: u64,
    /// Total number of callback overruns since startup.
    pub callback_overrun_count: u64,
    /// Total number of device-loss events since startup.
    pub device_loss_count: u64,
    /// Total number of device restart attempts since startup.
    pub restart_attempt_count: u64,
    /// Total number of failed device restart attempts since startup.
    pub restart_failure_count: u64,
}
