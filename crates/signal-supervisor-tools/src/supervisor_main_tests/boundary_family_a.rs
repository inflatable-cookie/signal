use crate::{
    render_au_boundary_json, render_au_boundary_text, render_block_timing_boundary_json,
    render_block_timing_boundary_text, render_critical_path_boundary_json,
    render_critical_path_boundary_text, render_cross_adapter_parity_boundary_json,
    render_cross_adapter_parity_boundary_text, render_deferred_work_policy_boundary_json,
    render_deferred_work_policy_boundary_text, render_fault_diagnostic_boundary_json,
    render_fault_diagnostic_boundary_text, render_interruption_boundary_json,
    render_interruption_boundary_text, render_jack_coordination_boundary_json,
    render_jack_coordination_boundary_text, render_linux_audio_backend_boundary_json,
    render_linux_audio_backend_boundary_text, render_linux_backend_clock_topology_boundary_json,
    render_linux_backend_clock_topology_boundary_text, render_linux_live_ownership_boundary_json,
    render_linux_live_ownership_boundary_text, render_linux_plugin_parity_boundary_json,
    render_linux_plugin_parity_boundary_text, render_lv2_boundary_json, render_lv2_boundary_text,
    render_offline_render_continuity_boundary_json, render_offline_render_continuity_boundary_text,
    render_pipewire_alsa_parity_boundary_json, render_pipewire_alsa_parity_boundary_text,
    render_plugin_continuity_boundary_json, render_plugin_continuity_boundary_text,
    render_recording_continuity_boundary_json, render_recording_continuity_boundary_text,
    render_vst3_boundary_json, render_vst3_boundary_text, supervisor_test_support::*,
};

#[test]
fn interruption_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_interruption_boundary_text(&render_interruption_boundary_text());
}

#[test]
fn interruption_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_interruption_boundary_json(&render_interruption_boundary_json());
}

#[test]
fn fault_diagnostic_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_fault_diagnostic_boundary_text(&render_fault_diagnostic_boundary_text());
}

#[test]
fn fault_diagnostic_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_fault_diagnostic_boundary_json(&render_fault_diagnostic_boundary_json());
}

#[test]
fn critical_path_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_critical_path_boundary_text(&render_critical_path_boundary_text());
}

#[test]
fn critical_path_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_critical_path_boundary_json(&render_critical_path_boundary_json());
}

#[test]
fn block_timing_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_block_timing_boundary_text(&render_block_timing_boundary_text());
}

#[test]
fn block_timing_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_block_timing_boundary_json(&render_block_timing_boundary_json());
}

#[test]
fn deferred_work_policy_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_deferred_work_policy_boundary_text(&render_deferred_work_policy_boundary_text());
}

#[test]
fn deferred_work_policy_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_deferred_work_policy_boundary_json(&render_deferred_work_policy_boundary_json());
}

#[test]
fn recording_continuity_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_recording_continuity_boundary_text(&render_recording_continuity_boundary_text());
}

#[test]
fn recording_continuity_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_recording_continuity_boundary_json(&render_recording_continuity_boundary_json());
}

#[test]
fn offline_render_continuity_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_offline_render_continuity_boundary_text(
        &render_offline_render_continuity_boundary_text(),
    );
}

#[test]
fn offline_render_continuity_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_offline_render_continuity_boundary_json(
        &render_offline_render_continuity_boundary_json(),
    );
}

#[test]
fn plugin_continuity_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_plugin_continuity_boundary_text(&render_plugin_continuity_boundary_text());
}

#[test]
fn plugin_continuity_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_plugin_continuity_boundary_json(&render_plugin_continuity_boundary_json());
}

#[test]
fn vst3_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_vst3_boundary_text(&render_vst3_boundary_text());
}

#[test]
fn vst3_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_vst3_boundary_json(&render_vst3_boundary_json());
}

#[test]
fn au_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_au_boundary_text(&render_au_boundary_text());
}

#[test]
fn au_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_au_boundary_json(&render_au_boundary_json());
}

#[test]
fn lv2_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_lv2_boundary_text(&render_lv2_boundary_text());
}

#[test]
fn lv2_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_lv2_boundary_json(&render_lv2_boundary_json());
}

#[test]
fn cross_adapter_parity_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_cross_adapter_parity_boundary_text(&render_cross_adapter_parity_boundary_text());
}

#[test]
fn cross_adapter_parity_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_cross_adapter_parity_boundary_json(&render_cross_adapter_parity_boundary_json());
}

#[test]
fn linux_plugin_parity_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_linux_plugin_parity_boundary_text(&render_linux_plugin_parity_boundary_text());
}

#[test]
fn linux_plugin_parity_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_linux_plugin_parity_boundary_json(&render_linux_plugin_parity_boundary_json());
}

#[test]
fn linux_audio_backend_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_linux_audio_backend_boundary_text(&render_linux_audio_backend_boundary_text());
}

#[test]
fn linux_audio_backend_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_linux_audio_backend_boundary_json(&render_linux_audio_backend_boundary_json());
}

#[test]
fn linux_live_ownership_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_linux_live_ownership_boundary_text(&render_linux_live_ownership_boundary_text());
}

#[test]
fn linux_live_ownership_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_linux_live_ownership_boundary_json(&render_linux_live_ownership_boundary_json());
}

#[test]
fn jack_coordination_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_jack_coordination_boundary_text(&render_jack_coordination_boundary_text());
}

#[test]
fn jack_coordination_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_jack_coordination_boundary_json(&render_jack_coordination_boundary_json());
}

#[test]
fn pipewire_alsa_parity_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_pipewire_alsa_parity_boundary_text(&render_pipewire_alsa_parity_boundary_text());
}

#[test]
fn pipewire_alsa_parity_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_pipewire_alsa_parity_boundary_json(&render_pipewire_alsa_parity_boundary_json());
}

#[test]
fn linux_backend_clock_topology_boundary_text_reports_runtime_and_host_edge_proofs() {
    assert_linux_backend_clock_topology_boundary_text(
        &render_linux_backend_clock_topology_boundary_text(),
    );
}

#[test]
fn linux_backend_clock_topology_boundary_json_reports_runtime_and_host_edge_proofs() {
    assert_linux_backend_clock_topology_boundary_json(
        &render_linux_backend_clock_topology_boundary_json(),
    );
}
