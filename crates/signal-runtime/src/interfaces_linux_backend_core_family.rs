#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeLinuxAudioBackendIdentity {
    NotLinux,
    Unavailable,
    Alsa,
    Jack,
    PipeWire,
    Unsupported,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeLinuxAudioBackendPortabilityBand {
    Portable,
    Guarded,
    Unsupported,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeLinuxAudioBackendClockingParityBand {
    Portable,
    Guarded,
    Unsupported,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeLinuxAudioBackendDuplexParityState {
    Aligned,
    Guarded,
    Partial,
    Unsupported,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeLinuxAudioBackendEndpointTopologyParityState {
    Portable,
    Guarded,
    Partial,
    Unsupported,
}

#[path = "interfaces_linux_backend_core_family/linux_session.rs"]
mod linux_session;
#[path = "interfaces_linux_backend_core_family/pipewire_alsa.rs"]
mod pipewire_alsa;

pub use linux_session::*;
pub use pipewire_alsa::*;
