use std::env;

mod acceptance_lanes;
mod descriptor_families;
mod supervisor_cli;
mod supervisor_describe_dispatch;
mod supervisor_export_surface;
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
    render_linux_live_ownership_boundary_text, render_linux_plugin_parity_boundary_json,
    render_linux_plugin_parity_boundary_text, render_lv2_boundary_json, render_lv2_boundary_text,
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

#[cfg(test)]
mod tests {
    use super::{
        parse_args, render_advanced_hardware_boundary_json, render_advanced_hardware_boundary_text,
        render_analysis_metadata_boundary_json, render_analysis_metadata_boundary_text,
        render_au_boundary_json, render_au_boundary_text, render_block_timing_boundary_json,
        render_block_timing_boundary_text, render_clock_topology_boundary_json,
        render_clock_topology_boundary_text, render_complex_io_boundary_json,
        render_complex_io_boundary_text, render_conformance_matrix_json,
        render_conformance_matrix_text, render_control_preview_workflow_acceptance_lane_json,
        render_control_preview_workflow_acceptance_lane_text, render_control_surface_boundary_json,
        render_control_surface_boundary_text, render_controller_expression_boundary_json,
        render_controller_expression_boundary_text, render_critical_path_boundary_json,
        render_critical_path_boundary_text, render_cross_adapter_parity_boundary_json,
        render_cross_adapter_parity_boundary_text, render_deferred_work_policy_boundary_json,
        render_deferred_work_policy_boundary_text, render_device_supervision_boundary_json,
        render_device_supervision_boundary_text, render_device_workflow_acceptance_lane_json,
        render_device_workflow_acceptance_lane_text, render_export_description_json,
        render_export_description_text, render_external_io_boundary_json,
        render_external_io_boundary_text, render_external_midi_boundary_json,
        render_external_midi_boundary_text, render_fault_diagnostic_boundary_json,
        render_fault_diagnostic_boundary_text, render_g07_acceptance_lane_json,
        render_g07_acceptance_lane_text, render_generic_event_boundary_json,
        render_generic_event_boundary_text, render_immersive_acceptance_lane_json,
        render_immersive_acceptance_lane_text, render_integrated_acceptance_lane_json,
        render_integrated_acceptance_lane_text,
        render_integrated_live_workflow_acceptance_lane_json,
        render_integrated_live_workflow_acceptance_lane_text, render_interruption_boundary_json,
        render_interruption_boundary_text, render_jack_coordination_boundary_json,
        render_jack_coordination_boundary_text, render_linux_audio_backend_boundary_json,
        render_linux_audio_backend_boundary_text,
        render_linux_backend_clock_topology_boundary_json,
        render_linux_backend_clock_topology_boundary_text, render_linux_live_acceptance_lane_json,
        render_linux_live_acceptance_lane_text, render_linux_live_ownership_boundary_json,
        render_linux_live_ownership_boundary_text, render_linux_plugin_parity_boundary_json,
        render_linux_plugin_parity_boundary_text, render_lv2_boundary_json,
        render_lv2_boundary_text, render_marker_analysis_boundary_json,
        render_marker_analysis_boundary_text, render_media_service_boundary_json,
        render_media_service_boundary_text, render_multi_bus_boundary_json,
        render_multi_bus_boundary_text, render_multichannel_boundary_json,
        render_multichannel_boundary_text, render_offline_render_continuity_boundary_json,
        render_offline_render_continuity_boundary_text, render_pipewire_alsa_parity_boundary_json,
        render_pipewire_alsa_parity_boundary_text, render_plugin_continuity_boundary_json,
        render_plugin_continuity_boundary_text, render_preview_transform_boundary_json,
        render_preview_transform_boundary_text, render_recall_portability_boundary_json,
        render_recall_portability_boundary_text, render_recording_continuity_boundary_json,
        render_recording_continuity_boundary_text, render_sidechain_boundary_json,
        render_sidechain_boundary_text, render_spatial_boundary_json, render_spatial_boundary_text,
        render_stretch_boundary_json, render_stretch_boundary_text,
        render_transform_artifact_boundary_json, render_transform_artifact_boundary_text,
        render_vst3_boundary_json, render_vst3_boundary_text, supervisor_test_support::*, CliArgs,
        CliMode, ExportDebugOptions, HostProfile, HostSummaryDebugSection, OutputFormat, Scenario,
    };
    #[test]
    fn parses_profiles() {
        assert_eq!(HostProfile::parse("local"), Ok(HostProfile::Local));
        assert_eq!(HostProfile::parse("server"), Ok(HostProfile::Server));
    }

