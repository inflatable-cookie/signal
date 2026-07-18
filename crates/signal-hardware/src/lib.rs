//! Common hardware and device abstractions for Signal.
//!
//! This crate defines the [`HardwareBackend`] trait — the core abstraction every
//! audio hardware implementation must satisfy. A backend advertises available
//! devices, negotiates a concrete stream configuration from a caller-supplied
//! request, and reports ongoing diagnostic health.
//!
//! Platform-specific realizations (e.g. CoreAudio) live in separate crates such
//! as the host-local hardware backend and depend on this crate for the shared types.
//!
//! For testing and CI, use [`SimulatedHardwareBackend`], which implements
//! [`HardwareBackend`] without requiring real audio hardware. It accepts an
//! explicit device list at construction and validates stream negotiations against
//! that list.

#![warn(missing_docs)]

mod backend_contract;
pub mod capture;
mod diagnostics;
pub mod fake_clocked;
pub mod fake_input;
mod input_stream;
pub mod midi_input;
mod output_stream;

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
pub use capture::{
    CaptureActivationGate, CaptureReport, CaptureSession, MonitorSession, MonitorSink, SpscRing,
};
pub use diagnostics::{
    BackendHealth, HardwareDiagnosticEvent, HardwareDiagnosticKind, HardwareDiagnosticSeverity,
    HardwareDiagnosticsSnapshot,
};
pub use fake_clocked::FakeClockedBackend;
pub use fake_input::FakeInputBackend;
pub use input_stream::{
    InputCaptureFn, InputStreamBackend, InputStreamError, InputStreamHandle, InputStreamSpec,
    InputStreamState,
};
pub use midi_input::{
    FakeMidiInputBackend, MidiEventRing, MidiInputBackend, MidiInputError, MidiInputErrorKind,
    MidiInputEvent, MidiPortDescription, MidiSubscription, MidiSubscriptionState,
};
pub use output_stream::{
    OutputRenderFn, OutputStreamBackend, OutputStreamError, OutputStreamHandle, OutputStreamSpec,
    OutputStreamState,
};

// Re-export simulated backend at crate root for backward compatibility.
pub use simulated::SimulatedHardwareBackend;
