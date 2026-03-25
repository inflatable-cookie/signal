pub(crate) fn assert_interruption_boundary_text(rendered: &str) {
    for expected in [
        "interruption_boundary: signal.runtime.interruption-boundary",
        "acceptance_task: effigy acceptance:interruption-boundary",
        "surface: RuntimeObservationReport::fault_status",
        "surface: RuntimeDeferredServiceReceipt::interruption_class",
        "surface: supervisor_report() -> RuntimeSupervisorReport",
        "cargo test -p signal-runtime public_runtime_interruption_boundary_reports_restartable_runtime_state",
        "cargo run -p signal-supervisor-tools -- --describe-interruption-boundary --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_interruption_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.interruption-boundary\"",
        "\"contract_path\":\"docs/contracts/012-runtime-interruption-taxonomy-and-resumability-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:interruption-boundary\"",
        "\"id\":\"runtime-fault-status\"",
        "\"id\":\"offline-render-execution-interruption-receipt\"",
        "\"id\":\"shared-host-supervisor-report\"",
        "\"id\":\"runtime-resumable-deferred-proof\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_fault_diagnostic_boundary_text(rendered: &str) {
    for expected in [
        "fault_diagnostic_boundary: signal.runtime.fault-diagnostic-boundary",
        "acceptance_task: effigy acceptance:fault-diagnostic-boundary",
        "surface: RuntimeObservationReport::fault_diagnostic_receipt and RuntimeSupervisorReport::observation.fault_diagnostic_receipt",
        "surface: RuntimeProfilingReceipt::fault_diagnostic_receipt",
        "cargo test -p signal-runtime public_runtime_fault_diagnostic_boundary_reports_canonical_runtime_receipts",
        "cargo run -p signal-supervisor-tools -- --describe-fault-diagnostic-boundary --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_fault_diagnostic_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.fault-diagnostic-boundary\"",
        "\"contract_path\":\"docs/contracts/016-runtime-fault-cause-attribution-and-diagnostic-receipt-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:fault-diagnostic-boundary\"",
        "\"id\":\"runtime-observation-fault-diagnostic\"",
        "\"id\":\"runtime-profiling-fault-diagnostic\"",
        "\"id\":\"shared-host-fault-diagnostic-report\"",
        "\"id\":\"runtime-public-fault-diagnostic-proof\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_critical_path_boundary_text(rendered: &str) {
    for expected in [
        "critical_path_boundary: signal.runtime.critical-path-boundary",
        "acceptance_task: effigy acceptance:critical-path-boundary",
        "surface: RuntimeObservationReport::performance_snapshot() and RuntimeSupervisorReport::performance_snapshot()",
        "surface: RuntimePerformanceTraceReceipt",
        "cargo test -p signal-runtime public_runtime_critical_path_boundary_reports_bounded_hotspot_receipts",
        "cargo run -p signal-supervisor-tools -- --describe-critical-path-boundary --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_critical_path_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.critical-path-boundary\"",
        "\"contract_path\":\"docs/contracts/018-graph-critical-path-hot-node-and-worker-lane-instrumentation-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:critical-path-boundary\"",
        "\"id\":\"runtime-performance-hotspot-report\"",
        "\"id\":\"runtime-performance-trace-digest\"",
        "\"id\":\"shared-host-critical-path-report\"",
        "\"id\":\"runtime-public-critical-path-proof\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_block_timing_boundary_text(rendered: &str) {
    for expected in [
        "block_timing_boundary: signal.runtime.block-timing-boundary",
        "acceptance_task: effigy acceptance:block-timing-boundary",
        "surface: RuntimeObservationReport::engine_block_snapshot and RuntimeSupervisorReport::observation.engine_block_snapshot",
        "surface: RuntimeObservationReport::performance_snapshot(), RuntimeSupervisorReport::performance_snapshot(), and RuntimePerformanceTraceReceipt",
        "cargo test -p signal-runtime public_runtime_block_timing_boundary_reports_bounded_runtime_measurements",
        "cargo run -p signal-supervisor-tools -- --describe-block-timing-boundary --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_block_timing_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.block-timing-boundary\"",
        "\"contract_path\":\"docs/contracts/017-per-block-execution-timing-and-pressure-snapshot-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:block-timing-boundary\"",
        "\"id\":\"runtime-engine-block-snapshot\"",
        "\"id\":\"runtime-performance-digests\"",
        "\"id\":\"shared-host-block-timing-report\"",
        "\"id\":\"runtime-public-block-timing-proof\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_deferred_work_policy_boundary_text(rendered: &str) {
    for expected in [
        "deferred_work_policy_boundary: signal.runtime.deferred-work-policy-boundary",
        "acceptance_task: effigy acceptance:deferred-work-policy-boundary",
        "surface: RuntimeObservationReport::last_deferred_service_receipt and RuntimeSupervisorReport::observation.last_deferred_service_receipt",
        "surface: RuntimeObservationReport::performance_snapshot(), RuntimeSupervisorReport::performance_snapshot(), and RuntimePerformanceTraceReceipt",
        "cargo test -p signal-runtime public_runtime_deferred_work_policy_boundary_reports_runtime_owned_scheduler_receipts",
        "cargo run -p signal-supervisor-tools -- --describe-deferred-work-policy-boundary --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_deferred_work_policy_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.deferred-work-policy-boundary\"",
        "\"contract_path\":\"docs/contracts/019-deferred-work-scheduler-priority-backpressure-and-cancellation-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:deferred-work-policy-boundary\"",
        "\"id\":\"runtime-deferred-service-policy-receipt\"",
        "\"id\":\"runtime-performance-policy-digests\"",
        "\"id\":\"shared-host-deferred-policy-report\"",
        "\"id\":\"runtime-public-deferred-policy-proof\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_recording_continuity_boundary_text(rendered: &str) {
    for expected in [
        "recording_continuity_boundary: signal.runtime.recording-continuity-boundary",
        "acceptance_task: effigy acceptance:recording-continuity",
        "surface: RuntimeObservationReport::recording_capture_snapshot and RuntimeSupervisorReport::observation.recording_capture_snapshot",
        "surface: RuntimeRecordingCaptureCommitReceipt::committed_checkpoint",
        "cargo test -p signal-runtime public_runtime_recording_continuity_boundary_reports_resumable_restartable_and_terminal_states",
        "cargo run -p signal-supervisor-tools -- --describe-recording-continuity-boundary --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_recording_continuity_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.recording-continuity-boundary\"",
        "\"contract_path\":\"docs/contracts/013-recording-continuity-midi-capture-and-checkpoint-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:recording-continuity\"",
        "\"id\":\"runtime-recording-capture-snapshot\"",
        "\"id\":\"runtime-recording-capture-commit-receipt\"",
        "\"id\":\"shared-host-recording-supervisor-report\"",
        "\"id\":\"runtime-terminal-capture-proof\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_offline_render_continuity_boundary_text(rendered: &str) {
    for expected in [
        "offline_render_continuity_boundary: signal.runtime.offline-render-continuity-boundary",
        "acceptance_task: effigy acceptance:offline-render-continuity",
        "surface: RuntimeObservationReport::offline_render_session_snapshot and RuntimeSupervisorReport::observation.offline_render_session_snapshot",
        "surface: RuntimeObservationApi::get_offline_render_session_snapshot()",
        "cargo test -p signal-runtime public_runtime_offline_render_continuity_boundary_reports_resumable_restartable_and_terminal_states",
        "cargo run -p signal-supervisor-tools -- --describe-offline-render-continuity-boundary --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_offline_render_continuity_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.offline-render-continuity-boundary\"",
        "\"contract_path\":\"docs/contracts/015-offline-render-recovery-and-resumability-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:offline-render-continuity\"",
        "\"id\":\"runtime-offline-render-session-snapshot\"",
        "\"id\":\"runtime-offline-render-observation-api\"",
        "\"id\":\"shared-host-offline-render-supervisor-report\"",
        "\"id\":\"runtime-terminal-render-proof\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_plugin_continuity_boundary_text(rendered: &str) {
    for expected in [
        "plugin_continuity_boundary: signal.runtime.plugin-continuity-boundary",
        "acceptance_task: effigy acceptance:plugin-continuity",
        "surface: RuntimeObservationReport::plugin_lifecycle_snapshot and RuntimeSupervisorReport::observation.plugin_lifecycle_snapshot",
        "surface: RuntimeObservationApi::get_plugin_chain_snapshot()",
        "cargo test -p signal-runtime public_runtime_plugin_continuity_boundary_reports_shared_boundary_and_policy_truth",
        "cargo run -p signal-supervisor-tools -- --describe-plugin-continuity-boundary --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_plugin_continuity_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.plugin-continuity-boundary\"",
        "\"contract_path\":\"docs/contracts/014-plugin-isolation-policy-transport-rebind-and-shared-sandbox-continuity-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:plugin-continuity\"",
        "\"id\":\"runtime-plugin-lifecycle-snapshot\"",
        "\"id\":\"runtime-plugin-chain-snapshot\"",
        "\"id\":\"shared-host-plugin-supervisor-report\"",
        "\"id\":\"runtime-placement-policy-proof\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}
