use signal_ipc::{PluginMessageEnvelope, PluginMessagePayload, SharedMemoryTransportPayload};
use signal_plugin_clap::{
    classify_sandbox_failure, sandbox_failure_event, ClapSandboxFailureStage,
};
use signal_runtime::{
    PluginFaultKind, RuntimeError, SandboxOperationFailureStage, SignalRuntime,
};

use super::super::FaultInjection;
use super::instance_state::plugin_instance_state_record;

pub(crate) fn lifecycle_stage_for_request(
    payload: &PluginMessagePayload,
) -> Option<signal_runtime::PluginSandboxLifecycleStage> {
    match payload {
        PluginMessagePayload::SandboxHandshakeRequest { .. } => {
            Some(signal_runtime::PluginSandboxLifecycleStage::SandboxHandshaken)
        }
        PluginMessagePayload::LoadPluginTypeRequest { .. } => {
            Some(signal_runtime::PluginSandboxLifecycleStage::PluginTypeLoaded)
        }
        PluginMessagePayload::CreateInstanceRequest { .. } => {
            Some(signal_runtime::PluginSandboxLifecycleStage::InstanceCreated)
        }
        PluginMessagePayload::PrepareInstanceRequest { .. } => {
            Some(signal_runtime::PluginSandboxLifecycleStage::InstancePrepared)
        }
        PluginMessagePayload::ActivateInstanceRequest { .. } => {
            Some(signal_runtime::PluginSandboxLifecycleStage::InstanceActivated)
        }
        PluginMessagePayload::DeactivateInstanceRequest { .. } => {
            Some(signal_runtime::PluginSandboxLifecycleStage::InstanceDeactivated)
        }
        PluginMessagePayload::ResetInstanceRequest { .. } => {
            Some(signal_runtime::PluginSandboxLifecycleStage::InstanceReset)
        }
        PluginMessagePayload::DestroyInstanceRequest { .. } => {
            Some(signal_runtime::PluginSandboxLifecycleStage::InstanceDestroyed)
        }
        _ => None,
    }
}

pub(crate) fn record_runtime_fault(runtime: &mut SignalRuntime, failure: &PluginMessageEnvelope) {
    if let PluginMessagePayload::SandboxFailure {
        sandbox_id,
        detail,
        processing_epoch,
        fault,
        instance_state,
        ..
    } = &failure.payload
    {
        let kind = runtime_plugin_fault_kind(Some(fault));
        runtime.record_plugin_sandbox_fault(
            sandbox_id.clone(),
            kind,
            detail.clone(),
            *processing_epoch,
        );
        if let Some(instance_state) = instance_state.as_ref() {
            runtime.record_plugin_sandbox_instance_state(plugin_instance_state_record(
                sandbox_id,
                *processing_epoch,
                instance_state,
            ));
        }
        if let Some(classification) = classify_sandbox_failure(failure) {
            runtime.record_sandbox_operation_failure(
                classification.sandbox_id,
                classification.lease_id,
                classification.processing_epoch,
                classification.operation,
                classification.error_kind,
                map_clap_sandbox_failure_stage(classification.stage),
                classification.detail,
            );
        }
    }
}

fn runtime_plugin_fault_kind(fault: Option<&signal_ipc::PluginFaultPayload>) -> PluginFaultKind {
    match fault.map(|fault| fault.kind.as_str()) {
        Some("timeout") => PluginFaultKind::Timeout,
        Some("crash") => PluginFaultKind::Crash,
        _ => PluginFaultKind::ProtocolViolation,
    }
}

fn map_clap_sandbox_failure_stage(stage: ClapSandboxFailureStage) -> SandboxOperationFailureStage {
    match stage {
        ClapSandboxFailureStage::PrepareAttach => SandboxOperationFailureStage::PrepareAttach,
        ClapSandboxFailureStage::ProcessAttach => SandboxOperationFailureStage::ProcessAttach,
        ClapSandboxFailureStage::ProcessFlush => SandboxOperationFailureStage::ProcessFlush,
        ClapSandboxFailureStage::ProcessProtocolViolation => {
            SandboxOperationFailureStage::ProcessProtocolViolation
        }
        ClapSandboxFailureStage::ControlProtocolViolation => {
            SandboxOperationFailureStage::ControlProtocolViolation
        }
    }
}

pub(crate) fn build_fault_envelope(
    sandbox_id: &str,
    instance_id: &str,
    lease_id: &str,
    processing_epoch: u64,
    fault: FaultInjection,
) -> PluginMessageEnvelope {
    let (error_kind, detail) = match fault {
        FaultInjection::Timeout => ("timeout", "sandbox exceeded block deadline"),
        FaultInjection::Crash => ("crash", "sandbox process exited unexpectedly"),
        FaultInjection::HeartbeatMiss
        | FaultInjection::DeviceLoss
        | FaultInjection::DeviceLossRestartFailure
        | FaultInjection::RecoveryDeferredTeardownFailure
        | FaultInjection::RecoveryDeferredTeardownThenCleanup
        | FaultInjection::RecoveryDeferredTeardownCleanupRetry
        | FaultInjection::RecoveryTeardownFailure
        | FaultInjection::RecoveryRestartFailure
        | FaultInjection::RecoveryOverlapContention
        | FaultInjection::RecoveryInterleavedFailures
        | FaultInjection::EscalatingHeartbeatMisses {
            restart_episodes: _,
        }
        | FaultInjection::MixedWatchdogEpisodes {
            restart_episodes: _,
        } => ("timeout", "sandbox heartbeat watchdog threshold exceeded"),
    };
    sandbox_failure_event(
        sandbox_id,
        Some(instance_id.into()),
        "processBlock",
        error_kind,
        detail,
        Some(processing_epoch),
        Some(lease_id.into()),
        None,
    )
}

pub(crate) fn extract_prepare_metadata(
    responses: &[PluginMessageEnvelope],
) -> (String, Option<SharedMemoryTransportPayload>) {
    responses
        .iter()
        .find_map(|response| match &response.payload {
            PluginMessagePayload::PrepareInstanceResponse {
                shared_memory_lease_id,
                shared_memory_transport,
                ..
            } => Some((
                shared_memory_lease_id.clone(),
                Some(shared_memory_transport.clone()),
            )),
            _ => None,
        })
        .unwrap_or_default()
}

pub(crate) fn runtime_error_from_failure(failure: &PluginMessageEnvelope) -> RuntimeError {
    match &failure.payload {
        PluginMessagePayload::SandboxFailure { detail, fault, .. } => RuntimeError {
            kind: match Some(fault).map(|fault| fault.kind.as_str()) {
                Some("timeout") => signal_runtime::RuntimeErrorKind::Timeout,
                Some("crash") | Some("fatal") => signal_runtime::RuntimeErrorKind::Fatal,
                _ => signal_runtime::RuntimeErrorKind::PluginFailure,
            },
            message: detail.clone(),
        },
        _ => RuntimeError {
            kind: signal_runtime::RuntimeErrorKind::PluginFailure,
            message: "plugin sandbox lifecycle failed".into(),
        },
    }
}
