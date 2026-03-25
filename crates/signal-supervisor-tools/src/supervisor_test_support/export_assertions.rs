pub(crate) fn assert_transport_fault_export(export: &str) {
    for expected in [
        "\"recovery_events\":1",
        "\"recovery_sequence\":[{",
        "\"intent\":\"WatchdogRecovery\"",
        "\"last_recovery_intent\":\"WatchdogRecovery\"",
        "\"lifecycle_events\":1",
        "\"lifecycle_sequence\":[{",
        "\"stage\":\"TransportAttached\"",
        "\"transport_events\":4",
        "\"transport_sequence\":[{",
        "\"region_id\":\"region-4\"",
        "\"heartbeat_events\":1",
        "\"heartbeat_sequence\":[{",
        "\"block_sequence\":9",
        "\"block_dispatch_events\":1",
        "\"block_dispatch_sequence\":[{",
        "\"completion_state\":\"Completed\"",
        "\"lease_rollover_events\":1",
        "\"lease_rollover_sequence\":[{",
        "\"previous_lease_id\":\"lease-3\"",
        "\"invalidation_events\":1",
        "\"invalidation_sequence\":[{",
        "\"stage\":\"CompletionRegionInvalidated\"",
        "\"completion_slot_events\":2",
        "\"completion_slot_sequence\":[{",
        "\"stage\":\"FallbackApplied\"",
        "\"transport_fault_events\":8",
        "\"last_transport_fault\":{",
        "\"transport_fault_sequence\":[{",
        "\"source\":\"HostBroker\"",
        "\"source\":\"SandboxOperation\"",
        "\"source\":\"RuntimeDispatch\"",
        "\"phase\":\"Dispatch\"",
        "\"phase\":\"Teardown\"",
        "\"resource\":\"SharedMemoryPayload\"",
        "\"resource\":\"SharedMemoryLease\"",
        "\"resource\":\"CompletionSlot\"",
        "\"operation\":\"block_payload.read\"",
        "\"operation\":\"transport.detach_request\"",
        "\"operation\":\"transport.detached\"",
        "\"operation\":\"transport.detach_fault\"",
        "\"operation\":\"completion_region.invalidate\"",
        "\"operation\":\"completion_slot.timeout\"",
        "\"operation\":\"completion_slot.fallback_apply\"",
        "\"operation\":\"processBlock\"",
        "\"stage\":\"TransportDetachRequested\"",
        "\"stage\":\"TransportDetached\"",
        "\"stage\":\"DetachFault\"",
        "\"stage\":\"CompletionRegionInvalidated\"",
        "\"stage\":\"CompletionSlotTimedOut\"",
        "\"transport_fault_summary\":{",
        "\"boundary_mode\":\"FaultAdjacentOnly\"",
        "\"host_broker_events\":4",
        "\"sandbox_operation_events\":1",
        "\"runtime_dispatch_events\":3",
        "\"transport_concurrency_snapshot\":{",
        "\"steady_session_limit\":1",
        "\"recovery_session_limit\":2",
        "\"current_attached_sessions\":0",
        "\"current_lingering_sessions\":0",
        "\"peak_lingering_sessions\":0",
        "\"current_detach_requested_sessions\":0",
        "\"current_detach_faulted_sessions\":0",
        "\"transport_session_summary\":{",
        "\"boundary_mode\":\"HealthyPathVisible\"",
        "\"current_state\":\"DetachFaulted\"",
        "\"currently_attached\":false",
        "\"heartbeat_freshness\":\"Fresh\"",
        "\"dispatch_state\":\"Completed\"",
        "\"current_attached_session_count\":0",
        "\"max_concurrent_attached_sessions\":1",
        "\"attach_events\":1",
        "\"detach_requested_events\":1",
        "\"detached_events\":1",
        "\"detach_fault_events\":1",
        "\"heartbeat_responded_events\":1",
        "\"dispatch_completed_events\":1",
        "\"active_sandbox_id\":null",
        "\"active_lease_id\":null",
        "\"active_region_id\":null",
        "\"active_sessions\":[]",
        "\"last_region_id\":\"region-4\"",
        "\"broker_failure_events\":1",
        "\"broker_failure_sequence\":[{",
        "\"stage\":\"PayloadRead\"",
        "\"sandbox_operation_failure_events\":1",
        "\"sandbox_operation_failure_sequence\":[{",
        "\"stage\":\"ProcessAttach\"",
    ] {
        assert!(export.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_integrated_acceptance_export(export: &str) {
    for expected in [
        "\"fault_status\":{",
        "\"primary_fault_cause\":\"WatchdogRestart\"",
        "\"interruption_summary\":{",
        "\"watchdog_restart_count\":2",
        "\"fault_diagnostic_receipt\":{",
        "\"primary_family\":\"DeferredWorkPressure\"",
        "\"last_deferred_service\":{",
        "\"decision\":\"Defer\"",
        "\"plugin_discovery_snapshot\":{",
        "\"plugin_type_id\":\"plugin:clap:export-consumer\"",
        "\"plugin_type_id\":\"plugin:vst3:export-instrument\"",
        "\"plugin_type_id\":\"plugin:au:export-au\"",
        "\"parity_coverage\":[{",
        "\"supported_platforms\":[\"MacOs\"]",
        "\"unsupported_platforms\":[\"Linux\",\"Windows\"]",
        "\"device_supervision_snapshot\":{",
        "\"external_io_snapshot\":{",
        "\"monitoring_state\":\"Guarded\"",
        "\"drift_state\":\"CrossClockManaged\"",
        "\"duplex_mismatch_state\":\"CrossClockDiverged\"",
        "\"endpoint_topology\":\"Duplex\"",
        "\"media_pipeline_snapshot\":{",
        "\"media_service_snapshot\":{",
        "\"preview_state\":\"Previewing\"",
        "\"invalidated_asset_count\":1",
        "\"media_library_snapshot\":{",
        "\"ready_descriptor_count\":1",
        "\"loudness_ready_descriptor_count\":1",
        "\"character_ready_descriptor_count\":1",
    ] {
        assert!(export.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_transport_liveness_export(export: &str) {
    for expected in [
        "\"active_sessions\":[{",
        "\"sandbox_id\":\"sandbox-a\"",
        "\"state\":\"DetachRequested\"",
        "\"currently_attached\":true",
        "\"heartbeat_freshness\":\"Missed\"",
        "\"dispatch_state\":\"TimedOut\"",
        "\"peak_attached_sessions\":",
        "\"active_block_sequence\":11",
        "\"transport_fault_count\":1",
        "\"last_transport_fault_source\":\"RuntimeDispatch\"",
        "\"last_transport_fault_stage\":\"CompletionSlotTimedOut\"",
        "\"last_transport_fault_phase\":\"Dispatch\"",
        "\"last_transport_fault_processing_epoch\":4",
        "\"last_transport_fault_block_sequence\":11",
        "\"sandbox_id\":\"sandbox-b\"",
        "\"heartbeat_freshness\":\"Fresh\"",
        "\"dispatch_state\":\"Completed\"",
        "\"active_block_sequence\":12",
        "\"last_transport_fault_source\":\"HostBroker\"",
        "\"last_transport_fault_stage\":\"PayloadRead\"",
        "\"last_transport_fault_processing_epoch\":5",
        "\"last_transport_fault_block_sequence\":12",
    ] {
        assert!(export.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_local_summary_json_without_payload(rendered: &str) {
    for expected in [
        "\"sections\":[\"execution\",\"transport\",\"faults\"]",
        "\"debug_sections_supported\":[\"payload\"]",
        "\"debug_sections_enabled\":[]",
        "\"last_recovery_intent\":\"WatchdogRecovery\"",
        "\"last_stop_reason\":\"DegradedModeRecovery\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
    assert!(!rendered.contains("\"payload\":{"));
}

pub(crate) fn assert_local_summary_json_with_payload(rendered: &str) {
    for expected in [
        "\"payload\"",
        "\"generated_event_bytes\"",
        "\"sections\":[\"execution\",\"transport\",\"faults\",\"payload\"]",
        "\"debug_sections_supported\":[\"payload\"]",
        "\"debug_sections_enabled\":[\"payload\"]",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_local_summary_text_sections(default_rendered: &str, payload_rendered: &str) {
    for expected in [
        "sections: execution,transport,faults",
        "debug_sections_supported: payload",
        "debug_sections_enabled: none",
        "last_recovery_intent=Some(WatchdogRecovery)",
        "last_stop_reason=Some(DegradedModeRecovery)",
    ] {
        assert!(default_rendered.contains(expected), "missing {expected}");
    }
    for expected in [
        "sections: execution,transport,faults,payload",
        "debug_sections_supported: payload",
        "debug_sections_enabled: payload",
    ] {
        assert!(payload_rendered.contains(expected), "missing {expected}");
    }
}
