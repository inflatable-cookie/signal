use signal_runtime::{
    TransportDispatchState, TransportHeartbeatFreshness, TransportSessionBoundaryMode,
    TransportSessionState, TransportSessionSummary,
};

pub fn sample_public_transport_session_summary(
    current_state: TransportSessionState,
    currently_attached: bool,
    heartbeat_freshness: TransportHeartbeatFreshness,
    dispatch_state: TransportDispatchState,
    attach_events: usize,
    detach_requested_events: usize,
    detached_events: usize,
) -> TransportSessionSummary {
    TransportSessionSummary {
        boundary_mode: TransportSessionBoundaryMode::HealthyPathVisible,
        current_state,
        currently_attached,
        heartbeat_freshness,
        dispatch_state,
        current_attached_session_count: usize::from(currently_attached),
        max_concurrent_attached_sessions: usize::from(currently_attached),
        attach_events,
        detach_requested_events,
        detached_events,
        detach_fault_events: 0,
        heartbeat_requested_events: usize::from(matches!(
            heartbeat_freshness,
            TransportHeartbeatFreshness::Requested
                | TransportHeartbeatFreshness::Fresh
                | TransportHeartbeatFreshness::Missed
        )),
        heartbeat_responded_events: usize::from(matches!(
            heartbeat_freshness,
            TransportHeartbeatFreshness::Fresh
        )),
        heartbeat_missed_events: usize::from(matches!(
            heartbeat_freshness,
            TransportHeartbeatFreshness::Missed
        )),
        dispatch_requested_events: usize::from(matches!(
            dispatch_state,
            TransportDispatchState::Requested
                | TransportDispatchState::Completed
                | TransportDispatchState::TimedOut
        )),
        dispatch_completed_events: usize::from(matches!(
            dispatch_state,
            TransportDispatchState::Completed
        )),
        dispatch_timed_out_events: usize::from(matches!(
            dispatch_state,
            TransportDispatchState::TimedOut
        )),
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
    }
}
