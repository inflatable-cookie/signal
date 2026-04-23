use signal_ipc::{CorrelationId, PluginInstanceStatePayload};

mod classification;
mod payloads;

pub(crate) use classification::failure_event;
pub use classification::{
    classify_sandbox_failure, ClapSandboxFailureClassification, ClapSandboxFailureStage,
};
pub(crate) use payloads::{lifecycle_state_string, plugin_fault_payload};

/// Input data describing a CLAP sandbox failure, used to classify and report the fault.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClapSandboxFailureInput {
    /// Identifier for the sandbox that faulted.
    pub sandbox_id: String,
    /// Identifier for the plugin instance involved, if known.
    pub instance_id: Option<String>,
    /// The lifecycle stage at which the failure occurred.
    pub stage: String,
    /// Machine-readable error classification string.
    pub error_kind: String,
    /// Human-readable detail about the failure.
    pub detail: String,
    /// The processing epoch in effect when the failure occurred, if applicable.
    pub processing_epoch: Option<u64>,
    /// The shared-memory lease ID in use when the failure occurred, if applicable.
    pub shared_memory_lease_id: Option<String>,
    /// Correlation ID from the request that triggered the failure, if applicable.
    pub correlation_id: Option<CorrelationId>,
    /// Snapshot of the instance lifecycle state at the time of failure, if available.
    pub instance_state: Option<PluginInstanceStatePayload>,
}

/// Converts a [`ClapSandboxFailureInput`] into a [`signal_ipc::PluginMessageEnvelope`] failure event.
pub fn sandbox_failure_event(input: ClapSandboxFailureInput) -> signal_ipc::PluginMessageEnvelope {
    failure_event(input)
}
