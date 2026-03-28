use signal_ipc::{PluginMessageEnvelope, PluginMessageName, PluginMessagePayload};

use super::payloads::{plugin_fault_for_error_kind, plugin_fault_payload};
use super::ClapSandboxFailureInput;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClapSandboxFailureStage {
    PrepareAttach,
    ProcessAttach,
    ProcessFlush,
    ProcessProtocolViolation,
    ControlProtocolViolation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClapSandboxFailureClassification {
    pub sandbox_id: String,
    pub lease_id: Option<String>,
    pub processing_epoch: Option<u64>,
    pub operation: String,
    pub error_kind: String,
    pub stage: ClapSandboxFailureStage,
    pub detail: String,
}

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
