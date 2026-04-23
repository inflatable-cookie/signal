use signal_ipc::{PluginMessageEnvelope, PluginMessageName, PluginMessagePayload};

use super::payloads::{plugin_fault_for_error_kind, plugin_fault_payload};
use super::ClapSandboxFailureInput;

/// The stage in the CLAP sandbox lifecycle at which a failure occurred.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClapSandboxFailureStage {
    /// Failure attaching the shared-memory region during `prepareInstance`.
    PrepareAttach,
    /// Failure attaching the shared-memory region during `processBlock`.
    ProcessAttach,
    /// Failure flushing the shared-memory region during `processBlock`.
    ProcessFlush,
    /// Protocol violation during `processBlock`.
    ProcessProtocolViolation,
    /// Protocol violation during a lifecycle control operation.
    ControlProtocolViolation,
}

/// Structured classification of a CLAP sandbox failure, decoded from a [`signal_ipc::PluginMessageEnvelope`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClapSandboxFailureClassification {
    /// Identifier of the sandbox that faulted.
    pub sandbox_id: String,
    /// Shared-memory lease ID in use at the time of failure, if any.
    pub lease_id: Option<String>,
    /// Processing epoch in use at the time of failure, if applicable.
    pub processing_epoch: Option<u64>,
    /// The lifecycle operation that was in progress when the failure occurred.
    pub operation: String,
    /// Machine-readable error kind string.
    pub error_kind: String,
    /// Classified failure stage.
    pub stage: ClapSandboxFailureStage,
    /// Human-readable detail about the failure.
    pub detail: String,
}

/// Classifies a [`signal_ipc::PluginMessageEnvelope`] as a structured [`ClapSandboxFailureClassification`], or returns `None` if the envelope is not a recognisable sandbox failure.
pub fn classify_sandbox_failure(
    envelope: &PluginMessageEnvelope,
) -> Option<ClapSandboxFailureClassification> {
    let PluginMessagePayload::SandboxFailure {
        sandbox_id,
        stage,
        error_kind,
        detail,
        processing_epoch,
        shared_memory_lease_id,
        ..
    } = &envelope.payload
    else {
        return None;
    };

    let stage = match (stage.as_str(), error_kind.as_str()) {
        ("prepareInstance", "resourceUnavailable")
            if detail.contains("attach shared-memory region") =>
        {
            ClapSandboxFailureStage::PrepareAttach
        }
        ("processBlock", "resourceUnavailable")
            if detail.contains("attach shared-memory region") =>
        {
            ClapSandboxFailureStage::ProcessAttach
        }
        ("processBlock", "resourceUnavailable")
            if detail.contains("flush shared-memory region") =>
        {
            ClapSandboxFailureStage::ProcessFlush
        }
        ("processBlock", "protocolViolation") => ClapSandboxFailureStage::ProcessProtocolViolation,
        (_, "protocolViolation") => ClapSandboxFailureStage::ControlProtocolViolation,
        _ => return None,
    };

    Some(ClapSandboxFailureClassification {
        sandbox_id: sandbox_id.clone(),
        lease_id: shared_memory_lease_id.clone(),
        processing_epoch: *processing_epoch,
        operation: stage_string(stage, &envelope.payload),
        error_kind: error_kind.clone(),
        stage,
        detail: detail.clone(),
    })
}

fn stage_string(stage: ClapSandboxFailureStage, payload: &PluginMessagePayload) -> String {
    match payload {
        PluginMessagePayload::SandboxFailure { stage, .. } => stage.clone(),
        _ => format!("{stage:?}"),
    }
}

pub(crate) fn failure_event(input: ClapSandboxFailureInput) -> PluginMessageEnvelope {
    let ClapSandboxFailureInput {
        sandbox_id,
        instance_id,
        stage,
        error_kind,
        detail,
        processing_epoch,
        shared_memory_lease_id,
        correlation_id,
        instance_state,
    } = input;
    let fault = plugin_fault_for_error_kind(&error_kind, &detail);
    PluginMessageEnvelope::event(
        PluginMessageName::SandboxFailure,
        correlation_id,
        PluginMessagePayload::SandboxFailure {
            sandbox_id,
            instance_id,
            stage,
            error_kind,
            detail,
            fault: plugin_fault_payload(&fault),
            instance_state,
            processing_epoch,
            shared_memory_lease_id,
        },
    )
}
