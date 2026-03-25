pub(crate) fn assert_advanced_hardware_boundary_text(rendered: &str) {
    for expected in [
        "advanced_hardware_boundary: signal.runtime.advanced-hardware-boundary",
        "acceptance_task: effigy acceptance:advanced-hardware-boundary",
        "surface: RuntimeObservationReport::control_surface_snapshot and RuntimeSupervisorReport::observation.control_surface_snapshot",
        "cargo test -p signal-runtime public_runtime_advanced_hardware_boundary_reports_runtime_owned_policy_and_feedback_truth",
        "cargo run -p signal-supervisor-tools -- --describe-advanced-hardware-boundary --format=json",
        "display_transport_device_count",
        "motor_transport_device_count",
        "haptic_transport_device_count",
        "scene_mapping_device_count",
        "feedback_page_device_count",
        "safe_action_graph_device_count",
        "display_transport_posture",
        "scene_mapping_posture",
        "feedback_page_posture",
        "feedback_page_class",
        "safe_action_graph_posture",
        "action_authority",
        "safe_action_outcome",
        "feedback_outcome",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_advanced_hardware_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.advanced-hardware-boundary\"",
        "\"contract_path\":\"docs/contracts/061-control-surface-scene-mapping-feedback-pages-and-safe-action-graph-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:advanced-hardware-boundary\"",
        "\"id\":\"runtime-advanced-hardware-report\"",
        "\"id\":\"runtime-advanced-hardware-control-surface-anchor\"",
        "\"id\":\"shared-host-advanced-hardware-report\"",
        "display_transport_device_count",
        "motor_transport_device_count",
        "haptic_transport_device_count",
        "scene_mapping_device_count",
        "feedback_page_device_count",
        "safe_action_graph_device_count",
        "display_content_class",
        "scene_mapping_posture",
        "feedback_page_posture",
        "feedback_page_class",
        "safe_action_graph_posture",
        "action_authority",
        "safe_action_outcome",
        "feedback_outcome",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_recall_portability_boundary_text(rendered: &str) {
    for expected in [
        "recall_portability_boundary: signal.runtime.recall-portability-boundary",
        "acceptance_task: effigy acceptance:recall-portability-boundary",
        "surface: RuntimeObservationReport::plugin_chain_snapshot and RuntimeSupervisorReport::observation.plugin_chain_snapshot",
        "surface: RuntimeObservationApi::get_plugin_recall_handoff_snapshot()",
        "cargo test -p signal-runtime --test public_contract_boundary public_runtime_recall_interchange_and_ara_context_truth_is_consumable_from_reexports",
        "cargo run -p signal-supervisor-tools -- --describe-recall-portability-boundary --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_recall_portability_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.recall-portability-boundary\"",
        "\"contract_path\":\"docs/contracts/024-plugin-preset-state-interchange-portable-recall-and-ara-context-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:recall-portability-boundary\"",
        "\"id\":\"runtime-plugin-chain-recall-report\"",
        "\"id\":\"runtime-plugin-recall-handoff\"",
        "\"id\":\"shared-host-recall-supervisor-report\"",
        "\"id\":\"runtime-recall-portability-public-proof\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_device_supervision_boundary_text(rendered: &str) {
    for expected in [
        "device_supervision_boundary: signal.runtime.device-supervision-boundary",
        "acceptance_task: effigy acceptance:device-supervision-boundary",
        "surface: RuntimeObservationReport::device_supervision_snapshot and RuntimeSupervisorReport::observation.device_supervision_snapshot",
        "surface: RuntimeObservationReport::fault_status and RuntimeObservationReport::interruption_summary",
        "cargo test -p signal-runtime public_runtime_device_supervision_boundary_reports_recovering_and_faulted_runtime_states",
        "cargo run -p signal-supervisor-tools -- --describe-device-supervision-boundary --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_device_supervision_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.device-supervision-boundary\"",
        "\"contract_path\":\"docs/contracts/025-device-supervision-restart-state-machine-and-fault-boundary-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:device-supervision-boundary\"",
        "\"id\":\"runtime-device-supervision-report\"",
        "\"id\":\"runtime-supervision-fault-alignment\"",
        "\"id\":\"shared-host-device-supervision-supervisor-report\"",
        "\"id\":\"runtime-device-supervision-public-proof\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_clock_topology_boundary_text(rendered: &str) {
    for expected in [
        "clock_topology_boundary: signal.runtime.clock-topology-boundary",
        "acceptance_task: effigy acceptance:clock-topology-boundary",
        "surface: RuntimeHostObservationReport::host_io and RuntimeHostSupervisorReport::observation.host_io",
        "surface: LocalRuntimeHost::host_supervisor_report() -> RuntimeHostSupervisorReport",
        "cargo test -p signal-runtime public_runtime_clock_topology_boundary_reports_drift_duplex_and_endpoint_receipts",
        "cargo run -p signal-supervisor-tools -- --describe-clock-topology-boundary --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_clock_topology_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.clock-topology-boundary\"",
        "\"contract_path\":\"docs/contracts/026-clock-domain-drift-duplex-mismatch-and-endpoint-topology-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:clock-topology-boundary\"",
        "\"id\":\"runtime-host-clocking-report\"",
        "\"id\":\"runtime-external-io-alignment\"",
        "\"id\":\"shared-local-host-clock-topology-report\"",
        "\"id\":\"runtime-clock-topology-public-proof\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_external_io_boundary_text(rendered: &str) {
    for expected in [
        "external_io_boundary: signal.runtime.external-io-boundary",
        "acceptance_task: effigy acceptance:external-io-boundary",
        "surface: RuntimeObservationReport::external_io_snapshot and RuntimeSupervisorReport::observation.external_io_snapshot",
        "surface: ServerRuntimeHost::supervisor_report() -> RuntimeSupervisorReport",
        "cargo test -p signal-runtime public_runtime_external_io_boundary_reports_runtime_owned_monitor_and_loopback_truth",
        "cargo run -p signal-supervisor-tools -- --describe-external-io-boundary --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_external_io_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.external-io-boundary\"",
        "\"contract_path\":\"docs/contracts/027-external-io-monitoring-tap-point-and-loopback-measurement-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:external-io-boundary\"",
        "\"id\":\"runtime-external-io-report\"",
        "\"id\":\"runtime-host-external-io-report\"",
        "\"id\":\"shared-local-host-external-io-report\"",
        "\"id\":\"shared-server-host-external-io-report\"",
        "\"id\":\"runtime-external-io-public-proof\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_media_service_boundary_text(rendered: &str) {
    for expected in [
        "media_service_boundary: signal.runtime.media-service-boundary",
        "acceptance_task: effigy acceptance:media-service-boundary",
        "surface: RuntimeObservationReport::media_pipeline_snapshot, RuntimeObservationReport::media_service_snapshot, and RuntimeSupervisorReport::observation.{media_pipeline_snapshot,media_service_snapshot}",
        "surface: supervisor_report() -> RuntimeSupervisorReport",
        "cargo test -p signal-runtime public_runtime_media_service_boundary_reports_runtime_owned_readiness_and_invalidation_truth",
        "cargo run -p signal-supervisor-tools -- --describe-media-service-boundary --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_media_service_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.media-service-boundary\"",
        "\"contract_path\":\"docs/contracts/028-media-indexing-waveform-analysis-and-preview-service-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:media-service-boundary\"",
        "\"id\":\"runtime-media-service-report\"",
        "\"id\":\"runtime-media-service-snapshot\"",
        "\"id\":\"shared-host-media-service-report\"",
        "\"id\":\"runtime-media-service-public-proof\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_analysis_metadata_boundary_text(rendered: &str) {
    for expected in [
        "analysis_metadata_boundary: signal.runtime.analysis-metadata-boundary",
        "acceptance_task: effigy acceptance:analysis-metadata-boundary",
        "surface: RuntimeObservationReport::media_library_snapshot and RuntimeSupervisorReport::observation.media_library_snapshot",
        "surface: RuntimeObservationApi::get_media_library_service_snapshot()",
        "cargo test -p signal-runtime public_runtime_analysis_metadata_boundary_reports_runtime_owned_library_descriptors",
        "cargo run -p signal-supervisor-tools -- --describe-analysis-metadata-boundary --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_analysis_metadata_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.analysis-metadata-boundary\"",
        "\"contract_path\":\"docs/contracts/029-analysis-metadata-extraction-and-library-service-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:analysis-metadata-boundary\"",
        "\"id\":\"runtime-analysis-metadata-report\"",
        "\"id\":\"runtime-analysis-metadata-snapshot\"",
        "\"id\":\"shared-host-analysis-metadata-report\"",
        "\"id\":\"runtime-analysis-metadata-public-proof\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}
