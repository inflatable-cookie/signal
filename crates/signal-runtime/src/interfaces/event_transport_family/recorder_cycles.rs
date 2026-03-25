use super::*;

pub(crate) fn json_heartbeat_record(record: Option<&HeartbeatCycleRecord>) -> String {
    match record {
        Some(record) => format!(
            concat!(
                "{{",
                "\"sandbox_id\":{},",
                "\"stage\":{},",
                "\"processing_epoch\":{},",
                "\"block_sequence\":{}",
                "}}"
            ),
            json_escape_string(&record.sandbox_id),
            json_escape_string(&format!("{:?}", record.stage)),
            json_option_u64(record.processing_epoch),
            json_option_u64(record.block_sequence),
        ),
        None => "null".into(),
    }
}

pub(crate) fn json_heartbeat_record_vec(records: &[HeartbeatCycleRecord]) -> String {
    let joined = records
        .iter()
        .map(|record| json_heartbeat_record(Some(record)))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

pub(crate) fn json_block_dispatch_record(record: Option<&BlockDispatchRecord>) -> String {
    match record {
        Some(record) => format!(
            concat!(
                "{{",
                "\"sandbox_id\":{},",
                "\"lease_id\":{},",
                "\"processing_epoch\":{},",
                "\"block_sequence\":{},",
                "\"frame_count\":{},",
                "\"stage\":{},",
                "\"completion_state\":{}",
                "}}"
            ),
            json_escape_string(&record.sandbox_id),
            json_escape_string(&record.lease_id),
            record.processing_epoch,
            record.block_sequence,
            record.frame_count,
            json_escape_string(&format!("{:?}", record.stage)),
            json_option_string(
                record
                    .completion_state
                    .map(|state| format!("{state:?}"))
                    .as_deref()
            ),
        ),
        None => "null".into(),
    }
}

pub(crate) fn json_block_dispatch_record_vec(records: &[BlockDispatchRecord]) -> String {
    let joined = records
        .iter()
        .map(|record| json_block_dispatch_record(Some(record)))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

pub(crate) fn json_lease_rollover_record(record: Option<&LeaseRolloverRecord>) -> String {
    match record {
        Some(record) => format!(
            concat!(
                "{{",
                "\"sandbox_id\":{},",
                "\"previous_lease_id\":{},",
                "\"lease_id\":{},",
                "\"processing_epoch\":{},",
                "\"first_block_sequence\":{}",
                "}}"
            ),
            json_escape_string(&record.sandbox_id),
            json_escape_string(&record.previous_lease_id),
            json_escape_string(&record.lease_id),
            record.processing_epoch,
            record.first_block_sequence,
        ),
        None => "null".into(),
    }
}

pub(crate) fn json_lease_rollover_record_vec(records: &[LeaseRolloverRecord]) -> String {
    let joined = records
        .iter()
        .map(|record| json_lease_rollover_record(Some(record)))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}
