use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalIoHealthState {
    Unavailable,
    Ready,
    FallbackActive,
    Recovering,
    Faulted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalIoDeviceChangeState {
    Unavailable,
    Stable,
    PendingRestart,
    Recovering,
    Failed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalIoPrimaryRole {
    Unavailable,
    ProgramOutput,
    ExternalInput,
    ProgramDuplex,
    AggregateProgram,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalIoMonitoringState {
    Unavailable,
    Direct,
    Guarded,
    Degraded,
    Faulted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalIoMonitoringTapPoint {
    Unavailable,
    PreGraphInput,
    PostRuntimeOutput,
    PostHardwareOutput,
    PostLoopbackReturn,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalIoLoopbackState {
    Unavailable,
    Ready,
    Guarded,
    Recovering,
    Faulted,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeExternalIoSnapshot {
    pub health_state: RuntimeExternalIoHealthState,
    pub device_change_state: RuntimeExternalIoDeviceChangeState,
    pub primary_role: RuntimeExternalIoPrimaryRole,
    pub monitoring_state: RuntimeExternalIoMonitoringState,
    pub monitoring_tap_point: RuntimeExternalIoMonitoringTapPoint,
    pub loopback_state: RuntimeExternalIoLoopbackState,
    pub io_layout: RuntimeMultichannelIoSummary,
    pub linux_backend_identity: RuntimeLinuxAudioBackendIdentity,
    pub linux_backend_portability: RuntimeLinuxAudioBackendPortabilityBand,
    pub backend_name: String,
    pub active_output_device_id: String,
    pub active_output_device_name: String,
    pub stream_state: RuntimeHostAudioStreamState,
    pub clock_source: RuntimeHostClockSource,
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
    pub fallback_active: bool,
    pub runtime_graph_id_matches_pump: bool,
    pub output_latency_samples: u32,
    pub estimated_output_latency_samples: u32,
    pub xrun_count: u64,
    pub callback_overrun_count: u64,
    pub device_loss_count: u64,
    pub restart_attempt_count: u64,
    pub restart_failure_count: u64,
    pub summary: String,
}