    #[test]
    fn parses_scenarios() {
        assert_eq!(Scenario::parse("default"), Ok(Scenario::Default));
        assert_eq!(Scenario::parse("mixed"), Ok(Scenario::Mixed));
        assert_eq!(Scenario::parse("soak"), Ok(Scenario::Soak));
    }

    #[test]
    fn parses_json_flag_and_positionals() {
        assert_eq!(
            parse_args(["--format=json".into(), "local".into(), "mixed".into(),]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::Run {
                    profile: HostProfile::Local,
                    scenario: Scenario::Mixed,
                },
            })
        );
    }

    #[test]
    fn rejects_missing_positionals() {
        let error = parse_args(["local".into()]).unwrap_err();
        assert!(error.contains("expected"));
    }

    #[test]
    fn only_payload_is_currently_supported_as_debug_section() {
        assert!(ExportDebugOptions { payload: true }.supports(HostSummaryDebugSection::Payload));
        assert_eq!(HostSummaryDebugSection::Payload.label(), "payload");
    }

    #[test]
    fn export_description_text_reports_frozen_policy() {
        let rendered = render_export_description_text();
        assert!(rendered.contains("schema: signal.supervisor.export"));
        assert!(rendered.contains("schema_version: 1"));
        assert!(rendered.contains("default_host_summary_sections: execution,transport,faults"));
        assert!(rendered.contains("supported_debug_sections: payload"));
    }

    #[test]
    fn export_description_json_reports_frozen_policy() {
        let rendered = render_export_description_json();
        assert!(rendered.contains("\"schema\":\"signal.supervisor.export\""));
        assert!(rendered.contains("\"schema_version\":1"));
        assert!(rendered.contains(
            "\"default_host_summary_sections\":[\"execution\",\"transport\",\"faults\"]"
        ));
        assert!(rendered.contains("\"supported_debug_sections\":[\"payload\"]"));
    }

    #[test]
    fn conformance_matrix_text_reports_runnable_consumer_boundary() {
        let rendered = render_conformance_matrix_text();
        assert!(rendered.contains("consumer_conformance_matrix:"));
        assert!(rendered.contains("runtime-public-contract-boundary"));
        assert!(rendered.contains("supervisor-export-discovery-consumer"));
        assert!(rendered.contains("plugin-backend-breadth-coverage"));
        assert!(rendered.contains("shared-host-edge-consumer"));
        assert!(rendered.contains("runtime-supervisor-report-demo"));
        assert!(rendered.contains("supervisor-export-schema-description"));
        assert!(rendered.contains("cargo test -p signal-runtime public_runtime_contract_boundary_is_consumable_from_reexports"));
        assert!(rendered.contains("effigy acceptance:plugin-backend-breadth"));
        assert!(rendered.contains("effigy acceptance:host-edge-consumer"));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-conformance-matrix --format=json"
        ));
    }

    #[test]
    fn conformance_matrix_json_reports_runnable_consumer_boundary() {
        let rendered = render_conformance_matrix_json();
        assert!(rendered.contains("\"matrix\":\"signal.consumer.conformance\""));
        assert!(rendered.contains("\"entry_count\":6"));
        assert!(rendered.contains("\"id\":\"runtime-public-contract-boundary\""));
        assert!(rendered.contains("\"kind\":\"export-consumer-test\""));
        assert!(rendered.contains("\"crate\":\"signal-supervisor-tools\""));
        assert!(rendered.contains("\"id\":\"plugin-backend-breadth-coverage\""));
        assert!(rendered.contains("\"id\":\"shared-host-edge-consumer\""));
        assert!(rendered.contains(
            "\"command\":\"cargo run -p signal-runtime --example supervisor_report_demo\""
        ));
    }

    #[test]
    fn interruption_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_interruption_boundary_text();
        assert_interruption_boundary_text(&rendered);
    }

    #[test]
    fn interruption_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_interruption_boundary_json();
        assert_interruption_boundary_json(&rendered);
    }

    #[test]
    fn fault_diagnostic_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_fault_diagnostic_boundary_text();
        assert_fault_diagnostic_boundary_text(&rendered);
    }

    #[test]
    fn fault_diagnostic_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_fault_diagnostic_boundary_json();
        assert_fault_diagnostic_boundary_json(&rendered);
    }

    #[test]
    fn critical_path_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_critical_path_boundary_text();
        assert_critical_path_boundary_text(&rendered);
    }

    #[test]
    fn critical_path_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_critical_path_boundary_json();
        assert_critical_path_boundary_json(&rendered);
    }

    #[test]
    fn block_timing_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_block_timing_boundary_text();
        assert_block_timing_boundary_text(&rendered);
    }

    #[test]
    fn block_timing_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_block_timing_boundary_json();
        assert_block_timing_boundary_json(&rendered);
    }

    #[test]
    fn deferred_work_policy_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_deferred_work_policy_boundary_text();
        assert_deferred_work_policy_boundary_text(&rendered);
    }

    #[test]
    fn deferred_work_policy_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_deferred_work_policy_boundary_json();
        assert_deferred_work_policy_boundary_json(&rendered);
    }

    #[test]
    fn recording_continuity_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_recording_continuity_boundary_text();
        assert_recording_continuity_boundary_text(&rendered);
    }

    #[test]
    fn recording_continuity_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_recording_continuity_boundary_json();
        assert_recording_continuity_boundary_json(&rendered);
    }

    #[test]
    fn offline_render_continuity_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_offline_render_continuity_boundary_text();
        assert_offline_render_continuity_boundary_text(&rendered);
    }

    #[test]
    fn offline_render_continuity_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_offline_render_continuity_boundary_json();
        assert_offline_render_continuity_boundary_json(&rendered);
    }

    #[test]
    fn plugin_continuity_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_plugin_continuity_boundary_text();
        assert_plugin_continuity_boundary_text(&rendered);
    }

    #[test]
    fn plugin_continuity_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_plugin_continuity_boundary_json();
        assert_plugin_continuity_boundary_json(&rendered);
    }

    #[test]
    fn vst3_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_vst3_boundary_text();
        assert_vst3_boundary_text(&rendered);
    }

    #[test]
    fn vst3_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_vst3_boundary_json();
        assert_vst3_boundary_json(&rendered);
    }

    #[test]
    fn au_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_au_boundary_text();
        assert_au_boundary_text(&rendered);
    }

    #[test]
    fn au_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_au_boundary_json();
        assert_au_boundary_json(&rendered);
    }

    #[test]
    fn lv2_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_lv2_boundary_text();
        assert_lv2_boundary_text(&rendered);
    }

    #[test]
    fn lv2_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_lv2_boundary_json();
        assert_lv2_boundary_json(&rendered);
    }

    #[test]
    fn cross_adapter_parity_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_cross_adapter_parity_boundary_text();
        assert_cross_adapter_parity_boundary_text(&rendered);
    }

    #[test]
    fn cross_adapter_parity_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_cross_adapter_parity_boundary_json();
        assert_cross_adapter_parity_boundary_json(&rendered);
    }

    #[test]
    fn linux_plugin_parity_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_linux_plugin_parity_boundary_text();
        assert_linux_plugin_parity_boundary_text(&rendered);
    }

    #[test]
    fn linux_plugin_parity_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_linux_plugin_parity_boundary_json();
        assert_linux_plugin_parity_boundary_json(&rendered);
    }

    #[test]
    fn linux_audio_backend_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_linux_audio_backend_boundary_text();
        assert_linux_audio_backend_boundary_text(&rendered);
    }

    #[test]
    fn linux_audio_backend_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_linux_audio_backend_boundary_json();
        assert_linux_audio_backend_boundary_json(&rendered);
    }

    #[test]
    fn linux_live_ownership_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_linux_live_ownership_boundary_text();
        assert_linux_live_ownership_boundary_text(&rendered);
    }

    #[test]
    fn linux_live_ownership_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_linux_live_ownership_boundary_json();
        assert_linux_live_ownership_boundary_json(&rendered);
    }

    #[test]
    fn jack_coordination_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_jack_coordination_boundary_text();
        assert_jack_coordination_boundary_text(&rendered);
    }

    #[test]
    fn jack_coordination_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_jack_coordination_boundary_json();
        assert_jack_coordination_boundary_json(&rendered);
    }

    #[test]
    fn pipewire_alsa_parity_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_pipewire_alsa_parity_boundary_text();
        assert_pipewire_alsa_parity_boundary_text(&rendered);
    }

    #[test]
    fn pipewire_alsa_parity_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_pipewire_alsa_parity_boundary_json();
        assert_pipewire_alsa_parity_boundary_json(&rendered);
    }

    #[test]
    fn linux_backend_clock_topology_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_linux_backend_clock_topology_boundary_text();
        assert_linux_backend_clock_topology_boundary_text(&rendered);
    }

    #[test]
    fn linux_backend_clock_topology_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_linux_backend_clock_topology_boundary_json();
        assert_linux_backend_clock_topology_boundary_json(&rendered);
    }

    #[test]
    fn external_midi_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_external_midi_boundary_text();
        assert_external_midi_boundary_text(&rendered);
    }

    #[test]
    fn external_midi_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_external_midi_boundary_json();
        assert_external_midi_boundary_json(&rendered);
    }

    #[test]
    fn generic_event_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_generic_event_boundary_text();
        assert_generic_event_boundary_text(&rendered);
    }

    #[test]
    fn generic_event_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_generic_event_boundary_json();
        assert_generic_event_boundary_json(&rendered);
    }

    #[test]
    fn controller_expression_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_controller_expression_boundary_text();
        assert_controller_expression_boundary_text(&rendered);
    }

    #[test]
    fn controller_expression_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_controller_expression_boundary_json();
        assert_controller_expression_boundary_json(&rendered);
    }

    #[test]
    fn control_surface_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_control_surface_boundary_text();
        assert_control_surface_boundary_text(&rendered);
    }

    #[test]
    fn control_surface_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_control_surface_boundary_json();
        assert_control_surface_boundary_json(&rendered);
    }

    #[test]
    fn advanced_hardware_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_advanced_hardware_boundary_text();
        assert_advanced_hardware_boundary_text(&rendered);
    }

    #[test]
    fn advanced_hardware_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_advanced_hardware_boundary_json();
        assert_advanced_hardware_boundary_json(&rendered);
    }

    #[test]
    fn recall_portability_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_recall_portability_boundary_text();
        assert_recall_portability_boundary_text(&rendered);
    }

    #[test]
    fn recall_portability_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_recall_portability_boundary_json();
        assert_recall_portability_boundary_json(&rendered);
    }

    #[test]
    fn device_supervision_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_device_supervision_boundary_text();
        assert_device_supervision_boundary_text(&rendered);
    }

    #[test]
    fn device_supervision_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_device_supervision_boundary_json();
        assert_device_supervision_boundary_json(&rendered);
    }

    #[test]
    fn clock_topology_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_clock_topology_boundary_text();
        assert_clock_topology_boundary_text(&rendered);
    }

    #[test]
    fn clock_topology_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_clock_topology_boundary_json();
        assert_clock_topology_boundary_json(&rendered);
    }

    #[test]
    fn external_io_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_external_io_boundary_text();
        assert_external_io_boundary_text(&rendered);
    }

    #[test]
    fn external_io_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_external_io_boundary_json();
        assert_external_io_boundary_json(&rendered);
    }

    #[test]
    fn media_service_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_media_service_boundary_text();
        assert_media_service_boundary_text(&rendered);
    }

    #[test]
    fn media_service_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_media_service_boundary_json();
        assert_media_service_boundary_json(&rendered);
    }

    #[test]
    fn analysis_metadata_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_analysis_metadata_boundary_text();
        assert_analysis_metadata_boundary_text(&rendered);
    }

    #[test]
    fn analysis_metadata_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_analysis_metadata_boundary_json();
        assert_analysis_metadata_boundary_json(&rendered);
    }

    #[test]
    fn multichannel_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_multichannel_boundary_text();
        assert_multichannel_boundary_text(&rendered);
    }

    #[test]
    fn multichannel_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_multichannel_boundary_json();
        assert_multichannel_boundary_json(&rendered);
    }

    #[test]
    fn multi_bus_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_multi_bus_boundary_text();
        assert_multi_bus_boundary_text(&rendered);
    }

    #[test]
    fn multi_bus_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_multi_bus_boundary_json();
        assert_multi_bus_boundary_json(&rendered);
    }

    #[test]
    fn sidechain_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_sidechain_boundary_text();
        assert_sidechain_boundary_text(&rendered);
    }

    #[test]
    fn sidechain_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_sidechain_boundary_json();
        assert_sidechain_boundary_json(&rendered);
    }

    #[test]
    fn complex_io_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_complex_io_boundary_text();
        assert_complex_io_boundary_text(&rendered);
    }

    #[test]
    fn complex_io_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_complex_io_boundary_json();
        assert_complex_io_boundary_json(&rendered);
    }

    #[test]
    fn spatial_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_spatial_boundary_text();
        assert_spatial_boundary_text(&rendered);
    }

    #[test]
    fn spatial_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_spatial_boundary_json();
        assert_spatial_boundary_json(&rendered);
    }

    #[test]
    fn stretch_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_stretch_boundary_text();
        assert_stretch_boundary_text(&rendered);
    }

    #[test]
    fn stretch_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_stretch_boundary_json();
        assert_stretch_boundary_json(&rendered);
    }

    #[test]
    fn marker_analysis_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_marker_analysis_boundary_text();
        assert_marker_analysis_boundary_text(&rendered);
    }

    #[test]
    fn marker_analysis_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_marker_analysis_boundary_json();
        assert_marker_analysis_boundary_json(&rendered);
    }

    #[test]
    fn transform_artifact_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_transform_artifact_boundary_text();
        assert_transform_artifact_boundary_text(&rendered);
    }

    #[test]
    fn transform_artifact_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_transform_artifact_boundary_json();
        assert_transform_artifact_boundary_json(&rendered);
    }

    #[test]
    fn preview_transform_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_preview_transform_boundary_text();
        assert_preview_transform_boundary_text(&rendered);
    }

    #[test]
    fn preview_transform_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_preview_transform_boundary_json();
        assert_preview_transform_boundary_json(&rendered);
    }

    #[test]
    fn integrated_acceptance_lane_text_reports_required_and_advisory_policy() {
        let rendered = render_integrated_acceptance_lane_text();
        assert_integrated_acceptance_lane_text(&rendered);
    }

    #[test]
    fn integrated_acceptance_lane_json_reports_required_and_advisory_policy() {
        let rendered = render_integrated_acceptance_lane_json();
        assert_integrated_acceptance_lane_json(&rendered);
    }

    #[test]
    fn g07_acceptance_lane_text_reports_required_and_advisory_policy() {
        let rendered = render_g07_acceptance_lane_text();
        assert_g07_acceptance_lane_text(&rendered);
    }

    #[test]
    fn g07_acceptance_lane_json_reports_required_and_advisory_policy() {
        let rendered = render_g07_acceptance_lane_json();
        assert_g07_acceptance_lane_json(&rendered);
    }

    #[test]
    fn device_workflow_acceptance_lane_text_reports_required_and_deferred_policy() {
        let rendered = render_device_workflow_acceptance_lane_text();
        assert_device_workflow_acceptance_lane_text(&rendered);
    }

    #[test]
    fn device_workflow_acceptance_lane_json_reports_required_and_deferred_policy() {
        let rendered = render_device_workflow_acceptance_lane_json();
        assert_device_workflow_acceptance_lane_json(&rendered);
    }

    #[test]
    fn linux_live_acceptance_lane_text_reports_required_and_deferred_policy() {
        let rendered = render_linux_live_acceptance_lane_text();
        assert_linux_live_acceptance_lane_text(&rendered);
    }

    #[test]
    fn linux_live_acceptance_lane_json_reports_required_and_deferred_policy() {
        let rendered = render_linux_live_acceptance_lane_json();
        assert_linux_live_acceptance_lane_json(&rendered);
    }

    #[test]
    fn immersive_acceptance_lane_text_reports_required_and_deferred_policy() {
        let rendered = render_immersive_acceptance_lane_text();
        assert_immersive_acceptance_lane_text(&rendered);
    }

    #[test]
    fn immersive_acceptance_lane_json_reports_required_and_deferred_policy() {
        let rendered = render_immersive_acceptance_lane_json();
        assert_immersive_acceptance_lane_json(&rendered);
    }

    #[test]
    fn control_preview_workflow_acceptance_lane_text_reports_required_and_deferred_policy() {
        let rendered = render_control_preview_workflow_acceptance_lane_text();
        assert_control_preview_workflow_acceptance_lane_text(&rendered);
    }

    #[test]
    fn control_preview_workflow_acceptance_lane_json_reports_required_and_deferred_policy() {
        let rendered = render_control_preview_workflow_acceptance_lane_json();
        assert_control_preview_workflow_acceptance_lane_json(&rendered);
    }

    #[test]
    fn integrated_live_workflow_acceptance_lane_text_reports_required_and_deferred_policy() {
        let rendered = render_integrated_live_workflow_acceptance_lane_text();
        assert_integrated_live_workflow_acceptance_lane_text(&rendered);
    }

    #[test]
    fn integrated_live_workflow_acceptance_lane_json_reports_required_and_deferred_policy() {
        let rendered = render_integrated_live_workflow_acceptance_lane_json();
        assert_integrated_live_workflow_acceptance_lane_json(&rendered);
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
}
