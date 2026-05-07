//! CLAP plugin format adapter for Signal. Implements the CLAP host protocol and bridges CLAP plugin instances to the format-neutral `signal-plugin` abstractions. macOS and Linux only.

#![warn(missing_docs)]

mod adapter;
mod clap_sandbox_harness;
mod discovery;
mod event_translation;
mod events;
mod protocol;
mod transport;

pub use adapter::{
    BrokeredBlockOutcome, ClapDiscoveredPluginType, ClapHostExtension, ClapInstanceControlSurface,
    ClapPluginHostAdapter, ClapPreparePlan, ClapSharedMemoryHeader,
};
pub use clap_sandbox_harness::{
    classify_sandbox_failure, sandbox_failure_event, ClapHarnessError, ClapHarnessResult,
    ClapSandboxFailureClassification, ClapSandboxFailureInput, ClapSandboxFailureStage,
    ClapSandboxLifecycleHarness,
};
pub use event_translation::{
    io_layout_from_payload, io_layout_payload, shared_memory_layout, shared_memory_layout_payload,
    translate_input_events, translate_output_events,
};
pub use events::{
    ClapEvent, ClapEventPacket, ClapMidiEvent, ClapNoteEvent, ClapNoteEventKind,
    ClapNoteExpressionEvent, ClapNoteExpressionKind, ClapParamGestureEvent, ClapParamGesturePhase,
    ClapParamModEvent, ClapParamValueEvent,
};
pub use protocol::ClapBlockProtocol;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
