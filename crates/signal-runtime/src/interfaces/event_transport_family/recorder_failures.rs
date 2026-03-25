use super::*;

pub(crate) fn json_broker_invalidation_record(record: Option<&BrokerInvalidationRecord>) -> String {
    match record {
        Some(record) => format!(
            concat!(
                "{{",
                "\"sandbox_id\":{},",
                "\"lease_id\":{},",
                "\"processing_epoch\":{},",
                "\"block_sequence\":{},",
                "\"stage\":{},",
                "\"reason\":{}",
                "}}"
            ),
            json_escape_string(&record.sandbox_id),
            json_escape_string(&record.lease_id),
            record.processing_epoch,
            json_option_u64(record.block_sequence),
            json_escape_string(&format!("{:?}", record.stage)),
            json_escape_string(&record.reason),
        ),
        None => "null".into(),
    }
}

pub(crate) fn json_broker_invalidation_record_vec(records: &[BrokerInvalidationRecord]) -> String {
    let joined = records
        .iter()
        .map(|record| json_broker_invalidation_record(Some(record)))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

pub(crate) fn json_completion_slot_record(record: Option<&CompletionSlotRecord>) -> String {
    match record {
        Some(record) => format!(
            concat!(
                "{{",
                "\"sandbox_id\":{},",
                "\"lease_id\":{},",
                "\"processing_epoch\":{},",
                "\"block_sequence\":{},",
                "\"stage\":{}",
                "}}"
            ),
            json_escape_string(&record.sandbox_id),
            json_escape_string(&record.lease_id),
            record.processing_epoch,
            record.block_sequence,
            json_escape_string(&format!("{:?}", record.stage)),
        ),
        None => "null".into(),
    }
}

pub(crate) fn json_completion_slot_record_vec(records: &[CompletionSlotRecord]) -> String {
    let joined = records
        .iter()
        .map(|record| json_completion_slot_record(Some(record)))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

pub(crate) fn json_transport_fault_record(record: Option<&TransportFaultRecord>) -> String {
    match record {
        Some(record) => format!(
            concat!(
                "{{",
                "\"sandbox_id\":{},",
                "\"lease_id\":{},",
                "\"processing_epoch\":{},",
                "\"block_sequence\":{},",
                "\"source\":{},",
                "\"stage\":{},",
                "\"phase\":{},",
                "\"resource\":{},",
                "\"operation\":{},",
                "\"error_kind\":{},",
                "\"detail\":{}",
                "}}"
            ),
            json_escape_string(&record.sandbox_id),
            json_option_string(record.lease_id.as_deref()),
            json_option_u64(record.processing_epoch),
            json_option_u64(record.block_sequence),
            json_escape_string(&format!("{:?}", record.source)),
            json_escape_string(&format!("{:?}", record.stage)),
            json_escape_string(&format!("{:?}", record.phase)),
            json_escape_string(&format!("{:?}", record.resource)),
            json_escape_string(&record.operation),
            json_option_string(record.error_kind.as_deref()),
            json_escape_string(&record.detail),
        ),
        None => "null".into(),
    }
}

pub(crate) fn json_transport_fault_record_vec(records: &[TransportFaultRecord]) -> String {
    let joined = records
        .iter()
        .map(|record| json_transport_fault_record(Some(record)))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

pub(crate) fn json_broker_failure_record(record: Option<&BrokerFailureRecord>) -> String {
    match record {
        Some(record) => format!(
            concat!(
                "{{",
                "\"sandbox_id\":{},",
                "\"lease_id\":{},",
                "\"processing_epoch\":{},",
                "\"block_sequence\":{},",
                "\"stage\":{},",
                "\"detail\":{}",
                "}}"
            ),
            json_escape_string(&record.sandbox_id),
            json_option_string(record.lease_id.as_deref()),
            json_option_u64(record.processing_epoch),
            json_option_u64(record.block_sequence),
            json_escape_string(&format!("{:?}", record.stage)),
            json_escape_string(&record.detail),
        ),
        None => "null".into(),
    }
}

pub(crate) fn json_broker_failure_record_vec(records: &[BrokerFailureRecord]) -> String {
    let joined = records
        .iter()
        .map(|record| json_broker_failure_record(Some(record)))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

pub(crate) fn json_sandbox_operation_failure_record(
    record: Option<&SandboxOperationFailureRecord>,
) -> String {
    match record {
        Some(record) => format!(
            concat!(
                "{{",
                "\"sandbox_id\":{},",
                "\"lease_id\":{},",
                "\"processing_epoch\":{},",
                "\"operation\":{},",
                "\"error_kind\":{},",
                "\"stage\":{},",
                "\"detail\":{}",
                "}}"
            ),
            json_escape_string(&record.sandbox_id),
            json_option_string(record.lease_id.as_deref()),
            json_option_u64(record.processing_epoch),
            json_escape_string(&record.operation),
            json_escape_string(&record.error_kind),
            json_escape_string(&format!("{:?}", record.stage)),
            json_escape_string(&record.detail),
        ),
        None => "null".into(),
    }
}

pub(crate) fn json_sandbox_operation_failure_record_vec(
    records: &[SandboxOperationFailureRecord],
) -> String {
    let joined = records
        .iter()
        .map(|record| json_sandbox_operation_failure_record(Some(record)))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

#[allow(dead_code)]
fn _event_transport_recorder_failure_helpers_smoke(
    invalidations: &[BrokerInvalidationRecord],
    completion_slots: &[CompletionSlotRecord],
    transport_faults: &[TransportFaultRecord],
    broker_failures: &[BrokerFailureRecord],
    sandbox_failures: &[SandboxOperationFailureRecord],
) -> usize {
    json_broker_invalidation_record_vec(invalidations).len()
        + json_completion_slot_record_vec(completion_slots).len()
        + json_transport_fault_record_vec(transport_faults).len()
        + json_broker_failure_record_vec(broker_failures).len()
        + json_sandbox_operation_failure_record_vec(sandbox_failures).len()
}
