use crate::{
    print_export_description, render_conformance_matrix_json, render_conformance_matrix_text,
    render_control_preview_workflow_acceptance_lane_json,
    render_control_preview_workflow_acceptance_lane_text,
    render_device_workflow_acceptance_lane_json, render_device_workflow_acceptance_lane_text,
    render_downstream_automation_json, render_downstream_automation_text,
    render_downstream_fail_gates_json, render_downstream_fail_gates_text,
    render_g06_soak_lane_json, render_g06_soak_lane_text, render_g07_acceptance_lane_json,
    render_g07_acceptance_lane_text, render_generation_closeout_json,
    render_generation_closeout_text, render_immersive_acceptance_lane_json,
    render_immersive_acceptance_lane_text, render_integrated_acceptance_lane_json,
    render_integrated_acceptance_lane_text, render_integrated_live_workflow_acceptance_lane_json,
    render_integrated_live_workflow_acceptance_lane_text, render_linux_live_acceptance_lane_json,
    render_linux_live_acceptance_lane_text, render_packaging_manifest_json,
    render_packaging_manifest_text, CliMode, OutputFormat,
};

use super::print_surface;

pub(super) fn print_lane_describe_mode(mode: &CliMode, format: OutputFormat) -> bool {
    match mode {
        CliMode::DescribeExport => {
            print_export_description(format);
            true
        }
        CliMode::DescribeConformanceMatrix => {
            print_surface(
                format,
                render_conformance_matrix_text,
                render_conformance_matrix_json,
            );
            true
        }
        CliMode::DescribeIntegratedAcceptanceLane => {
            print_surface(
                format,
                render_integrated_acceptance_lane_text,
                render_integrated_acceptance_lane_json,
            );
            true
        }
        CliMode::DescribeG07AcceptanceLane => {
            print_surface(
                format,
                render_g07_acceptance_lane_text,
                render_g07_acceptance_lane_json,
            );
            true
        }
        CliMode::DescribeDeviceWorkflowAcceptanceLane => {
            print_surface(
                format,
                render_device_workflow_acceptance_lane_text,
                render_device_workflow_acceptance_lane_json,
            );
            true
        }
        CliMode::DescribeLinuxLiveAcceptanceLane => {
            print_surface(
                format,
                render_linux_live_acceptance_lane_text,
                render_linux_live_acceptance_lane_json,
            );
            true
        }
        CliMode::DescribeImmersiveAcceptanceLane => {
            print_surface(
                format,
                render_immersive_acceptance_lane_text,
                render_immersive_acceptance_lane_json,
            );
            true
        }
        CliMode::DescribeControlPreviewWorkflowAcceptanceLane => {
            print_surface(
                format,
                render_control_preview_workflow_acceptance_lane_text,
                render_control_preview_workflow_acceptance_lane_json,
            );
            true
        }
        CliMode::DescribeIntegratedLiveWorkflowAcceptanceLane => {
            print_surface(
                format,
                render_integrated_live_workflow_acceptance_lane_text,
                render_integrated_live_workflow_acceptance_lane_json,
            );
            true
        }
        CliMode::DescribeG06SoakLane => {
            print_surface(format, render_g06_soak_lane_text, render_g06_soak_lane_json);
            true
        }
        CliMode::DescribePackagingManifest => {
            print_surface(
                format,
                render_packaging_manifest_text,
                render_packaging_manifest_json,
            );
            true
        }
        CliMode::DescribeDownstreamAutomation => {
            print_surface(
                format,
                render_downstream_automation_text,
                render_downstream_automation_json,
            );
            true
        }
        CliMode::DescribeDownstreamFailGates => {
            print_surface(
                format,
                render_downstream_fail_gates_text,
                render_downstream_fail_gates_json,
            );
            true
        }
        CliMode::DescribeGenerationCloseout => {
            print_surface(
                format,
                render_generation_closeout_text,
                render_generation_closeout_json,
            );
            true
        }
        _ => false,
    }
}
