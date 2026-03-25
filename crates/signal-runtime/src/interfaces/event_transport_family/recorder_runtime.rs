use super::*;

pub(crate) fn json_recovery_record(record: Option<&RecoveryRecord>) -> String {
    match record {
        Some(record) => format!(
            concat!(
                "{{",
                "\"sandbox_id\":{},",
                "\"intent\":{},",
                "\"stop_reason\":{},",
                "\"processing_epoch\":{}",
                "}}"
            ),
            json_escape_string(&record.sandbox_id),
            json_escape_string(&format!("{:?}", record.intent)),
            json_escape_string(&format!("{:?}", record.stop_reason)),
            json_option_u64(record.processing_epoch),
        ),
        None => "null".into(),
    }
}

fn json_plugin_instance_fault_record(record: Option<&PluginSandboxInstanceFaultRecord>) -> String {
    match record {
        Some(record) => format!(
            concat!(
                "{{",
                "\"kind\":{},",
                "\"severity\":{},",
                "\"message\":{}",
                "}}"
            ),
            json_escape_string(record.kind.as_str()),
            json_escape_string(record.severity.as_str()),
            json_escape_string(record.message.as_str()),
        ),
        None => "null".into(),
    }
}

pub(crate) fn json_plugin_instance_state_record(
    record: Option<&PluginSandboxInstanceStateRecord>,
) -> String {
    match record {
        Some(record) => format!(
            concat!(
                "{{",
                "\"sandbox_id\":{},",
                "\"plugin_type_id\":{},",
                "\"instance_id\":{},",
                "\"lifecycle_state\":{},",
                "\"readiness_state\":{},",
                "\"degraded_reasons\":{},",
                "\"active\":{},",
                "\"processing_epoch\":{},",
                "\"processing_sample_rate_hz\":{},",
                "\"processing_max_block_frames\":{},",
                "\"audio_inputs\":{},",
                "\"audio_outputs\":{},",
                "\"midi_inputs\":{},",
                "\"midi_outputs\":{},",
                "\"last_fault\":{}",
                "}}"
            ),
            json_escape_string(record.sandbox_id.as_str()),
            json_escape_string(record.plugin_type_id.as_str()),
            json_escape_string(record.instance_id.as_str()),
            json_escape_string(record.lifecycle_state.as_str()),
            json_escape_string(record.readiness_state.as_str()),
            format!(
                "[{}]",
                record
                    .degraded_reasons
                    .iter()
                    .map(|reason| json_escape_string(reason.as_str()))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            record.active,
            json_option_u64(record.processing_epoch),
            json_option_u64(record.processing_sample_rate_hz.map(u64::from)),
            json_option_u64(record.processing_max_block_frames.map(u64::from)),
            json_option_u64(record.audio_inputs.map(u64::from)),
            json_option_u64(record.audio_outputs.map(u64::from)),
            json_option_u64(record.midi_inputs.map(u64::from)),
            json_option_u64(record.midi_outputs.map(u64::from)),
            json_plugin_instance_fault_record(record.last_fault.as_ref()),
        ),
        None => "null".into(),
    }
}

pub(crate) fn json_plugin_instance_state_record_vec(
    records: &[PluginSandboxInstanceStateRecord],
) -> String {
    format!(
        "[{}]",
        records
            .iter()
            .map(|record| json_plugin_instance_state_record(Some(record)))
            .collect::<Vec<_>>()
            .join(",")
    )
}

pub(crate) fn json_recovery_record_vec(records: &[RecoveryRecord]) -> String {
    let joined = records
        .iter()
        .map(|record| json_recovery_record(Some(record)))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

pub(crate) fn json_lifecycle_record(record: Option<&PluginSandboxLifecycleRecord>) -> String {
    match record {
        Some(record) => format!(
            concat!(
                "{{",
                "\"sandbox_id\":{},",
                "\"stage\":{},",
                "\"processing_epoch\":{}",
                "}}"
            ),
            json_escape_string(&record.sandbox_id),
            json_escape_string(&format!("{:?}", record.stage)),
            json_option_u64(record.processing_epoch),
        ),
        None => "null".into(),
    }
}

pub(crate) fn json_lifecycle_record_vec(records: &[PluginSandboxLifecycleRecord]) -> String {
    let joined = records
        .iter()
        .map(|record| json_lifecycle_record(Some(record)))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

pub(crate) fn json_transport_record(record: Option<&PluginSandboxTransportRecord>) -> String {
    match record {
        Some(record) => format!(
            concat!(
                "{{",
                "\"sandbox_id\":{},",
                "\"lease_id\":{},",
                "\"region_id\":{},",
                "\"stage\":{},",
                "\"processing_epoch\":{},",
                "\"detail\":{}",
                "}}"
            ),
            json_escape_string(&record.sandbox_id),
            json_escape_string(&record.lease_id),
            json_escape_string(&record.region_id),
            json_escape_string(&format!("{:?}", record.stage)),
            json_option_u64(record.processing_epoch),
            json_option_string(record.detail.as_deref()),
        ),
        None => "null".into(),
    }
}

pub(crate) fn json_transport_record_vec(records: &[PluginSandboxTransportRecord]) -> String {
    let joined = records
        .iter()
        .map(|record| json_transport_record(Some(record)))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

#[allow(dead_code)]
fn _event_transport_recorder_runtime_helpers_smoke(
    plugin_states: &[PluginSandboxInstanceStateRecord],
    recoveries: &[RecoveryRecord],
    lifecycles: &[PluginSandboxLifecycleRecord],
    transports: &[PluginSandboxTransportRecord],
    heartbeats: &[HeartbeatCycleRecord],
    dispatches: &[BlockDispatchRecord],
    rollovers: &[LeaseRolloverRecord],
) -> usize {
    json_plugin_instance_state_record_vec(plugin_states).len()
        + json_recovery_record_vec(recoveries).len()
        + json_lifecycle_record_vec(lifecycles).len()
        + json_transport_record_vec(transports).len()
        + super::json_heartbeat_record_vec(heartbeats).len()
        + super::json_block_dispatch_record_vec(dispatches).len()
        + super::json_lease_rollover_record_vec(rollovers).len()
}
