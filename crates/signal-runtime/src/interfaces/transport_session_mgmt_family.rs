use super::*;

mod transport_session_mgmt_helpers;
use transport_session_mgmt_helpers::*;

/// Scoping policy for transport session observation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportSessionBoundaryMode {
    /// Only healthy (non-faulted) transport sessions are visible.
    HealthyPathVisible,
}

/// Lifecycle state of a transport session between the runtime and a plugin sandbox.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportSessionState {
    /// Session is not attached.
    Detached,
    /// Session attachment is active.
    AttachActive,
    /// A detach has been requested.
    DetachRequested,
    /// Detach encountered a fault.
    DetachFaulted,
}

/// Freshness of the last heartbeat exchange with a sandbox.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportHeartbeatFreshness {
    /// Heartbeat state is unknown (no exchange yet).
    Unknown,
    /// A heartbeat request has been sent and is awaiting a response.
    Requested,
    /// The most recent heartbeat exchange completed successfully.
    Fresh,
    /// A heartbeat response was not received within the deadline.
    Missed,
}

/// Current dispatch pipeline state for a transport session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportDispatchState {
    /// No dispatch is in progress.
    Idle,
    /// A block dispatch has been sent and is awaiting completion.
    Requested,
    /// The most recent block dispatch completed successfully.
    Completed,
    /// The most recent block dispatch timed out.
    TimedOut,
}

/// Point-in-time state of one active transport session: attachment, heartbeat, dispatch, and fault detail.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActiveTransportSessionRecord {
    /// Sandbox ID for this session.
    pub sandbox_id: String,
    /// Lease ID for this session.
    pub lease_id: String,
    /// Region ID for this session.
    pub region_id: String,
    /// Current lifecycle state of this session.
    pub state: TransportSessionState,
    /// Whether this session is currently attached.
    pub currently_attached: bool,
    /// Freshness of the most recent heartbeat for this session.
    pub heartbeat_freshness: TransportHeartbeatFreshness,
    /// Current dispatch state for this session.
    pub dispatch_state: TransportDispatchState,
    /// Processing epoch of the most recent event for this session.
    pub processing_epoch: Option<u64>,
    /// Most recent active block sequence number.
    pub active_block_sequence: Option<u64>,
    /// Number of transport faults recorded for this session.
    pub transport_fault_count: usize,
    /// Source of the most recent transport fault.
    pub last_transport_fault_source: Option<TransportFaultSource>,
    /// Stage of the most recent transport fault.
    pub last_transport_fault_stage: Option<TransportFaultStage>,
    /// Phase of the most recent transport fault.
    pub last_transport_fault_phase: Option<TransportFaultPhase>,
    /// Processing epoch of the most recent transport fault.
    pub last_transport_fault_processing_epoch: Option<u64>,
    /// Block sequence number of the most recent transport fault.
    pub last_transport_fault_block_sequence: Option<u64>,
}

/// Aggregate transport session summary: overall state, event counts, epoch bounds, and active session list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransportSessionSummary {
    /// Scoping policy for this session summary.
    pub boundary_mode: TransportSessionBoundaryMode,
    /// Current overall session state.
    pub current_state: TransportSessionState,
    /// Whether any session is currently attached.
    pub currently_attached: bool,
    /// Freshness of the most recent heartbeat.
    pub heartbeat_freshness: TransportHeartbeatFreshness,
    /// Current dispatch state.
    pub dispatch_state: TransportDispatchState,
    /// Number of currently attached sessions.
    pub current_attached_session_count: usize,
    /// Peak number of concurrently attached sessions observed.
    pub max_concurrent_attached_sessions: usize,
    /// Total number of attach events.
    pub attach_events: usize,
    /// Total number of detach-requested events.
    pub detach_requested_events: usize,
    /// Total number of detached events.
    pub detached_events: usize,
    /// Total number of detach-fault events.
    pub detach_fault_events: usize,
    /// Total number of heartbeat-requested events.
    pub heartbeat_requested_events: usize,
    /// Total number of heartbeat-responded events.
    pub heartbeat_responded_events: usize,
    /// Total number of heartbeat-missed events.
    pub heartbeat_missed_events: usize,
    /// Total number of dispatch-requested events.
    pub dispatch_requested_events: usize,
    /// Total number of dispatch-completed events.
    pub dispatch_completed_events: usize,
    /// Total number of dispatch-timed-out events.
    pub dispatch_timed_out_events: usize,
    /// Earliest processing epoch seen across all events.
    pub first_processing_epoch: Option<u64>,
    /// Latest processing epoch seen across all events.
    pub last_processing_epoch: Option<u64>,
    /// Earliest block sequence seen across all events.
    pub first_block_sequence: Option<u64>,
    /// Latest block sequence seen across all events.
    pub last_block_sequence: Option<u64>,
    /// Sandbox ID of the currently active session.
    pub active_sandbox_id: Option<String>,
    /// Lease ID of the currently active session.
    pub active_lease_id: Option<String>,
    /// Region ID of the currently active session.
    pub active_region_id: Option<String>,
    /// Most recently seen active block sequence.
    pub active_block_sequence: Option<u64>,
    /// List of currently active transport session records.
    pub active_sessions: Vec<ActiveTransportSessionRecord>,
    /// Sandbox ID from the most recent transport event.
    pub last_sandbox_id: Option<String>,
    /// Lease ID from the most recent transport event.
    pub last_lease_id: Option<String>,
    /// Region ID from the most recent transport event.
    pub last_region_id: Option<String>,
}

impl TransportSessionSummary {
    /// Builds a transport session summary by replaying all transport, heartbeat, dispatch, and fault events from the diagnostics log.
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
