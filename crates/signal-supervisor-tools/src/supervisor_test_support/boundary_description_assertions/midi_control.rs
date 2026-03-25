pub(crate) fn assert_external_midi_boundary_text(rendered: &str) {
    for expected in [
        "external_midi_boundary: signal.runtime.external-midi-boundary",
        "acceptance_task: effigy acceptance:external-midi-boundary",
        "contract_path: docs/contracts/065-live-external-midi-device-ownership-and-backend-parity-contract.md",
        "surface: RuntimeObservationReport::external_midi_snapshot and RuntimeSupervisorReport::observation.external_midi_snapshot",
        "live_ownership",
        "ownership_posture",
        "attach_continuity",
        "backend_parity",
        "guarded_parity_outcome",
        "cargo test -p signal-runtime public_runtime_external_midi_boundary_reports_runtime_owned_endpoint_graph_truth",
        "cargo run -p signal-supervisor-tools -- --describe-external-midi-boundary --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_external_midi_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.external-midi-boundary\"",
        "\"contract_path\":\"docs/contracts/065-live-external-midi-device-ownership-and-backend-parity-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:external-midi-boundary\"",
        "\"id\":\"runtime-external-midi-report\"",
        "\"id\":\"shared-host-external-midi-report\"",
        "\"id\":\"runtime-external-midi-public-proof\"",
        "live_ownership",
        "ownership_posture",
        "attach_continuity",
        "backend_parity",
        "guarded_parity_outcome",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_generic_event_boundary_text(rendered: &str) {
    for expected in [
        "generic_event_boundary: signal.runtime.generic-event-boundary",
        "acceptance_task: effigy acceptance:generic-event-boundary",
        "surface: RuntimeObservationReport::plugin_event_snapshot and RuntimeSupervisorReport::observation.plugin_event_snapshot",
        "surface: RuntimeObservationApi::get_plugin_discovery_snapshot() capability_coverage.supports_note_expression_count",
        "cargo test -p signal-runtime public_runtime_generic_event_boundary_reports_runtime_owned_event_and_capability_truth",
        "cargo run -p signal-supervisor-tools -- --describe-generic-event-boundary --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_generic_event_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.generic-event-boundary\"",
        "\"contract_path\":\"docs/contracts/023-generic-midi-note-expression-and-plugin-event-model-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:generic-event-boundary\"",
        "\"id\":\"runtime-generic-event-report\"",
        "\"id\":\"runtime-generic-event-capability-coverage\"",
        "\"id\":\"shared-host-generic-event-supervisor-report\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_controller_expression_boundary_text(rendered: &str) {
    for expected in [
        "controller_expression_boundary: signal.runtime.controller-expression-boundary",
        "acceptance_task: effigy acceptance:controller-expression-boundary",
        "surface: RuntimeObservationReport::plugin_event_snapshot and RuntimeSupervisorReport::observation.plugin_event_snapshot",
        "surface: RuntimeObservationReport::external_midi_snapshot.endpoints[*].capability and RuntimeSupervisorReport::observation.external_midi_snapshot.endpoints[*].capability",
        "cargo test -p signal-runtime public_runtime_controller_expression_boundary_reports_runtime_owned_expression_truth",
        "cargo run -p signal-supervisor-tools -- --describe-controller-expression-boundary --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_controller_expression_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.controller-expression-boundary\"",
        "\"contract_path\":\"docs/contracts/043-midi-2-0-mpe-and-richer-controller-expression-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:controller-expression-boundary\"",
        "\"id\":\"runtime-controller-expression-report\"",
        "\"id\":\"runtime-controller-expression-device-capability\"",
        "\"id\":\"shared-host-controller-expression-report\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_control_surface_boundary_text(rendered: &str) {
    for expected in [
        "control_surface_boundary: signal.runtime.control-surface-boundary",
        "acceptance_task: effigy acceptance:control-surface-boundary",
        "surface: RuntimeObservationReport::control_surface_snapshot and RuntimeSupervisorReport::observation.control_surface_snapshot",
        "surface: RuntimeObservationReport::external_midi_snapshot and RuntimeSupervisorReport::observation.external_midi_snapshot",
        "cargo test -p signal-runtime public_runtime_control_surface_boundary_reports_runtime_owned_transport_and_feedback_truth",
        "cargo run -p signal-supervisor-tools -- --describe-control-surface-boundary --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_control_surface_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.control-surface-boundary\"",
        "\"contract_path\":\"docs/contracts/044-control-surface-transport-mapping-and-feedback-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:control-surface-boundary\"",
        "\"id\":\"runtime-control-surface-report\"",
        "\"id\":\"runtime-control-surface-external-midi-anchor\"",
        "\"id\":\"shared-host-control-surface-report\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}
