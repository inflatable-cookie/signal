#[path = "timeout_prework/engine_snapshot.rs"]
mod engine_snapshot;
#[path = "timeout_prework/transport_continuity.rs"]
mod transport_continuity;

use super::super::*;
use engine_snapshot::assert_timeout_engine_snapshot;
use transport_continuity::assert_timeout_transport_continuity;

#[test]
fn local_host_rolls_leases_forward_after_timeout() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    let summary = host
        .boot_with_timeout_recovery()
        .expect("timeout recovery boot");
    let supervisor = host.supervisor_report();

    assert_eq!(summary.execution.processing_epoch, 2);
    assert_eq!(summary.execution.restart_count, 1);
    assert_eq!(summary.execution.teardown_count, 1);
    assert_eq!(
        summary.execution.last_recovery_intent,
        Some(RecoveryRestartIntent::WatchdogRecovery)
    );
    assert_eq!(
        summary.execution.last_stop_reason,
        Some(StopReason::DegradedModeRecovery)
    );
    assert_eq!(
        summary.execution.last_completion_state,
        CompletionState::Completed
    );
    assert_eq!(summary.execution.processed_blocks, 10);
    assert_eq!(summary.execution.engine_processed_blocks, 10);
    assert_eq!(
        summary.execution.last_block_sequence,
        supervisor
            .observation
            .timeline_snapshot
            .block_sequence_continuity
            .last_block_sequence()
            .expect("last block sequence")
    );
    assert_eq!(
        summary.execution.last_engine_graph_id.as_deref(),
        Some("signal.host.local.demo")
    );
    assert!(
        summary
            .execution
            .last_engine_output_peak
            .unwrap_or_default()
            <= 0.8
    );
    assert!(summary.execution.last_engine_output_rms.is_some());
    assert!(summary.audio_pump.last_callback_output_peak.is_some());

    assert_timeout_engine_snapshot(&summary, &supervisor);
    assert_timeout_transport_continuity(&summary, &supervisor);
}
