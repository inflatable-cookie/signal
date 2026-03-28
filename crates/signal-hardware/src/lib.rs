//! Common hardware and device abstractions for Signal.

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
