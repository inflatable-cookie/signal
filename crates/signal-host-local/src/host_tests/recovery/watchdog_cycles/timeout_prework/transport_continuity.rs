use super::super::super::*;
use crate::LocalRuntimeHostSummary;

pub(super) fn assert_timeout_transport_continuity(
    summary: &LocalRuntimeHostSummary,
    supervisor: &signal_runtime::RuntimeSupervisorReport,
) {
    assert_eq!(summary.last_payload.event_count, 11);
    assert_eq!(summary.last_payload.parameter_event_count, 2);
    assert_eq!(summary.last_payload.parameter_gesture_event_count, 2);
    assert_eq!(summary.last_payload.parameter_modulation_event_count, 2);
    assert_eq!(summary.last_payload.note_event_count, 1);
    assert_eq!(summary.last_payload.note_expression_event_count, 3);
    assert_eq!(summary.last_payload.midi_event_count, 1);
    assert_eq!(summary.last_payload.generated_event_bytes, 268);
    assert_eq!(
        summary.last_payload.first_output_sample,
        Some(summary.execution.last_block_sequence as f32)
    );
    assert_eq!(summary.faults.deadline_misses, 2);
    assert_eq!(summary.faults.heartbeat_misses, 0);
    assert!(summary.faults.watchdog_triggered);
    assert_eq!(
        summary.faults.watchdog_trigger_reason,
        Some(WatchdogTriggerReason::DeadlineMisses)
    );
    assert_eq!(
        supervisor
            .observation
            .supervision_snapshot
            .watchdog_restart_count,
        1
    );
    assert!(
        !supervisor
            .observation
            .supervision_snapshot
            .safe_mode_enabled
    );
    assert!(summary.transport.shared_memory_lease_id.contains("epoch-2"));
    assert!(summary
        .transport
        .shared_memory_region_id
        .starts_with("region-"));
    assert!(
        summary
            .transport
            .shared_memory_path
            .ends_with(".signal-shm")
    );
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .current_attached_sessions,
        1
    );
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .peak_attached_sessions,
        2
    );
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .current_recovery_overlap_sessions,
        0
    );
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .last_admitted_sandbox_id
            .as_deref(),
        Some("local-default-sandbox")
    );
    let automation = &supervisor.observation.automation_snapshot;
    assert_eq!(automation.parameter_id, 4096);
    assert_eq!(automation.value_events, 8);
    assert_eq!(automation.modulation_events, 8);
    assert_eq!(automation.gesture_begin_events, 2);
    assert_eq!(automation.gesture_end_events, 6);
    assert!(automation.first_value.is_some());
    assert!(automation.last_value.is_some(), "{automation:?}");
    assert!(automation.last_modulation.is_some());
    assert_runtime_automation_continuity(&supervisor, 1, 2, &[1, 2], 1);
    assert_runtime_plugin_event_snapshot(&supervisor, 2, 2, &[2], 0);
    let timeline = &supervisor
        .observation
        .timeline_snapshot
        .block_sequence_continuity;
    assert!(timeline.segment_count() >= 2);
    assert!(timeline.first_block_sequence().is_some());
    assert!(timeline
        .last_block_sequence()
        .is_some_and(|last| last >= summary.execution.last_block_sequence));
    assert!(timeline.sequence_gaps <= 1, "{timeline:?}");
    assert_eq!(timeline.lease_rollovers, 1);
    assert_local_plugin_topology(&summary);
    assert_plugin_dispatch_summary(&summary, &supervisor, 2);
}
