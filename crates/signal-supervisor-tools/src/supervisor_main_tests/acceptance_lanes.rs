use crate::{
    render_control_preview_workflow_acceptance_lane_json,
    render_control_preview_workflow_acceptance_lane_text,
    render_device_workflow_acceptance_lane_json, render_device_workflow_acceptance_lane_text,
    render_g07_acceptance_lane_json, render_g07_acceptance_lane_text,
    render_immersive_acceptance_lane_json, render_immersive_acceptance_lane_text,
    render_integrated_acceptance_lane_json, render_integrated_acceptance_lane_text,
    render_integrated_live_workflow_acceptance_lane_json,
    render_integrated_live_workflow_acceptance_lane_text, render_linux_live_acceptance_lane_json,
    render_linux_live_acceptance_lane_text, supervisor_test_support::*,
};

#[test]
fn integrated_acceptance_lane_text_reports_required_and_advisory_policy() {
    assert_integrated_acceptance_lane_text(&render_integrated_acceptance_lane_text());
}

#[test]
fn integrated_acceptance_lane_json_reports_required_and_advisory_policy() {
    assert_integrated_acceptance_lane_json(&render_integrated_acceptance_lane_json());
}

#[test]
fn g07_acceptance_lane_text_reports_required_and_advisory_policy() {
    assert_g07_acceptance_lane_text(&render_g07_acceptance_lane_text());
}

#[test]
fn g07_acceptance_lane_json_reports_required_and_advisory_policy() {
    assert_g07_acceptance_lane_json(&render_g07_acceptance_lane_json());
}

#[test]
fn device_workflow_acceptance_lane_text_reports_required_and_deferred_policy() {
    assert_device_workflow_acceptance_lane_text(&render_device_workflow_acceptance_lane_text());
}

#[test]
fn device_workflow_acceptance_lane_json_reports_required_and_deferred_policy() {
    assert_device_workflow_acceptance_lane_json(&render_device_workflow_acceptance_lane_json());
}

#[test]
fn linux_live_acceptance_lane_text_reports_required_and_deferred_policy() {
    assert_linux_live_acceptance_lane_text(&render_linux_live_acceptance_lane_text());
}

#[test]
fn linux_live_acceptance_lane_json_reports_required_and_deferred_policy() {
    assert_linux_live_acceptance_lane_json(&render_linux_live_acceptance_lane_json());
}

#[test]
fn immersive_acceptance_lane_text_reports_required_and_deferred_policy() {
    assert_immersive_acceptance_lane_text(&render_immersive_acceptance_lane_text());
}

#[test]
fn immersive_acceptance_lane_json_reports_required_and_deferred_policy() {
    assert_immersive_acceptance_lane_json(&render_immersive_acceptance_lane_json());
}

#[test]
fn control_preview_workflow_acceptance_lane_text_reports_required_and_deferred_policy() {
    assert_control_preview_workflow_acceptance_lane_text(
        &render_control_preview_workflow_acceptance_lane_text(),
    );
}

#[test]
fn control_preview_workflow_acceptance_lane_json_reports_required_and_deferred_policy() {
    assert_control_preview_workflow_acceptance_lane_json(
        &render_control_preview_workflow_acceptance_lane_json(),
    );
}

#[test]
fn integrated_live_workflow_acceptance_lane_text_reports_required_and_deferred_policy() {
    assert_integrated_live_workflow_acceptance_lane_text(
        &render_integrated_live_workflow_acceptance_lane_text(),
    );
}

#[test]
fn integrated_live_workflow_acceptance_lane_json_reports_required_and_deferred_policy() {
    assert_integrated_live_workflow_acceptance_lane_json(
        &render_integrated_live_workflow_acceptance_lane_json(),
    );
}

#[test]
fn export_json_carries_cross_family_device_workflow_acceptance_evidence() {
    verify_export_json_carries_cross_family_device_workflow_acceptance_evidence();
}

#[test]
fn export_json_carries_cross_family_linux_live_acceptance_evidence() {
    verify_export_json_carries_cross_family_linux_live_acceptance_evidence();
}

#[test]
fn export_json_carries_cross_family_immersive_acceptance_evidence() {
    verify_export_json_carries_cross_family_immersive_acceptance_evidence();
}

#[test]
fn export_json_carries_cross_family_control_preview_workflow_acceptance_evidence() {
    verify_export_json_carries_cross_family_control_preview_workflow_acceptance_evidence();
}

#[test]
fn export_json_carries_cross_family_integrated_live_workflow_acceptance_evidence() {
    verify_export_json_carries_cross_family_integrated_live_workflow_acceptance_evidence();
}

#[test]
fn export_json_carries_cross_family_g07_acceptance_evidence() {
    verify_export_json_carries_cross_family_g07_acceptance_evidence();
}

#[test]
fn g06_soak_lane_text_reports_required_and_deferred_policy() {
    verify_g06_soak_lane_text_reports_required_and_deferred_policy();
}

#[test]
fn g06_soak_lane_json_reports_required_and_deferred_policy() {
    verify_g06_soak_lane_json_reports_required_and_deferred_policy();
}
