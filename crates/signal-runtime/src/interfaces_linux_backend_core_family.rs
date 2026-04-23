/// Identifies which Linux audio backend (ALSA, JACK, PipeWire) is active,
/// or `NotLinux` on non-Linux platforms.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeLinuxAudioBackendIdentity {
    /// Running on a non-Linux platform.
    NotLinux,
    /// Linux backend identity is unavailable.
    Unavailable,
    /// ALSA (Advanced Linux Sound Architecture) backend.
    Alsa,
    /// JACK Audio Connection Kit backend.
    Jack,
    /// PipeWire backend.
    PipeWire,
    /// Linux backend identity is unsupported or unrecognised.
    Unsupported,
}

/// Cross-platform portability band for the Linux audio backend.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeLinuxAudioBackendPortabilityBand {
    /// Backend is portable across platforms without known constraints.
    Portable,
    /// Backend portability is guarded by a platform-specific constraint.
    Guarded,
    /// Backend portability classification is not applicable or unsupported.
    Unsupported,
}

/// Clocking parity band for the Linux audio backend.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeLinuxAudioBackendClockingParityBand {
    /// Clocking is portable across platforms without known constraints.
    Portable,
    /// Clocking portability is guarded by a platform-specific constraint.
    Guarded,
    /// Clocking parity classification is not applicable or unsupported.
    Unsupported,
}

/// Duplex (input+output) alignment parity of the Linux audio backend.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeLinuxAudioBackendDuplexParityState {
    /// Input and output endpoints are on the same clock and fully aligned.
    Aligned,
    /// Duplex alignment is guarded by a platform-specific constraint.
    Guarded,
    /// Only partial duplex availability is present (input-only or output-only).
    Partial,
    /// Duplex parity classification is not applicable or unsupported.
    Unsupported,
}

/// Endpoint topology parity of the Linux audio backend.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeLinuxAudioBackendEndpointTopologyParityState {
    /// Endpoint topology is portable across platforms.
    Portable,
    /// Endpoint topology portability is guarded by a platform-specific constraint.
    Guarded,
    /// Only partial endpoint topology availability is present.
    Partial,
    /// Endpoint topology parity classification is not applicable or unsupported.
    Unsupported,
}

#[path = "interfaces_linux_backend_core_family/linux_session.rs"]
mod linux_session;
#[path = "interfaces_linux_backend_core_family/pipewire_alsa.rs"]
mod pipewire_alsa;

pub use linux_session::*;
pub use pipewire_alsa::*;
