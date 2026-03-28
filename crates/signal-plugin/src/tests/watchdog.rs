use super::*;

#[test]
fn sandbox_watchdog_requires_restart_after_consecutive_timeouts() {
    let mut watchdog = SandboxWatchdogState::new(SandboxWatchdogPolicy {
        max_consecutive_deadline_misses: 2,
        max_consecutive_heartbeat_misses: 3,
    });

    assert_eq!(
        watchdog.record_block_completion(CompletionState::TimedOut),
        WatchdogOutcome::Healthy
    );
    assert_eq!(
        watchdog.record_block_completion(CompletionState::TimedOut),
        WatchdogOutcome::RestartRequired {
            reason: WatchdogTriggerReason::DeadlineMisses,
            consecutive_misses: 2,
        }
    );
}

#[test]
fn sandbox_watchdog_resets_heartbeat_misses_after_response() {
    let mut watchdog = SandboxWatchdogState::new(SandboxWatchdogPolicy {
        max_consecutive_deadline_misses: 2,
        max_consecutive_heartbeat_misses: 2,
    });

    assert_eq!(watchdog.record_heartbeat_miss(), WatchdogOutcome::Healthy);
    watchdog.record_heartbeat_response();
    assert_eq!(watchdog.consecutive_heartbeat_misses(), 0);
}

#[test]
fn restart_escalation_requests_safe_mode_after_threshold() {
    let mut escalation = RestartEscalationState::new(RestartEscalationPolicy {
        safe_mode_restart_threshold: 2,
    });

    assert!(!escalation.record_watchdog_restart());
    assert_eq!(escalation.watchdog_restart_count(), 1);
    assert!(escalation.record_watchdog_restart());
    assert!(escalation.safe_mode_requested());
}
