use crate::supervisor_test_support::*;

#[test]
fn host_edge_boundary_text_reports_stable_and_unstable_edges() {
    verify_host_edge_boundary_text_reports_stable_and_unstable_edges();
}

#[test]
fn host_edge_boundary_json_reports_stable_and_unstable_edges() {
    verify_host_edge_boundary_json_reports_stable_and_unstable_edges();
}

#[test]
fn release_boundary_text_reports_packaging_baseline() {
    verify_release_boundary_text_reports_packaging_baseline();
}

#[test]
fn release_boundary_json_reports_packaging_baseline() {
    verify_release_boundary_json_reports_packaging_baseline();
}

#[test]
fn packaging_manifest_text_reports_release_bundle_and_receipts() {
    verify_packaging_manifest_text_reports_release_bundle_and_receipts();
}

#[test]
fn packaging_manifest_json_reports_release_bundle_and_receipts() {
    verify_packaging_manifest_json_reports_release_bundle_and_receipts();
}

#[test]
fn downstream_automation_text_reports_mandatory_and_optional_fixtures() {
    verify_downstream_automation_text_reports_mandatory_and_optional_fixtures();
}

#[test]
fn downstream_automation_json_reports_mandatory_and_optional_fixtures() {
    verify_downstream_automation_json_reports_mandatory_and_optional_fixtures();
}

#[test]
fn downstream_fail_gates_text_reports_required_and_deferred_policy() {
    verify_downstream_fail_gates_text_reports_required_and_deferred_policy();
}

#[test]
fn downstream_fail_gates_json_reports_required_and_deferred_policy() {
    verify_downstream_fail_gates_json_reports_required_and_deferred_policy();
}

#[test]
fn generation_closeout_text_reports_combined_boundary_and_next_queue() {
    verify_generation_closeout_text_reports_combined_boundary_and_next_queue();
}

#[test]
fn generation_closeout_json_reports_combined_boundary_and_next_queue() {
    verify_generation_closeout_json_reports_combined_boundary_and_next_queue();
}

#[test]
fn export_json_is_versioned() {
    verify_export_json_is_versioned();
}

#[test]
fn export_json_carries_last_deferred_service_receipt() {
    verify_export_json_carries_last_deferred_service_receipt();
}

#[test]
fn export_json_carries_runtime_owned_plugin_discovery_catalog() {
    verify_export_json_carries_runtime_owned_plugin_discovery_catalog();
}

#[test]
fn export_json_carries_runtime_owned_plugin_discovery_capability_coverage() {
    verify_export_json_carries_runtime_owned_plugin_discovery_capability_coverage();
}

#[test]
fn export_json_carries_runtime_recovery_sequence() {
    verify_export_json_carries_runtime_recovery_sequence();
}

#[test]
fn export_json_carries_cross_family_integrated_acceptance_evidence() {
    verify_export_json_carries_cross_family_integrated_acceptance_evidence();
}

#[test]
fn export_json_serializes_per_session_transport_liveness() {
    verify_export_json_serializes_per_session_transport_liveness();
}

#[test]
fn local_summary_json_excludes_payload_by_default() {
    verify_local_summary_json_excludes_payload_by_default();
}

#[test]
fn local_summary_json_includes_payload_when_requested() {
    verify_local_summary_json_includes_payload_when_requested();
}

#[test]
fn local_summary_text_reports_section_list() {
    verify_local_summary_text_reports_section_list();
}
