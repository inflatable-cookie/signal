use signal_hardware::BackendHealth;

use crate::{
    RuntimeHostAudioStreamState, RuntimeHostClockDomain, RuntimeHostClockFallbackState,
    RuntimeHostLifecycleOwnership, RuntimeHostRestartPolicy, RuntimeLinuxAudioBackendIdentity,
    RuntimeLinuxAudioBackendPortabilityBand,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimePipeWireAlsaSessionRoleParity {
    NotPipeWireOrAlsa,
    Unavailable,
    PrimaryAudioIo,
    MonitoringCapable,
    OfflineUnavailable,
    FallbackContinuation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimePipeWireAlsaDeviceClaimParity {
    NotPipeWireOrAlsa,
    Unavailable,
    NoClaim,
    DirectClaim,
    SharedGraph,
    Lost,
    Released,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimePipeWireAlsaStreamPolicyParity {
    NotPipeWireOrAlsa,
    Unavailable,
    DirectHostCallback,
    BackendManagedGraph,
    Restarting,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimePipeWireAlsaGuardedParityState {
    NotPipeWireOrAlsa,
    Unavailable,
    Direct,
    BackendManaged,
    ClockGuarded,
    TransferGuarded,
    RecoveryGuarded,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePipeWireAlsaParitySnapshot {
    pub backend_identity: RuntimeLinuxAudioBackendIdentity,
    pub backend_name: String,
    pub portability_band: RuntimeLinuxAudioBackendPortabilityBand,
    pub session_role_parity: RuntimePipeWireAlsaSessionRoleParity,
    pub device_claim_parity: RuntimePipeWireAlsaDeviceClaimParity,
    pub stream_policy_parity: RuntimePipeWireAlsaStreamPolicyParity,
    pub guarded_state: RuntimePipeWireAlsaGuardedParityState,
    pub lifecycle_ownership: RuntimeHostLifecycleOwnership,
    pub restart_policy: RuntimeHostRestartPolicy,
    pub clock_domain: RuntimeHostClockDomain,
    pub fallback_state: RuntimeHostClockFallbackState,
    pub device_id: String,
    pub device_name: String,
    pub stream_state: RuntimeHostAudioStreamState,
    pub backend_health: BackendHealth,
    pub simulated: bool,
    pub device_loss_count: u64,
    pub restart_attempt_count: u64,
    pub restart_failure_count: u64,
    pub summary: String,
}
