use signal_hardware::BackendHealth;

use crate::{
    RuntimeHostAudioStreamState, RuntimeHostClockDomain, RuntimeHostClockFallbackState,
    RuntimeHostLifecycleOwnership, RuntimeHostRestartPolicy, RuntimeLinuxAudioBackendIdentity,
    RuntimeLinuxAudioBackendPortabilityBand,
};

/// Session role parity classification for a PipeWire or ALSA backend.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimePipeWireAlsaSessionRoleParity {
    /// Running on a backend other than PipeWire or ALSA.
    NotPipeWireOrAlsa,
    /// Session role parity is unavailable.
    Unavailable,
    /// Session is the primary audio I/O path.
    PrimaryAudioIo,
    /// Session is capable of monitoring but not full duplex I/O.
    MonitoringCapable,
    /// Session is unavailable for offline rendering.
    OfflineUnavailable,
    /// Session is serving as a fallback continuation path.
    FallbackContinuation,
}

/// Device claim parity classification for a PipeWire or ALSA backend.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimePipeWireAlsaDeviceClaimParity {
    /// Running on a backend other than PipeWire or ALSA.
    NotPipeWireOrAlsa,
    /// Device claim parity is unavailable.
    Unavailable,
    /// Device has not been claimed by any session.
    NoClaim,
    /// Session holds a direct exclusive claim on the device.
    DirectClaim,
    /// Session participates in a shared backend graph.
    SharedGraph,
    /// Device was lost and has not been reacquired.
    Lost,
    /// Device claim has been released.
    Released,
}

/// Callback stream policy parity classification for a PipeWire or ALSA backend.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimePipeWireAlsaStreamPolicyParity {
    /// Running on a backend other than PipeWire or ALSA.
    NotPipeWireOrAlsa,
    /// Stream policy parity is unavailable.
    Unavailable,
    /// Host drives the audio callback directly.
    DirectHostCallback,
    /// Backend manages the audio graph and drives the callback.
    BackendManagedGraph,
    /// Stream is in the process of restarting.
    Restarting,
}

/// Guarded parity state summarising which constraint is limiting direct PipeWire/ALSA ownership.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimePipeWireAlsaGuardedParityState {
    /// Running on a backend other than PipeWire or ALSA.
    NotPipeWireOrAlsa,
    /// Guarded parity state is unavailable.
    Unavailable,
    /// Session has direct ownership with no guarded constraint active.
    Direct,
    /// Backend manages the session; direct ownership is guarded.
    BackendManaged,
    /// A clock-domain constraint is limiting direct ownership.
    ClockGuarded,
    /// A transfer-policy constraint is limiting direct ownership.
    TransferGuarded,
    /// Session is in a recovery-guarded state.
    RecoveryGuarded,
}

/// Full parity snapshot for a PipeWire or ALSA session: role, device claim, stream policy, guarded state, clocking, and fault counters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePipeWireAlsaParitySnapshot {
    /// Linux audio backend identity (PipeWire or ALSA).
    pub backend_identity: RuntimeLinuxAudioBackendIdentity,
    /// Human-readable name of the active backend.
    pub backend_name: String,
    /// Portability band for this backend.
    pub portability_band: RuntimeLinuxAudioBackendPortabilityBand,
    /// Session role parity classification.
    pub session_role_parity: RuntimePipeWireAlsaSessionRoleParity,
    /// Device claim parity classification.
    pub device_claim_parity: RuntimePipeWireAlsaDeviceClaimParity,
    /// Callback stream policy parity classification.
    pub stream_policy_parity: RuntimePipeWireAlsaStreamPolicyParity,
    /// Guarded parity state summarising any active ownership constraints.
    pub guarded_state: RuntimePipeWireAlsaGuardedParityState,
    /// Who owns the audio callback lifecycle.
    pub lifecycle_ownership: RuntimeHostLifecycleOwnership,
    /// Who is responsible for backend restarts after a fault.
    pub restart_policy: RuntimeHostRestartPolicy,
    /// Clock domain relationship between the host and runtime.
    pub clock_domain: RuntimeHostClockDomain,
    /// Active clock fallback mode.
    pub fallback_state: RuntimeHostClockFallbackState,
    /// Stable identifier for the active audio device.
    pub device_id: String,
    /// Human-readable name of the active audio device.
    pub device_name: String,
    /// Current state of the host audio stream.
    pub stream_state: RuntimeHostAudioStreamState,
    /// Health state reported by the audio backend.
    pub backend_health: BackendHealth,
    /// Whether the backend is operating in simulated mode.
    pub simulated: bool,
    /// Number of times the audio device was lost and had to be reacquired.
    pub device_loss_count: u64,
    /// Number of backend restart attempts.
    pub restart_attempt_count: u64,
    /// Number of backend restart attempts that failed.
    pub restart_failure_count: u64,
    /// Human-readable summary of the parity snapshot.
    pub summary: String,
}
