use super::super::*;
use super::*;

pub fn apply_transport_session_state(
    summary: &mut TransportSessionSummary,
    active_sessions: &mut BTreeMap<(String, String, String), ActiveTransportSessionRecord>,
    record: &PluginSandboxTransportRecord,
) -> (String, String, String) {
    let key = (
        record.sandbox_id.clone(),
        record.lease_id.clone(),
        record.region_id.clone(),
    );
    let prior_session = active_sessions.remove(&key);
    match record.stage {
        PluginSandboxTransportStage::Attached => {
            summary.current_state = TransportSessionState::AttachActive;
            summary.currently_attached = true;
            summary.active_sandbox_id = Some(record.sandbox_id.clone());
            summary.active_lease_id = Some(record.lease_id.clone());
            summary.active_region_id = Some(record.region_id.clone());
            active_sessions.insert(
                key.clone(),
                ActiveTransportSessionRecord {
                    sandbox_id: record.sandbox_id.clone(),
                    lease_id: record.lease_id.clone(),
                    region_id: record.region_id.clone(),
                    state: TransportSessionState::AttachActive,
                    currently_attached: true,
                    heartbeat_freshness: prior_session
                        .as_ref()
                        .map_or(TransportHeartbeatFreshness::Unknown, |session| {
                            session.heartbeat_freshness
                        }),
                    dispatch_state: prior_session
                        .as_ref()
                        .map_or(TransportDispatchState::Idle, |session| {
                            session.dispatch_state
                        }),
                    processing_epoch: record.processing_epoch.or(prior_session
                        .as_ref()
                        .and_then(|session| session.processing_epoch)),
                    active_block_sequence: prior_session
                        .as_ref()
                        .and_then(|session| session.active_block_sequence),
                    transport_fault_count: prior_session
                        .as_ref()
                        .map_or(0, |session| session.transport_fault_count),
                    last_transport_fault_source: prior_session
                        .as_ref()
                        .and_then(|session| session.last_transport_fault_source),
                    last_transport_fault_stage: prior_session
                        .as_ref()
                        .and_then(|session| session.last_transport_fault_stage),
                    last_transport_fault_phase: prior_session
                        .as_ref()
                        .and_then(|session| session.last_transport_fault_phase),
                    last_transport_fault_processing_epoch: prior_session
                        .as_ref()
                        .and_then(|session| session.last_transport_fault_processing_epoch),
                    last_transport_fault_block_sequence: prior_session
                        .as_ref()
                        .and_then(|session| session.last_transport_fault_block_sequence),
                },
            );
        }
        PluginSandboxTransportStage::DetachRequested => {
            summary.current_state = TransportSessionState::DetachRequested;
            summary.currently_attached = true;
            summary.active_sandbox_id = Some(record.sandbox_id.clone());
            summary.active_lease_id = Some(record.lease_id.clone());
            summary.active_region_id = Some(record.region_id.clone());
            active_sessions.insert(
                key.clone(),
                ActiveTransportSessionRecord {
                    sandbox_id: record.sandbox_id.clone(),
                    lease_id: record.lease_id.clone(),
                    region_id: record.region_id.clone(),
                    state: TransportSessionState::DetachRequested,
                    currently_attached: true,
                    heartbeat_freshness: prior_session
                        .as_ref()
                        .map_or(TransportHeartbeatFreshness::Unknown, |session| {
                            session.heartbeat_freshness
                        }),
                    dispatch_state: prior_session
                        .as_ref()
                        .map_or(TransportDispatchState::Idle, |session| {
                            session.dispatch_state
                        }),
                    processing_epoch: record.processing_epoch.or(prior_session
                        .as_ref()
                        .and_then(|session| session.processing_epoch)),
                    active_block_sequence: prior_session
                        .as_ref()
                        .and_then(|session| session.active_block_sequence),
                    transport_fault_count: prior_session
                        .as_ref()
                        .map_or(0, |session| session.transport_fault_count),
                    last_transport_fault_source: prior_session
                        .as_ref()
                        .and_then(|session| session.last_transport_fault_source),
                    last_transport_fault_stage: prior_session
                        .as_ref()
                        .and_then(|session| session.last_transport_fault_stage),
                    last_transport_fault_phase: prior_session
                        .as_ref()
                        .and_then(|session| session.last_transport_fault_phase),
                    last_transport_fault_processing_epoch: prior_session
                        .as_ref()
                        .and_then(|session| session.last_transport_fault_processing_epoch),
                    last_transport_fault_block_sequence: prior_session
                        .as_ref()
                        .and_then(|session| session.last_transport_fault_block_sequence),
                },
            );
        }
        PluginSandboxTransportStage::Detached => {
            summary.current_state = TransportSessionState::Detached;
            summary.currently_attached = false;
            summary.active_sandbox_id = None;
            summary.active_lease_id = None;
            summary.active_region_id = None;
            summary.active_block_sequence = None;
            active_sessions.remove(&key);
        }
        PluginSandboxTransportStage::DetachFault => {
            summary.current_state = TransportSessionState::DetachFaulted;
            summary.currently_attached = false;
            summary.active_sandbox_id = None;
            summary.active_lease_id = None;
            summary.active_region_id = None;
            summary.active_block_sequence = None;
            active_sessions.remove(&key);
        }
    }
    summary.max_concurrent_attached_sessions = summary
        .max_concurrent_attached_sessions
        .max(active_sessions.len());
    key
}

pub fn resolve_active_session_mut<'a>(
    active_sessions: &'a mut BTreeMap<(String, String, String), ActiveTransportSessionRecord>,
    sandbox_id: &str,
    lease_id: Option<&str>,
    last_transport_key: Option<&(String, String, String)>,
) -> Option<&'a mut ActiveTransportSessionRecord> {
    if let Some(lease_id) = lease_id {
        if let Some(key) = active_sessions
            .keys()
            .find(|(sandbox, lease, _)| sandbox == sandbox_id && lease == lease_id)
            .cloned()
        {
            return active_sessions.get_mut(&key);
        }
    }

    if let Some(key) = last_transport_key {
        if key.0 == sandbox_id {
            return active_sessions.get_mut(key);
        }
    }

    let fallback_key = active_sessions
        .keys()
        .rev()
        .find(|(sandbox, _, _)| sandbox == sandbox_id)
        .cloned()?;
    active_sessions.get_mut(&fallback_key)
}

pub fn update_transport_session_epoch_bounds(
    summary: &mut TransportSessionSummary,
    epoch: Option<u64>,
) {
    if let Some(epoch) = epoch {
        summary.first_processing_epoch = Some(
            summary
                .first_processing_epoch
                .map_or(epoch, |current| current.min(epoch)),
        );
        summary.last_processing_epoch = Some(
            summary
                .last_processing_epoch
                .map_or(epoch, |current| current.max(epoch)),
        );
    }
}

pub fn update_transport_session_block_bounds(
    summary: &mut TransportSessionSummary,
    block_sequence: Option<u64>,
) {
    if let Some(block_sequence) = block_sequence {
        summary.first_block_sequence = Some(
            summary
                .first_block_sequence
                .map_or(block_sequence, |current| current.min(block_sequence)),
        );
        summary.last_block_sequence = Some(
            summary
                .last_block_sequence
                .map_or(block_sequence, |current| current.max(block_sequence)),
        );
    }
}
