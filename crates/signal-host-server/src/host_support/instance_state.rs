use signal_ipc::{PluginInstanceStatePayload, PluginMessageEnvelope, PluginMessagePayload};
use signal_runtime::{PluginSandboxInstanceFaultRecord, PluginSandboxInstanceStateRecord};

pub(crate) fn plugin_instance_state_record(
    sandbox_id: &str,
    processing_epoch: Option<u64>,
    state: &PluginInstanceStatePayload,
) -> PluginSandboxInstanceStateRecord {
    let processing = state.processing.as_ref();
    PluginSandboxInstanceStateRecord {
        sandbox_id: sandbox_id.to_string(),
        plugin_type_id: state.plugin_type_id.clone(),
        instance_id: state.instance_id.clone(),
        lifecycle_state: state.lifecycle_state.clone(),
        readiness_state: state.readiness_state.clone(),
        degraded_reasons: state.degraded_reasons.clone(),
        active: state.active,
        processing_epoch,
        processing_sample_rate_hz: processing.map(|processing| processing.sample_rate_hz),
        processing_max_block_frames: processing.map(|processing| processing.max_block_frames),
        audio_inputs: processing.map(|processing| processing.io_layout.audio_inputs),
        audio_outputs: processing.map(|processing| processing.io_layout.audio_outputs),
        midi_inputs: processing.map(|processing| processing.io_layout.midi_inputs),
        midi_outputs: processing.map(|processing| processing.io_layout.midi_outputs),
        last_fault: state
            .last_fault
            .as_ref()
            .map(|fault| PluginSandboxInstanceFaultRecord {
                kind: fault.kind.clone(),
                severity: fault.severity.clone(),
                message: fault.message.clone(),
            }),
    }
}

pub(crate) fn plugin_instance_state_record_from_response(
    sandbox_id: &str,
    processing_epoch: Option<u64>,
    response: &PluginMessageEnvelope,
) -> Option<PluginSandboxInstanceStateRecord> {
    match &response.payload {
        PluginMessagePayload::CreateInstanceResponse { instance_state, .. }
        | PluginMessagePayload::PrepareInstanceResponse { instance_state, .. }
        | PluginMessagePayload::ActivateInstanceResponse { instance_state, .. }
        | PluginMessagePayload::DeactivateInstanceResponse { instance_state, .. }
        | PluginMessagePayload::ResetInstanceResponse { instance_state, .. }
        | PluginMessagePayload::DestroyInstanceResponse { instance_state, .. } => Some(
            plugin_instance_state_record(sandbox_id, processing_epoch, instance_state),
        ),
        PluginMessagePayload::HeartbeatResponse {
            instance_state: Some(instance_state),
            ..
        } => Some(plugin_instance_state_record(
            sandbox_id,
            processing_epoch,
            instance_state,
        )),
        PluginMessagePayload::SandboxFailure {
            instance_state: Some(instance_state),
            ..
        } => Some(plugin_instance_state_record(
            sandbox_id,
            processing_epoch,
            instance_state,
        )),
        _ => None,
    }
}
