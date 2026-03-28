//! Format-neutral plugin host abstractions for Signal.

mod blocks;
mod event_codec;
mod events;
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
pub use render_context_codec::{read_render_context_from_slice, write_render_context_to_slice};

pub use plugin_block_transport::{
    BlockDispatch, BlockProcessResult, BlockProcessingHeader, CompletionSlot, CompletionState,
    SandboxStateMachine, SharedMemoryLayout, SharedMemoryLease, SharedMemoryRegion,
};
pub use plugin_event_reports::{
    AutomationContinuityReport, AutomationContinuitySegment, BlockSequenceContinuityReport,
    BlockSequenceContinuitySegment, EventPacketContinuityReport, EventPacketContinuitySegment,
    EventPacketSummary, ParameterAutomationSummary,
};
pub use plugin_model::{
    PluginAudioBusDescriptor, PluginAudioBusDirection, PluginDegradedReason, PluginDescriptor,
    PluginFault, PluginFaultKind, PluginFaultSeverity, PluginFeature, PluginFormat,
    PluginInstanceId, PluginInstanceSnapshot, PluginIoLayout, PluginLifecycleContract,
    PluginLifecycleState, PluginParameterDescriptor, PluginParameterDomain, PluginParameterFlags,
    PluginProcessConfiguration, PluginProcessingContract, PluginReadiness, PluginStateContract,
    PluginTypeId, SandboxPolicy,
};
pub use sandbox_protocol::{
    LoopRange, PluginRenderContext, PluginSandboxCapabilities, PluginSandboxError,
    PluginSandboxErrorKind, PluginSandboxRequest, RestartEscalationPolicy, RestartEscalationState,
    SandboxControlCommand, SandboxControlRequest, SandboxControlResponse, SandboxTransport,
    SandboxWatchdogPolicy, SandboxWatchdogState, WatchdogOutcome, WatchdogTriggerReason,
};

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
