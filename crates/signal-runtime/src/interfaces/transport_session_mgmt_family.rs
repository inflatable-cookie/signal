use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportSessionBoundaryMode {
    HealthyPathVisible,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportSessionState {
    Detached,
    AttachActive,
    DetachRequested,
    DetachFaulted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportHeartbeatFreshness {
    Unknown,
    Requested,
    Fresh,
    Missed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportDispatchState {
    Idle,
    Requested,
    Completed,
    TimedOut,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActiveTransportSessionRecord {
    pub sandbox_id: String,
    pub lease_id: String,
    pub region_id: String,
    pub state: TransportSessionState,
    pub currently_attached: bool,
    pub heartbeat_freshness: TransportHeartbeatFreshness,
    pub dispatch_state: TransportDispatchState,
    pub processing_epoch: Option<u64>,
    pub active_block_sequence: Option<u64>,
    pub transport_fault_count: usize,
    pub last_transport_fault_source: Option<TransportFaultSource>,
    pub last_transport_fault_stage: Option<TransportFaultStage>,
    pub last_transport_fault_phase: Option<TransportFaultPhase>,
    pub last_transport_fault_processing_epoch: Option<u64>,
    pub last_transport_fault_block_sequence: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransportSessionSummary {
    pub boundary_mode: TransportSessionBoundaryMode,
    pub current_state: TransportSessionState,
    pub currently_attached: bool,
    pub heartbeat_freshness: TransportHeartbeatFreshness,
    pub dispatch_state: TransportDispatchState,
    pub current_attached_session_count: usize,
    pub max_concurrent_attached_sessions: usize,
    pub attach_events: usize,
    pub detach_requested_events: usize,
    pub detached_events: usize,
    pub detach_fault_events: usize,
    pub heartbeat_requested_events: usize,
    pub heartbeat_responded_events: usize,
    pub heartbeat_missed_events: usize,
    pub dispatch_requested_events: usize,
    pub dispatch_completed_events: usize,
    pub dispatch_timed_out_events: usize,
    pub first_processing_epoch: Option<u64>,
    pub last_processing_epoch: Option<u64>,
    pub first_block_sequence: Option<u64>,
    pub last_block_sequence: Option<u64>,
    pub active_sandbox_id: Option<String>,
    pub active_lease_id: Option<String>,
    pub active_region_id: Option<String>,
    pub active_block_sequence: Option<u64>,
    pub active_sessions: Vec<ActiveTransportSessionRecord>,
    pub last_sandbox_id: Option<String>,
    pub last_lease_id: Option<String>,
    pub last_region_id: Option<String>,
}

impl TransportSessionSummary {
    pub fn from_diagnostics(diagnostics: &RuntimeObservationDiagnostics) -> Self {
        let mut summary = Self {
            boundary_mode: TransportSessionBoundaryMode::HealthyPathVisible,
            current_state: TransportSessionState::Detached,
            currently_attached: false,
            heartbeat_freshness: TransportHeartbeatFreshness::Unknown,
            dispatch_state: TransportDispatchState::Idle,
            current_attached_session_count: 0,
            max_concurrent_attached_sessions: 0,
            attach_events: 0,
            detach_requested_events: 0,
            detached_events: 0,
            detach_fault_events: 0,
            heartbeat_requested_events: 0,
            heartbeat_responded_events: 0,
            heartbeat_missed_events: 0,
            dispatch_requested_events: 0,
            dispatch_completed_events: 0,
            dispatch_timed_out_events: 0,
            first_processing_epoch: None,
            last_processing_epoch: None,
            first_block_sequence: None,
            last_block_sequence: None,
            active_sandbox_id: None,
            active_lease_id: None,
            active_region_id: None,
            active_block_sequence: None,
            active_sessions: Vec::new(),
            last_sandbox_id: None,
            last_lease_id: None,
            last_region_id: None,
        };
        let mut active_sessions: BTreeMap<(String, String, String), ActiveTransportSessionRecord> =
            BTreeMap::new();
        let mut last_transport_key: Option<(String, String, String)> = None;

        for record in &diagnostics.transport_events {
            match record.stage {
                PluginSandboxTransportStage::Attached => {
                    summary.attach_events = summary.attach_events.saturating_add(1)
                }
                PluginSandboxTransportStage::DetachRequested => {
                    summary.detach_requested_events =
                        summary.detach_requested_events.saturating_add(1)
                }
                PluginSandboxTransportStage::Detached => {
                    summary.detached_events = summary.detached_events.saturating_add(1)
                }
                PluginSandboxTransportStage::DetachFault => {
                    summary.detach_fault_events = summary.detach_fault_events.saturating_add(1)
                }
            }
            update_transport_session_epoch_bounds(&mut summary, record.processing_epoch);
            last_transport_key = Some(apply_transport_session_state(
                &mut summary,
                &mut active_sessions,
                record,
            ));
            summary.last_sandbox_id = Some(record.sandbox_id.clone());
            summary.last_lease_id = Some(record.lease_id.clone());
            summary.last_region_id = Some(record.region_id.clone());
        }

        for record in &diagnostics.heartbeat_events {
            match record.stage {
                HeartbeatCycleStage::Requested => {
                    summary.heartbeat_requested_events =
                        summary.heartbeat_requested_events.saturating_add(1);
                    summary.heartbeat_freshness = TransportHeartbeatFreshness::Requested;
                }
                HeartbeatCycleStage::Responded => {
                    summary.heartbeat_responded_events =
                        summary.heartbeat_responded_events.saturating_add(1);
                    summary.heartbeat_freshness = TransportHeartbeatFreshness::Fresh;
                }
                HeartbeatCycleStage::Missed => {
                    summary.heartbeat_missed_events =
                        summary.heartbeat_missed_events.saturating_add(1);
                    summary.heartbeat_freshness = TransportHeartbeatFreshness::Missed;
                }
            }
            update_transport_session_epoch_bounds(&mut summary, record.processing_epoch);
            update_transport_session_block_bounds(&mut summary, record.block_sequence);
            if let Some(session) = resolve_active_session_mut(
                &mut active_sessions,
                &record.sandbox_id,
                None,
                last_transport_key.as_ref(),
            ) {
                session.heartbeat_freshness = summary.heartbeat_freshness;
                session.processing_epoch = record.processing_epoch.or(session.processing_epoch);
            }
            summary.last_sandbox_id = Some(record.sandbox_id.clone());
        }

        for record in &diagnostics.block_dispatch_events {
            match record.stage {
                BlockDispatchStage::Requested => {
                    summary.dispatch_requested_events =
                        summary.dispatch_requested_events.saturating_add(1);
                    summary.dispatch_state = TransportDispatchState::Requested;
                }
                BlockDispatchStage::Completed => {
                    summary.dispatch_completed_events =
                        summary.dispatch_completed_events.saturating_add(1);
                    summary.dispatch_state = TransportDispatchState::Completed;
                }
                BlockDispatchStage::TimedOut => {
                    summary.dispatch_timed_out_events =
                        summary.dispatch_timed_out_events.saturating_add(1);
                    summary.dispatch_state = TransportDispatchState::TimedOut;
                }
            }
            update_transport_session_epoch_bounds(&mut summary, Some(record.processing_epoch));
            update_transport_session_block_bounds(&mut summary, Some(record.block_sequence));
            summary.last_sandbox_id = Some(record.sandbox_id.clone());
            summary.last_lease_id = Some(record.lease_id.clone());
            summary.active_block_sequence = Some(record.block_sequence);
            if let Some(session) = resolve_active_session_mut(
                &mut active_sessions,
                &record.sandbox_id,
                Some(&record.lease_id),
                last_transport_key.as_ref(),
            ) {
                session.dispatch_state = summary.dispatch_state;
                session.processing_epoch = Some(record.processing_epoch);
                session.active_block_sequence = Some(record.block_sequence);
            }
        }

        for record in &diagnostics.transport_fault_events {
            if let Some(session) = resolve_active_session_mut(
                &mut active_sessions,
                &record.sandbox_id,
                record.lease_id.as_deref(),
                last_transport_key.as_ref(),
            ) {
                session.transport_fault_count = session.transport_fault_count.saturating_add(1);
                session.last_transport_fault_source = Some(record.source);
                session.last_transport_fault_stage = Some(record.stage);
                session.last_transport_fault_phase = Some(record.phase);
                session.last_transport_fault_processing_epoch = record.processing_epoch;
                session.last_transport_fault_block_sequence = record.block_sequence;
                session.processing_epoch = record.processing_epoch.or(session.processing_epoch);
                if let Some(block_sequence) = record.block_sequence {
                    session.active_block_sequence = Some(block_sequence);
                }
            }
        }

        summary.current_attached_session_count = active_sessions.len();
        summary.active_sessions = active_sessions.into_values().collect();

        summary
    }
}

fn apply_transport_session_state(
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

fn resolve_active_session_mut<'a>(
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

fn update_transport_session_epoch_bounds(
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

fn update_transport_session_block_bounds(
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
