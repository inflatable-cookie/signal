use crate::{
    render_advanced_hardware_boundary_json, render_advanced_hardware_boundary_text,
    render_analysis_metadata_boundary_json, render_analysis_metadata_boundary_text,
    render_clock_topology_boundary_json, render_clock_topology_boundary_text,
    render_complex_io_boundary_json, render_complex_io_boundary_text,
    render_control_surface_boundary_json, render_control_surface_boundary_text,
    render_controller_expression_boundary_json, render_controller_expression_boundary_text,
    render_device_supervision_boundary_json, render_device_supervision_boundary_text,
    render_external_io_boundary_json, render_external_io_boundary_text,
    render_external_midi_boundary_json, render_external_midi_boundary_text,
    render_generic_event_boundary_json, render_generic_event_boundary_text,
    render_marker_analysis_boundary_json, render_marker_analysis_boundary_text,
    render_media_service_boundary_json, render_media_service_boundary_text,
    render_multi_bus_boundary_json, render_multi_bus_boundary_text,
    render_multichannel_boundary_json, render_multichannel_boundary_text,
    render_preview_transform_boundary_json, render_preview_transform_boundary_text,
    render_recall_portability_boundary_json, render_recall_portability_boundary_text,
    render_sidechain_boundary_json, render_sidechain_boundary_text, render_spatial_boundary_json,
    render_spatial_boundary_text, render_stretch_boundary_json, render_stretch_boundary_text,
    render_transform_artifact_boundary_json, render_transform_artifact_boundary_text,
    supervisor_test_support::*,
};

#[test]
fn external_midi_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_external_midi_boundary_text(&render_external_midi_boundary_text());
}

#[test]
fn external_midi_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_external_midi_boundary_json(&render_external_midi_boundary_json());
}

#[test]
fn generic_event_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_generic_event_boundary_text(&render_generic_event_boundary_text());
}

#[test]
fn generic_event_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_generic_event_boundary_json(&render_generic_event_boundary_json());
}

#[test]
fn controller_expression_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_controller_expression_boundary_text(&render_controller_expression_boundary_text());
}

#[test]
fn controller_expression_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_controller_expression_boundary_json(&render_controller_expression_boundary_json());
}

#[test]
fn control_surface_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_control_surface_boundary_text(&render_control_surface_boundary_text());
}

#[test]
fn control_surface_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_control_surface_boundary_json(&render_control_surface_boundary_json());
}

#[test]
fn advanced_hardware_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_advanced_hardware_boundary_text(&render_advanced_hardware_boundary_text());
}

#[test]
fn advanced_hardware_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_advanced_hardware_boundary_json(&render_advanced_hardware_boundary_json());
}

#[test]
fn recall_portability_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_recall_portability_boundary_text(&render_recall_portability_boundary_text());
}

#[test]
fn recall_portability_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_recall_portability_boundary_json(&render_recall_portability_boundary_json());
}

#[test]
fn device_supervision_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_device_supervision_boundary_text(&render_device_supervision_boundary_text());
}

#[test]
fn device_supervision_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_device_supervision_boundary_json(&render_device_supervision_boundary_json());
}

#[test]
fn clock_topology_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_clock_topology_boundary_text(&render_clock_topology_boundary_text());
}

#[test]
fn clock_topology_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_clock_topology_boundary_json(&render_clock_topology_boundary_json());
}

#[test]
fn external_io_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_external_io_boundary_text(&render_external_io_boundary_text());
}

#[test]
fn external_io_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_external_io_boundary_json(&render_external_io_boundary_json());
}

#[test]
fn media_service_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_media_service_boundary_text(&render_media_service_boundary_text());
}

#[test]
fn media_service_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_media_service_boundary_json(&render_media_service_boundary_json());
}

#[test]
fn analysis_metadata_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_analysis_metadata_boundary_text(&render_analysis_metadata_boundary_text());
}

#[test]
fn analysis_metadata_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_analysis_metadata_boundary_json(&render_analysis_metadata_boundary_json());
}

#[test]
fn multichannel_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_multichannel_boundary_text(&render_multichannel_boundary_text());
}

#[test]
fn multichannel_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_multichannel_boundary_json(&render_multichannel_boundary_json());
}

#[test]
fn multi_bus_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_multi_bus_boundary_text(&render_multi_bus_boundary_text());
}

#[test]
fn multi_bus_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_multi_bus_boundary_json(&render_multi_bus_boundary_json());
}

#[test]
fn sidechain_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_sidechain_boundary_text(&render_sidechain_boundary_text());
}

#[test]
fn sidechain_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_sidechain_boundary_json(&render_sidechain_boundary_json());
}

#[test]
fn complex_io_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_complex_io_boundary_text(&render_complex_io_boundary_text());
}

#[test]
fn complex_io_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_complex_io_boundary_json(&render_complex_io_boundary_json());
}

#[test]
fn spatial_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_spatial_boundary_text(&render_spatial_boundary_text());
}

#[test]
fn spatial_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_spatial_boundary_json(&render_spatial_boundary_json());
}

#[test]
fn stretch_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_stretch_boundary_text(&render_stretch_boundary_text());
}

#[test]
fn stretch_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_stretch_boundary_json(&render_stretch_boundary_json());
}

#[test]
fn marker_analysis_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_marker_analysis_boundary_text(&render_marker_analysis_boundary_text());
}

#[test]
fn marker_analysis_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_marker_analysis_boundary_json(&render_marker_analysis_boundary_json());
}

#[test]
fn transform_artifact_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_transform_artifact_boundary_text(&render_transform_artifact_boundary_text());
}

#[test]
fn transform_artifact_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_transform_artifact_boundary_json(&render_transform_artifact_boundary_json());
}

#[test]
fn preview_transform_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_preview_transform_boundary_text(&render_preview_transform_boundary_text());
}

#[test]
fn preview_transform_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_preview_transform_boundary_json(&render_preview_transform_boundary_json());
}
