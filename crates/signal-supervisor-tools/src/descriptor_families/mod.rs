use super::*;

mod advanced_hardware;
mod controller_media;
mod downstream;
mod external_midi;
mod host_edge;
mod jack_coordination;
mod linux_audio_backend;
mod linux_backend_clock_topology;
mod linux_live_ownership;
mod packaging;
mod pipewire_alsa;
mod plugin_au;
mod plugin_lv2;
mod plugin_parity;
mod plugin_vst3;
mod preview_transform;
mod release_boundary;
mod routing_media;
mod runtime_continuity;
mod runtime_diagnostics;
mod spatial;
mod transform_artifact;

pub(crate) use advanced_hardware::{
    render_advanced_hardware_boundary_json, render_advanced_hardware_boundary_text,
};
pub(crate) use controller_media::{
    render_control_surface_boundary_json, render_control_surface_boundary_text,
    render_controller_expression_boundary_json, render_controller_expression_boundary_text,
    render_device_supervision_boundary_json, render_device_supervision_boundary_text,
    render_generic_event_boundary_json, render_generic_event_boundary_text,
    render_recall_portability_boundary_json, render_recall_portability_boundary_text,
};
pub(crate) use downstream::{
    render_downstream_automation_json, render_downstream_automation_text,
    render_downstream_fail_gates_json, render_downstream_fail_gates_text,
};
pub(crate) use external_midi::{
    render_external_midi_boundary_json, render_external_midi_boundary_text,
};
pub(crate) use host_edge::{render_host_edge_boundary_json, render_host_edge_boundary_text};
pub(crate) use jack_coordination::{
    render_jack_coordination_boundary_json, render_jack_coordination_boundary_text,
};
pub(crate) use linux_audio_backend::{
    render_linux_audio_backend_boundary_json, render_linux_audio_backend_boundary_text,
};
pub(crate) use linux_backend_clock_topology::{
    render_linux_backend_clock_topology_boundary_json,
    render_linux_backend_clock_topology_boundary_text,
};
pub(crate) use linux_live_ownership::{
    render_linux_live_ownership_boundary_json, render_linux_live_ownership_boundary_text,
};
pub(crate) use packaging::{render_packaging_manifest_json, render_packaging_manifest_text};
pub(crate) use pipewire_alsa::{
    render_pipewire_alsa_parity_boundary_json, render_pipewire_alsa_parity_boundary_text,
};
pub(crate) use plugin_au::{render_au_boundary_json, render_au_boundary_text};
pub(crate) use plugin_lv2::{render_lv2_boundary_json, render_lv2_boundary_text};
pub(crate) use plugin_parity::{
    render_cross_adapter_parity_boundary_json, render_cross_adapter_parity_boundary_text,
    render_linux_plugin_parity_boundary_json, render_linux_plugin_parity_boundary_text,
};
pub(crate) use plugin_vst3::{render_vst3_boundary_json, render_vst3_boundary_text};
pub(crate) use preview_transform::{
    render_preview_transform_boundary_json, render_preview_transform_boundary_text,
};
pub(crate) use release_boundary::{render_release_boundary_json, render_release_boundary_text};
pub(crate) use routing_media::{
    render_analysis_metadata_boundary_json, render_analysis_metadata_boundary_text,
    render_clock_topology_boundary_json, render_clock_topology_boundary_text,
    render_complex_io_boundary_json, render_complex_io_boundary_text,
    render_external_io_boundary_json, render_external_io_boundary_text,
    render_media_service_boundary_json, render_media_service_boundary_text,
    render_multi_bus_boundary_json, render_multi_bus_boundary_text,
    render_multichannel_boundary_json, render_multichannel_boundary_text,
    render_sidechain_boundary_json, render_sidechain_boundary_text,
};
pub(crate) use runtime_continuity::{
    render_interruption_boundary_json, render_interruption_boundary_text,
    render_offline_render_continuity_boundary_json, render_offline_render_continuity_boundary_text,
    render_plugin_continuity_boundary_json, render_plugin_continuity_boundary_text,
    render_recording_continuity_boundary_json, render_recording_continuity_boundary_text,
};
pub(crate) use runtime_diagnostics::{
    render_block_timing_boundary_json, render_block_timing_boundary_text,
    render_critical_path_boundary_json, render_critical_path_boundary_text,
    render_deferred_work_policy_boundary_json, render_deferred_work_policy_boundary_text,
    render_fault_diagnostic_boundary_json, render_fault_diagnostic_boundary_text,
};
pub(crate) use spatial::{render_spatial_boundary_json, render_spatial_boundary_text};
pub(crate) use transform_artifact::{
    render_transform_artifact_boundary_json, render_transform_artifact_boundary_text,
};
