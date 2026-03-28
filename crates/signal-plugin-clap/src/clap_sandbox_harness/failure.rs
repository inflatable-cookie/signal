use signal_ipc::{CorrelationId, PluginInstanceStatePayload};

mod classification;
mod payloads;

pub(crate) use classification::failure_event;
pub use classification::{
    classify_sandbox_failure, ClapSandboxFailureClassification, ClapSandboxFailureStage,
};
pub(crate) use payloads::{lifecycle_state_string, plugin_fault_payload};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClapSandboxFailureInput {
    pub sandbox_id: String,
    pub instance_id: Option<String>,
    pub stage: String,
    pub error_kind: String,
    pub detail: String,
    pub processing_epoch: Option<u64>,
    pub shared_memory_lease_id: Option<String>,
    pub correlation_id: Option<CorrelationId>,
    pub instance_state: Option<PluginInstanceStatePayload>,
}

pub fn sandbox_failure_event(input: ClapSandboxFailureInput) -> signal_ipc::PluginMessageEnvelope {
    failure_event(input)
}
