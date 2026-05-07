//! Shared IPC seam for runtime control and plugin messaging.
//!
//! This crate defines the message model and shared-memory broker used across the
//! runtime/plugin boundary. It is dependency-free with respect to the runtime
//! engine itself, so both sides of the process boundary can depend on it without
//! creating a circular dependency.
//!
//! The two main building blocks are:
//!
//! - **Message model** — [`RuntimeMessage`] and its associated types model
//!   typed, domain-scoped commands, responses, and events. [`PluginMessageEnvelope`]
//!   wraps the full plugin sandbox protocol on top of this model.
//! - **Shared-memory broker** — [`SharedMemoryBroker`] creates, attaches to, and
//!   destroys memory-mapped file regions used to pass audio buffers between the
//!   host and plugin sandboxes.

#![warn(missing_docs)]

mod plugin_protocol;
mod runtime_message;
mod shared_memory;

#[cfg(test)]
mod tests;

pub use plugin_protocol::{
    PluginDescriptorPayload, PluginFaultPayload, PluginInstanceStatePayload, PluginIoLayoutPayload,
    PluginMessageEnvelope, PluginMessageName, PluginMessagePayload,
    PluginProcessConfigurationPayload, SharedMemoryLayoutPayload, SharedMemoryRegionPayload,
    SharedMemoryTransportKind, SharedMemoryTransportPayload,
};
pub use runtime_message::{CorrelationId, MessageKind, RuntimeDomain, RuntimeMessage};
pub use shared_memory::{
    MappedSharedMemoryRegion, SharedMemoryBroker, SharedMemoryRegionLifecycleError,
    SharedMemoryRegionLifecycleErrorKind, SharedMemoryRegionOperation,
};
