//! Shared runtime control and message model for Signal.

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
