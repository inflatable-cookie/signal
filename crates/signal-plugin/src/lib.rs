//! Format-neutral plugin host abstractions for Signal.
//!
//! This crate defines the contracts, data types, and transport primitives that the Signal runtime
//! uses to communicate with plugins. All types are independent of the underlying plugin format:
//! CLAP, VST3, AU, and LV2 adapters all map their native concepts onto these shared abstractions.
//!
//! Use this crate when writing code that must work across formats — host-side scheduling,
//! sandboxing, event routing, and block dispatch. For format-specific loading or bridging,
//! use the corresponding adapter crate (`signal-plugin-clap`, `signal-plugin-vst3`, etc.).
//!
//! # Example
//!
//! ```no_run
//! use signal_plugin::{PluginDescriptor, PluginFormat, PluginTypeId, SandboxPolicy};
//!
//! fn accepts_strict_sandbox(descriptor: &PluginDescriptor) -> bool {
//!     descriptor.format == PluginFormat::Clap
//! }
//! ```

#![warn(missing_docs)]

mod blocks;
mod event_codec;
mod events;
mod param_changes;
mod plugin_block_transport;
mod plugin_event_reports;
mod plugin_model;
mod render_context_codec;
mod sandbox_protocol;

pub use blocks::{AudioBlock, BlockPayload, EventPacket};
pub use event_codec::{read_event_from_slice, write_event_to_slice};
pub use events::{
    MidiEvent, NoteEvent, NoteEventKind, NoteExpressionEvent, NoteExpressionKind,
    ParameterGestureEvent, ParameterGesturePhase, ParameterModulationEvent, ParameterValueEvent,
    PluginEvent,
};
pub use param_changes::{PluginParamChange, PluginParamChangeQueue, PLUGIN_PARAM_CHANGE_CAPACITY};
pub use render_context_codec::{read_render_context_from_slice, write_render_context_to_slice};

pub use plugin_block_transport::{
    BlockDispatch, BlockProcessResult, BlockProcessingHeader, CompletionSlot, CompletionState,
    SandboxStateMachine, SharedMemoryLayout, SharedMemoryLease, SharedMemoryRegion,
};
pub use plugin_event_reports::{EventPacketSummary, ParameterAutomationSummary};
pub use plugin_model::{
    PluginAudioBusDescriptor, PluginAudioBusDirection, PluginDegradedReason, PluginDescriptor,
    PluginFault, PluginFaultKind, PluginFaultSeverity, PluginFeature, PluginFormat,
    PluginInstanceId, PluginInstanceSnapshot, PluginIoLayout, PluginIsolationTier,
    PluginLifecycleContract, PluginLifecycleState, PluginParameterDescriptor,
    PluginParameterDomain, PluginParameterFlags, PluginProcessConfiguration,
    PluginProcessingContract, PluginReadiness, PluginStateContract, PluginTypeId, SandboxPolicy,
};
pub use sandbox_protocol::{
    LoopRange, PluginRenderContext, PluginSandboxCapabilities, PluginSandboxError,
    PluginSandboxErrorKind, PluginSandboxRequest, RestartEscalationPolicy, RestartEscalationState,
    SandboxTransport, SandboxWatchdogPolicy, SandboxWatchdogState, WatchdogOutcome,
    WatchdogTriggerReason,
};

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
