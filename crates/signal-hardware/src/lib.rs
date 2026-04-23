//! Common hardware and device abstractions for Signal.
//!
//! This crate defines the [`HardwareBackend`] trait — the core abstraction every
//! audio hardware implementation must satisfy. A backend advertises available
//! devices, negotiates a concrete stream configuration from a caller-supplied
//! request, and reports ongoing diagnostic health.
//!
//! Platform-specific realizations (e.g. CoreAudio) live in separate crates such
//! as `signal-hardware-coreaudio` and depend on this crate for the shared types.
//!
//! For testing and CI, use [`SimulatedHardwareBackend`], which implements
//! [`HardwareBackend`] without requiring real audio hardware. It accepts an
//! explicit device list at construction and validates stream negotiations against
//! that list.

#![warn(missing_docs)]


mod backend_contract;
mod diagnostics;

pub mod simulated;

pub use signal_primitives::SampleRate;

pub use backend_contract::{
    AudioDeviceDescriptor, AudioSampleFormat, AudioStreamDirection, BackendPolicyRecord,
    BackendPolicyTier, HardwareBackend, HardwareBackendIdentity, HardwareClockSource,
    HardwareClockTopology, HardwareConfigRequest, HardwareLatencyProfile,
    HardwareLifecycleContract, HardwareLifecycleOwnership, HardwareNegotiationError,
    HardwareNegotiationErrorKind, HardwareRestartPolicy, HardwareStreamConfig,
    HardwareStreamRequest, LinuxAudioBackendKind,
};
pub use diagnostics::{
    BackendHealth, HardwareDiagnosticEvent, HardwareDiagnosticKind, HardwareDiagnosticSeverity,
    HardwareDiagnosticsSnapshot,
};

// Re-export simulated backend at crate root for backward compatibility.
pub use simulated::SimulatedHardwareBackend;
