use crate::{
    render_au_boundary_json, render_au_boundary_text, render_block_timing_boundary_json,
    render_block_timing_boundary_text, render_critical_path_boundary_json,
    render_critical_path_boundary_text, render_cross_adapter_parity_boundary_json,
    render_cross_adapter_parity_boundary_text, render_deferred_work_policy_boundary_json,
    render_deferred_work_policy_boundary_text, render_external_midi_boundary_json,
    render_external_midi_boundary_text, render_fault_diagnostic_boundary_json,
    render_fault_diagnostic_boundary_text, render_interruption_boundary_json,
    render_interruption_boundary_text, render_jack_coordination_boundary_json,
    render_jack_coordination_boundary_text, render_linux_audio_backend_boundary_json,
    render_linux_audio_backend_boundary_text, render_linux_backend_clock_topology_boundary_json,
    render_linux_backend_clock_topology_boundary_text, render_linux_live_ownership_boundary_json,
    render_linux_live_ownership_boundary_text, render_linux_lv2_execution_boundary_json,
    render_linux_lv2_execution_boundary_text, render_linux_plugin_parity_boundary_json,
    render_linux_plugin_parity_boundary_text, render_lv2_boundary_json, render_lv2_boundary_text,
    render_macos_au_coreaudio_boundary_json, render_macos_au_coreaudio_boundary_text,
    render_offline_render_continuity_boundary_json, render_offline_render_continuity_boundary_text,
    render_plugin_continuity_boundary_json, render_plugin_continuity_boundary_text,
    render_recording_continuity_boundary_json, render_recording_continuity_boundary_text,
    render_vst3_boundary_json, render_vst3_boundary_text, CliMode, OutputFormat,
};

use super::super::print_surface;

pub(super) fn print_runtime_core_boundary_mode(mode: &CliMode, format: OutputFormat) -> bool {
    match mode {
        CliMode::DescribeInterruptionBoundary => {
            print_surface(
                format,
                render_interruption_boundary_text,
                render_interruption_boundary_json,
            );
            true
        }
        CliMode::DescribeFaultDiagnosticBoundary => {
            print_surface(
                format,
                render_fault_diagnostic_boundary_text,
                render_fault_diagnostic_boundary_json,
            );
            true
        }
        CliMode::DescribeCriticalPathBoundary => {
            print_surface(
                format,
                render_critical_path_boundary_text,
                render_critical_path_boundary_json,
            );
            true
        }
        CliMode::DescribeBlockTimingBoundary => {
            print_surface(
                format,
                render_block_timing_boundary_text,
                render_block_timing_boundary_json,
            );
            true
        }
        CliMode::DescribeDeferredWorkPolicyBoundary => {
            print_surface(
                format,
                render_deferred_work_policy_boundary_text,
                render_deferred_work_policy_boundary_json,
            );
            true
        }
        CliMode::DescribeRecordingContinuityBoundary => {
            print_surface(
                format,
                render_recording_continuity_boundary_text,
                render_recording_continuity_boundary_json,
            );
            true
        }
        CliMode::DescribeOfflineRenderContinuityBoundary => {
            print_surface(
                format,
                render_offline_render_continuity_boundary_text,
                render_offline_render_continuity_boundary_json,
            );
            true
        }
        CliMode::DescribePluginContinuityBoundary => {
            print_surface(
                format,
                render_plugin_continuity_boundary_text,
                render_plugin_continuity_boundary_json,
            );
            true
        }
        CliMode::DescribeVst3Boundary => {
            print_surface(format, render_vst3_boundary_text, render_vst3_boundary_json);
            true
        }
        CliMode::DescribeAuBoundary => {
            print_surface(format, render_au_boundary_text, render_au_boundary_json);
            true
        }
        CliMode::DescribeMacosAuCoreaudioBoundary => {
            print_surface(
                format,
                render_macos_au_coreaudio_boundary_text,
                render_macos_au_coreaudio_boundary_json,
            );
            true
        }
        CliMode::DescribeLv2Boundary => {
            print_surface(format, render_lv2_boundary_text, render_lv2_boundary_json);
            true
        }
        CliMode::DescribeLinuxLv2ExecutionBoundary => {
            print_surface(
                format,
                render_linux_lv2_execution_boundary_text,
                render_linux_lv2_execution_boundary_json,
            );
            true
        }
        CliMode::DescribeCrossAdapterParityBoundary => {
            print_surface(
                format,
                render_cross_adapter_parity_boundary_text,
                render_cross_adapter_parity_boundary_json,
            );
            true
        }
        CliMode::DescribeLinuxPluginParityBoundary => {
            print_surface(
                format,
                render_linux_plugin_parity_boundary_text,
                render_linux_plugin_parity_boundary_json,
            );
            true
        }
        CliMode::DescribeLinuxAudioBackendBoundary => {
            print_surface(
                format,
                render_linux_audio_backend_boundary_text,
                render_linux_audio_backend_boundary_json,
            );
            true
        }
        CliMode::DescribeLinuxLiveOwnershipBoundary => {
            print_surface(
                format,
                render_linux_live_ownership_boundary_text,
                render_linux_live_ownership_boundary_json,
            );
            true
        }
        CliMode::DescribeJackCoordinationBoundary => {
            print_surface(
                format,
                render_jack_coordination_boundary_text,
                render_jack_coordination_boundary_json,
            );
            true
        }
        CliMode::DescribeLinuxBackendClockTopologyBoundary => {
            print_surface(
                format,
                render_linux_backend_clock_topology_boundary_text,
                render_linux_backend_clock_topology_boundary_json,
            );
            true
        }
        CliMode::DescribeExternalMidiBoundary => {
            print_surface(
                format,
                render_external_midi_boundary_text,
                render_external_midi_boundary_json,
            );
            true
        }
        _ => false,
    }
}
