use crate::{
    render_advanced_hardware_boundary_json, render_advanced_hardware_boundary_text,
    render_analysis_metadata_boundary_json, render_analysis_metadata_boundary_text,
    render_clock_topology_boundary_json, render_clock_topology_boundary_text,
    render_complex_io_boundary_json, render_complex_io_boundary_text,
    render_control_surface_boundary_json, render_control_surface_boundary_text,
    render_controller_expression_boundary_json, render_controller_expression_boundary_text,
    render_device_supervision_boundary_json, render_device_supervision_boundary_text,
    render_external_io_boundary_json, render_external_io_boundary_text,
    render_generic_event_boundary_json, render_generic_event_boundary_text,
    render_host_edge_boundary_json, render_host_edge_boundary_text,
    render_marker_analysis_boundary_json, render_marker_analysis_boundary_text,
    render_media_service_boundary_json, render_media_service_boundary_text,
    render_multi_bus_boundary_json, render_multi_bus_boundary_text,
    render_multichannel_boundary_json, render_multichannel_boundary_text,
    render_pipewire_alsa_parity_boundary_json, render_pipewire_alsa_parity_boundary_text,
    render_preview_transform_boundary_json, render_preview_transform_boundary_text,
    render_recall_portability_boundary_json, render_recall_portability_boundary_text,
    render_release_boundary_json, render_release_boundary_text, render_sidechain_boundary_json,
    render_sidechain_boundary_text, render_spatial_boundary_json, render_spatial_boundary_text,
    render_stretch_boundary_json, render_stretch_boundary_text,
    render_transform_artifact_boundary_json, render_transform_artifact_boundary_text, CliMode,
    OutputFormat,
};

use super::super::print_surface;

pub(super) fn print_runtime_surface_boundary_mode(mode: &CliMode, format: OutputFormat) -> bool {
    match mode {
        CliMode::DescribePipeWireAlsaParityBoundary => {
            print_surface(
                format,
                render_pipewire_alsa_parity_boundary_text,
                render_pipewire_alsa_parity_boundary_json,
            );
            true
        }
        CliMode::DescribeGenericEventBoundary => {
            print_surface(
                format,
                render_generic_event_boundary_text,
                render_generic_event_boundary_json,
            );
            true
        }
        CliMode::DescribeControllerExpressionBoundary => {
            print_surface(
                format,
                render_controller_expression_boundary_text,
                render_controller_expression_boundary_json,
            );
            true
        }
        CliMode::DescribeControlSurfaceBoundary => {
            print_surface(
                format,
                render_control_surface_boundary_text,
                render_control_surface_boundary_json,
            );
            true
        }
        CliMode::DescribeAdvancedHardwareBoundary => {
            print_surface(
                format,
                render_advanced_hardware_boundary_text,
                render_advanced_hardware_boundary_json,
            );
            true
        }
        CliMode::DescribeRecallPortabilityBoundary => {
            print_surface(
                format,
                render_recall_portability_boundary_text,
                render_recall_portability_boundary_json,
            );
            true
        }
        CliMode::DescribeDeviceSupervisionBoundary => {
            print_surface(
                format,
                render_device_supervision_boundary_text,
                render_device_supervision_boundary_json,
            );
            true
        }
        CliMode::DescribeClockTopologyBoundary => {
            print_surface(
                format,
                render_clock_topology_boundary_text,
                render_clock_topology_boundary_json,
            );
            true
        }
        CliMode::DescribeExternalIoBoundary => {
            print_surface(
                format,
                render_external_io_boundary_text,
                render_external_io_boundary_json,
            );
            true
        }
        CliMode::DescribeMediaServiceBoundary => {
            print_surface(
                format,
                render_media_service_boundary_text,
                render_media_service_boundary_json,
            );
            true
        }
        CliMode::DescribeAnalysisMetadataBoundary => {
            print_surface(
                format,
                render_analysis_metadata_boundary_text,
                render_analysis_metadata_boundary_json,
            );
            true
        }
        CliMode::DescribeMultichannelBoundary => {
            print_surface(
                format,
                render_multichannel_boundary_text,
                render_multichannel_boundary_json,
            );
            true
        }
        CliMode::DescribeMultiBusBoundary => {
            print_surface(
                format,
                render_multi_bus_boundary_text,
                render_multi_bus_boundary_json,
            );
            true
        }
        CliMode::DescribeSidechainBoundary => {
            print_surface(
                format,
                render_sidechain_boundary_text,
                render_sidechain_boundary_json,
            );
            true
        }
        CliMode::DescribeComplexIoBoundary => {
            print_surface(
                format,
                render_complex_io_boundary_text,
                render_complex_io_boundary_json,
            );
            true
        }
        CliMode::DescribeSpatialBoundary => {
            print_surface(
                format,
                render_spatial_boundary_text,
                render_spatial_boundary_json,
            );
            true
        }
        CliMode::DescribeStretchBoundary => {
            print_surface(
                format,
                render_stretch_boundary_text,
                render_stretch_boundary_json,
            );
            true
        }
        CliMode::DescribeMarkerAnalysisBoundary => {
            print_surface(
                format,
                render_marker_analysis_boundary_text,
                render_marker_analysis_boundary_json,
            );
            true
        }
        CliMode::DescribeTransformArtifactBoundary => {
            print_surface(
                format,
                render_transform_artifact_boundary_text,
                render_transform_artifact_boundary_json,
            );
            true
        }
        CliMode::DescribePreviewTransformBoundary => {
            print_surface(
                format,
                render_preview_transform_boundary_text,
                render_preview_transform_boundary_json,
            );
            true
        }
        CliMode::DescribeHostEdgeBoundary => {
            print_surface(
                format,
                render_host_edge_boundary_text,
                render_host_edge_boundary_json,
            );
            true
        }
        CliMode::DescribeReleaseBoundary => {
            print_surface(
                format,
                render_release_boundary_text,
                render_release_boundary_json,
            );
            true
        }
        _ => false,
    }
}
