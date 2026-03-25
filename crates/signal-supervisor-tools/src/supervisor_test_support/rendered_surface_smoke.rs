use crate::{
    render_downstream_automation_json, render_downstream_automation_text,
    render_downstream_fail_gates_json, render_downstream_fail_gates_text,
    render_g06_soak_lane_json, render_g06_soak_lane_text, render_generation_closeout_json,
    render_generation_closeout_text, render_host_edge_boundary_json,
    render_host_edge_boundary_text, render_local_summary, render_local_summary_json,
    render_packaging_manifest_json, render_packaging_manifest_text, render_release_boundary_json,
    render_release_boundary_text, ExportDebugOptions,
};

use super::{
    assert_downstream_automation_json, assert_downstream_automation_text,
    assert_downstream_fail_gates_json, assert_downstream_fail_gates_text,
    assert_g06_soak_lane_json, assert_g06_soak_lane_text, assert_generation_closeout_json,
    assert_generation_closeout_text, assert_host_edge_boundary_json,
    assert_host_edge_boundary_text, assert_local_summary_json_with_payload,
    assert_local_summary_json_without_payload, assert_local_summary_text_sections,
    assert_packaging_manifest_json, assert_packaging_manifest_text, assert_release_boundary_json,
    assert_release_boundary_text, sample_local_summary,
};

pub(crate) fn verify_g06_soak_lane_text_reports_required_and_deferred_policy() {
    assert_g06_soak_lane_text(&render_g06_soak_lane_text());
}

pub(crate) fn verify_g06_soak_lane_json_reports_required_and_deferred_policy() {
    assert_g06_soak_lane_json(&render_g06_soak_lane_json());
}

pub(crate) fn verify_host_edge_boundary_text_reports_stable_and_unstable_edges() {
    assert_host_edge_boundary_text(&render_host_edge_boundary_text());
}

pub(crate) fn verify_host_edge_boundary_json_reports_stable_and_unstable_edges() {
    assert_host_edge_boundary_json(&render_host_edge_boundary_json());
}

pub(crate) fn verify_release_boundary_text_reports_packaging_baseline() {
    assert_release_boundary_text(&render_release_boundary_text());
}

pub(crate) fn verify_release_boundary_json_reports_packaging_baseline() {
    assert_release_boundary_json(&render_release_boundary_json());
}

pub(crate) fn verify_packaging_manifest_text_reports_release_bundle_and_receipts() {
    assert_packaging_manifest_text(&render_packaging_manifest_text());
}

pub(crate) fn verify_packaging_manifest_json_reports_release_bundle_and_receipts() {
    assert_packaging_manifest_json(&render_packaging_manifest_json());
}

pub(crate) fn verify_downstream_automation_text_reports_mandatory_and_optional_fixtures() {
    assert_downstream_automation_text(&render_downstream_automation_text());
}

pub(crate) fn verify_downstream_automation_json_reports_mandatory_and_optional_fixtures() {
    assert_downstream_automation_json(&render_downstream_automation_json());
}

pub(crate) fn verify_downstream_fail_gates_text_reports_required_and_deferred_policy() {
    assert_downstream_fail_gates_text(&render_downstream_fail_gates_text());
}

pub(crate) fn verify_downstream_fail_gates_json_reports_required_and_deferred_policy() {
    assert_downstream_fail_gates_json(&render_downstream_fail_gates_json());
}

pub(crate) fn verify_generation_closeout_text_reports_combined_boundary_and_next_queue() {
    assert_generation_closeout_text(&render_generation_closeout_text());
}

pub(crate) fn verify_generation_closeout_json_reports_combined_boundary_and_next_queue() {
    assert_generation_closeout_json(&render_generation_closeout_json());
}

pub(crate) fn verify_local_summary_json_excludes_payload_by_default() {
    let summary = sample_local_summary();
    assert_local_summary_json_without_payload(&render_local_summary_json(
        &summary,
        ExportDebugOptions { payload: false },
    ));
}

pub(crate) fn verify_local_summary_json_includes_payload_when_requested() {
    let summary = sample_local_summary();
    assert_local_summary_json_with_payload(&render_local_summary_json(
        &summary,
        ExportDebugOptions { payload: true },
    ));
}

pub(crate) fn verify_local_summary_text_reports_section_list() {
    let summary = sample_local_summary();
    let default_rendered = render_local_summary(&summary, ExportDebugOptions { payload: false });
    let payload_rendered = render_local_summary(&summary, ExportDebugOptions { payload: true });
    assert_local_summary_text_sections(&default_rendered, &payload_rendered);
}
