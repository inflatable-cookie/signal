use super::*;

pub(crate) fn json_transport_fault_summary(summary: &TransportFaultSummary) -> String {
    format!(
        concat!(
            "{{",
            "\"boundary_mode\":{},",
            "\"total_events\":{},",
            "\"host_broker_events\":{},",
            "\"sandbox_operation_events\":{},",
            "\"runtime_dispatch_events\":{},",
            "\"prepare_events\":{},",
            "\"dispatch_events\":{},",
            "\"teardown_events\":{},",
            "\"control_events\":{},",
            "\"first_processing_epoch\":{},",
            "\"last_processing_epoch\":{},",
            "\"first_block_sequence\":{},",
            "\"last_block_sequence\":{}",
            "}}"
        ),
        json_escape_string(&format!("{:?}", summary.boundary_mode)),
        summary.total_events,
        summary.host_broker_events,
        summary.sandbox_operation_events,
        summary.runtime_dispatch_events,
        summary.prepare_events,
        summary.dispatch_events,
        summary.teardown_events,
        summary.control_events,
        json_option_u64(summary.first_processing_epoch),
        json_option_u64(summary.last_processing_epoch),
        json_option_u64(summary.first_block_sequence),
        json_option_u64(summary.last_block_sequence),
    )
}

pub(crate) fn json_transport_session_summary(summary: &TransportSessionSummary) -> String {
    format!(
        concat!(
            "{{",
            "\"boundary_mode\":{},",
            "\"current_state\":{},",
            "\"currently_attached\":{},",
            "\"heartbeat_freshness\":{},",
            "\"dispatch_state\":{},",
            "\"current_attached_session_count\":{},",
            "\"max_concurrent_attached_sessions\":{},",
            "\"attach_events\":{},",
            "\"detach_requested_events\":{},",
            "\"detached_events\":{},",
            "\"detach_fault_events\":{},",
            "\"heartbeat_requested_events\":{},",
            "\"heartbeat_responded_events\":{},",
            "\"heartbeat_missed_events\":{},",
            "\"dispatch_requested_events\":{},",
            "\"dispatch_completed_events\":{},",
            "\"dispatch_timed_out_events\":{},",
            "\"first_processing_epoch\":{},",
            "\"last_processing_epoch\":{},",
            "\"first_block_sequence\":{},",
            "\"last_block_sequence\":{},",
            "\"active_sandbox_id\":{},",
            "\"active_lease_id\":{},",
            "\"active_region_id\":{},",
            "\"active_block_sequence\":{},",
            "\"active_sessions\":{},",
            "\"last_sandbox_id\":{},",
            "\"last_lease_id\":{},",
            "\"last_region_id\":{}",
            "}}"
        ),
        json_escape_string(&format!("{:?}", summary.boundary_mode)),
        json_escape_string(&format!("{:?}", summary.current_state)),
        summary.currently_attached,
        json_escape_string(&format!("{:?}", summary.heartbeat_freshness)),
        json_escape_string(&format!("{:?}", summary.dispatch_state)),
        summary.current_attached_session_count,
        summary.max_concurrent_attached_sessions,
        summary.attach_events,
        summary.detach_requested_events,
        summary.detached_events,
        summary.detach_fault_events,
        summary.heartbeat_requested_events,
        summary.heartbeat_responded_events,
        summary.heartbeat_missed_events,
        summary.dispatch_requested_events,
        summary.dispatch_completed_events,
        summary.dispatch_timed_out_events,
        json_option_u64(summary.first_processing_epoch),
        json_option_u64(summary.last_processing_epoch),
        json_option_u64(summary.first_block_sequence),
        json_option_u64(summary.last_block_sequence),
        json_option_string(summary.active_sandbox_id.as_deref()),
        json_option_string(summary.active_lease_id.as_deref()),
        json_option_string(summary.active_region_id.as_deref()),
        json_option_u64(summary.active_block_sequence),
        json_active_transport_session_record_vec(&summary.active_sessions),
        json_option_string(summary.last_sandbox_id.as_deref()),
        json_option_string(summary.last_lease_id.as_deref()),
        json_option_string(summary.last_region_id.as_deref()),
    )
}

pub(crate) fn json_active_transport_session_record(
    record: &ActiveTransportSessionRecord,
) -> String {
    format!(
        concat!(
            "{{",
            "\"sandbox_id\":{},",
            "\"lease_id\":{},",
            "\"region_id\":{},",
            "\"state\":{},",
            "\"currently_attached\":{},",
            "\"heartbeat_freshness\":{},",
            "\"dispatch_state\":{},",
            "\"processing_epoch\":{},",
            "\"active_block_sequence\":{},",
            "\"transport_fault_count\":{},",
            "\"last_transport_fault_source\":{},",
            "\"last_transport_fault_stage\":{},",
            "\"last_transport_fault_phase\":{},",
            "\"last_transport_fault_processing_epoch\":{},",
            "\"last_transport_fault_block_sequence\":{}",
            "}}"
        ),
        json_escape_string(&record.sandbox_id),
        json_escape_string(&record.lease_id),
        json_escape_string(&record.region_id),
        json_escape_string(&format!("{:?}", record.state)),
        record.currently_attached,
        json_escape_string(&format!("{:?}", record.heartbeat_freshness)),
        json_escape_string(&format!("{:?}", record.dispatch_state)),
        json_option_u64(record.processing_epoch),
        json_option_u64(record.active_block_sequence),
        record.transport_fault_count,
        json_option_string(
            record
                .last_transport_fault_source
                .map(|value| format!("{value:?}"))
                .as_deref(),
        ),
        json_option_string(
            record
                .last_transport_fault_stage
                .map(|value| format!("{value:?}"))
                .as_deref(),
        ),
        json_option_string(
            record
                .last_transport_fault_phase
                .map(|value| format!("{value:?}"))
                .as_deref(),
        ),
        json_option_u64(record.last_transport_fault_processing_epoch),
        json_option_u64(record.last_transport_fault_block_sequence),
    )
}

pub(crate) fn json_active_transport_session_record_vec(
    records: &[ActiveTransportSessionRecord],
) -> String {
    let joined = records
        .iter()
        .map(json_active_transport_session_record)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}
