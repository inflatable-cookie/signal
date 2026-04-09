use std::env;

mod acceptance_lanes;
mod descriptor_families;
mod supervisor_cli;
mod supervisor_describe_dispatch;
mod supervisor_export_surface;
#[cfg(test)]
mod supervisor_main_tests;
#[cfg(test)]
mod supervisor_parse_args_tests;
mod supervisor_runtime_runner;
mod supervisor_schema;
#[cfg(test)]
mod supervisor_test_support;

use acceptance_lanes::{
    render_control_preview_workflow_acceptance_lane_json,
    render_control_preview_workflow_acceptance_lane_text,
    render_device_workflow_acceptance_lane_json, render_device_workflow_acceptance_lane_text,
    render_g06_soak_lane_json, render_g06_soak_lane_text, render_g07_acceptance_lane_json,
    render_g07_acceptance_lane_text, render_generation_closeout_json,
    render_generation_closeout_text, render_immersive_acceptance_lane_json,
    render_immersive_acceptance_lane_text, render_integrated_acceptance_lane_json,
    render_integrated_acceptance_lane_text, render_integrated_live_workflow_acceptance_lane_json,
    render_integrated_live_workflow_acceptance_lane_text, render_linux_live_acceptance_lane_json,
    render_linux_live_acceptance_lane_text,
};
use descriptor_families::{
    render_advanced_hardware_boundary_json, render_advanced_hardware_boundary_text,
    render_analysis_metadata_boundary_json, render_analysis_metadata_boundary_text,
    render_au_boundary_json, render_au_boundary_text, render_block_timing_boundary_json,
    render_block_timing_boundary_text, render_clock_topology_boundary_json,
    render_clock_topology_boundary_text, render_complex_io_boundary_json,
    render_complex_io_boundary_text, render_control_surface_boundary_json,
    render_control_surface_boundary_text, render_controller_expression_boundary_json,
    render_controller_expression_boundary_text, render_critical_path_boundary_json,
    render_critical_path_boundary_text, render_cross_adapter_parity_boundary_json,
    render_cross_adapter_parity_boundary_text, render_deferred_work_policy_boundary_json,
    render_deferred_work_policy_boundary_text, render_device_supervision_boundary_json,
    render_device_supervision_boundary_text, render_downstream_automation_json,
    render_downstream_automation_text, render_downstream_fail_gates_json,
    render_downstream_fail_gates_text, render_external_io_boundary_json,
    render_external_io_boundary_text, render_external_midi_boundary_json,
    render_external_midi_boundary_text, render_fault_diagnostic_boundary_json,
    render_fault_diagnostic_boundary_text, render_generic_event_boundary_json,
    render_generic_event_boundary_text, render_host_edge_boundary_json,
    render_host_edge_boundary_text, render_interruption_boundary_json,
    render_interruption_boundary_text, render_jack_coordination_boundary_json,
    render_jack_coordination_boundary_text, render_linux_audio_backend_boundary_json,
    render_linux_audio_backend_boundary_text, render_linux_backend_clock_topology_boundary_json,
    render_linux_backend_clock_topology_boundary_text, render_linux_live_ownership_boundary_json,
    render_linux_live_ownership_boundary_text, render_linux_lv2_execution_boundary_json,
    render_linux_lv2_execution_boundary_text, render_linux_plugin_parity_boundary_json,
    render_linux_plugin_parity_boundary_text, render_lv2_boundary_json, render_lv2_boundary_text,
    render_macos_au_coreaudio_boundary_json, render_macos_au_coreaudio_boundary_text,
    render_marker_analysis_boundary_json, render_marker_analysis_boundary_text,
    render_media_service_boundary_json, render_media_service_boundary_text,
    render_multi_bus_boundary_json, render_multi_bus_boundary_text,
    render_multichannel_boundary_json, render_multichannel_boundary_text,
    render_offline_render_continuity_boundary_json, render_offline_render_continuity_boundary_text,
    render_packaging_manifest_json, render_packaging_manifest_text,
    render_pipewire_alsa_parity_boundary_json, render_pipewire_alsa_parity_boundary_text,
    render_plugin_continuity_boundary_json, render_plugin_continuity_boundary_text,
    render_preview_transform_boundary_json, render_preview_transform_boundary_text,
    render_recall_portability_boundary_json, render_recall_portability_boundary_text,
    render_recording_continuity_boundary_json, render_recording_continuity_boundary_text,
    render_release_boundary_json, render_release_boundary_text, render_sidechain_boundary_json,
    render_sidechain_boundary_text, render_spatial_boundary_json, render_spatial_boundary_text,
    render_stretch_boundary_json, render_stretch_boundary_text,
    render_transform_artifact_boundary_json, render_transform_artifact_boundary_text,
    render_vst3_boundary_json, render_vst3_boundary_text,
};
#[cfg(test)]
pub(crate) use supervisor_cli::CliArgs;
pub(crate) use supervisor_cli::{
    parse_args, print_usage, CliMode, ExportDebugOptions, HostProfile, HostSummaryDebugSection,
    OutputFormat, Scenario,
};
use supervisor_describe_dispatch::print_describe_mode;
pub(crate) use supervisor_export_surface::{
    print_export_description, render_conformance_matrix_json, render_conformance_matrix_text,
    render_local_summary, render_local_summary_json, render_server_summary,
    render_server_summary_json, render_supervisor_export_json,
};
#[cfg(test)]
pub(crate) use supervisor_export_surface::{
    render_export_description_json, render_export_description_text,
};
use supervisor_runtime_runner::{run_local, run_server};
pub(crate) use supervisor_schema::*;

fn json_string(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    format!("\"{escaped}\"")
}

fn json_option_debug<T: std::fmt::Debug>(value: Option<T>) -> String {
    match value {
        Some(value) => json_string(&format!("{value:?}")),
        None => "null".into(),
    }
}

fn main() {
    let args = match parse_args(env::args().skip(1)) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("{message}");
            print_usage();
            std::process::exit(2);
        }
    };

    let result = if print_describe_mode(&args.mode, args.format) {
        Ok(())
    } else {
        match args.mode {
            CliMode::Run { profile, scenario } => match profile {
                HostProfile::Local => run_local(args.format, args.debug, scenario),
                HostProfile::Server => run_server(args.format, args.debug, scenario),
            },
            _ => unreachable!("describe modes are handled by supervisor_describe_dispatch"),
        }
    };

    if let Err(message) = result {
        eprintln!("{message}");
        std::process::exit(1);
    }
}
