use std::env;

mod acceptance_lanes;
mod descriptor_families;
mod supervisor_describe_dispatch;
mod supervisor_export_surface;

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
use signal_host_local::LocalRuntimeHost;
use signal_host_server::ServerRuntimeHost;
use signal_runtime::{
    RuntimeConfig, RuntimeProfilingReceipt, RuntimeSoakReceipt, RuntimeSupervisorReport,
    SignalRuntime,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostProfile {
    Local,
    Server,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Scenario {
    Default,
    Timeout,
    Crash,
    Heartbeat,
    Soak,
    Mixed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostSummaryDebugSection {
    Payload,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum CliMode {
    Run {
        profile: HostProfile,
        scenario: Scenario,
    },
    DescribeExport,
    DescribeConformanceMatrix,
    DescribeInterruptionBoundary,
    DescribeFaultDiagnosticBoundary,
    DescribeCriticalPathBoundary,
    DescribeBlockTimingBoundary,
    DescribeDeferredWorkPolicyBoundary,
    DescribeRecordingContinuityBoundary,
    DescribeOfflineRenderContinuityBoundary,
    DescribePluginContinuityBoundary,
    DescribeVst3Boundary,
    DescribeAuBoundary,
    DescribeLv2Boundary,
    DescribeCrossAdapterParityBoundary,
    DescribeLinuxPluginParityBoundary,
    DescribeLinuxAudioBackendBoundary,
    DescribeLinuxLiveOwnershipBoundary,
    DescribeJackCoordinationBoundary,
    DescribePipeWireAlsaParityBoundary,
    DescribeLinuxBackendClockTopologyBoundary,
    DescribeExternalMidiBoundary,
    DescribeGenericEventBoundary,
    DescribeControllerExpressionBoundary,
    DescribeControlSurfaceBoundary,
    DescribeAdvancedHardwareBoundary,
    DescribeRecallPortabilityBoundary,
    DescribeDeviceSupervisionBoundary,
    DescribeClockTopologyBoundary,
    DescribeExternalIoBoundary,
    DescribeMediaServiceBoundary,
    DescribeAnalysisMetadataBoundary,
    DescribeMultichannelBoundary,
    DescribeMultiBusBoundary,
    DescribeSidechainBoundary,
    DescribeComplexIoBoundary,
    DescribeSpatialBoundary,
    DescribeStretchBoundary,
    DescribeMarkerAnalysisBoundary,
    DescribeTransformArtifactBoundary,
    DescribePreviewTransformBoundary,
    DescribeIntegratedAcceptanceLane,
    DescribeG07AcceptanceLane,
    DescribeDeviceWorkflowAcceptanceLane,
    DescribeLinuxLiveAcceptanceLane,
    DescribeImmersiveAcceptanceLane,
    DescribeControlPreviewWorkflowAcceptanceLane,
    DescribeIntegratedLiveWorkflowAcceptanceLane,
    DescribeG06SoakLane,
    DescribeHostEdgeBoundary,
    DescribeReleaseBoundary,
    DescribePackagingManifest,
    DescribeDownstreamAutomation,
    DescribeDownstreamFailGates,
    DescribeGenerationCloseout,
}

const EXPORT_SCHEMA: &str = "signal.supervisor.export";
const EXPORT_SCHEMA_VERSION: u32 = 1;
const DEFAULT_HOST_SUMMARY_SECTIONS: &[&str] = &["execution", "transport", "faults"];
const SUPPORTED_DEBUG_SECTIONS: &[HostSummaryDebugSection] = &[HostSummaryDebugSection::Payload];
const INTERRUPTION_BOUNDARY: &str = "signal.runtime.interruption-boundary";
const INTERRUPTION_CONTRACT_PATH: &str =
    "docs/contracts/012-runtime-interruption-taxonomy-and-resumability-contract.md";
const INTERRUPTION_ACCEPTANCE_TASK: &str = "effigy acceptance:interruption-boundary";
const FAULT_DIAGNOSTIC_BOUNDARY: &str = "signal.runtime.fault-diagnostic-boundary";
const FAULT_DIAGNOSTIC_CONTRACT_PATH: &str =
    "docs/contracts/016-runtime-fault-cause-attribution-and-diagnostic-receipt-contract.md";
const FAULT_DIAGNOSTIC_ACCEPTANCE_TASK: &str = "effigy acceptance:fault-diagnostic-boundary";
const CRITICAL_PATH_BOUNDARY: &str = "signal.runtime.critical-path-boundary";
const CRITICAL_PATH_CONTRACT_PATH: &str =
    "docs/contracts/018-graph-critical-path-hot-node-and-worker-lane-instrumentation-contract.md";
const CRITICAL_PATH_ACCEPTANCE_TASK: &str = "effigy acceptance:critical-path-boundary";
const BLOCK_TIMING_BOUNDARY: &str = "signal.runtime.block-timing-boundary";
const BLOCK_TIMING_CONTRACT_PATH: &str =
    "docs/contracts/017-per-block-execution-timing-and-pressure-snapshot-contract.md";
const BLOCK_TIMING_ACCEPTANCE_TASK: &str = "effigy acceptance:block-timing-boundary";
const DEFERRED_WORK_POLICY_BOUNDARY: &str = "signal.runtime.deferred-work-policy-boundary";
const DEFERRED_WORK_POLICY_CONTRACT_PATH: &str =
    "docs/contracts/019-deferred-work-scheduler-priority-backpressure-and-cancellation-contract.md";
const DEFERRED_WORK_POLICY_ACCEPTANCE_TASK: &str =
    "effigy acceptance:deferred-work-policy-boundary";
const RECORDING_CONTINUITY_BOUNDARY: &str = "signal.runtime.recording-continuity-boundary";
const RECORDING_CONTINUITY_CONTRACT_PATH: &str =
    "docs/contracts/013-recording-continuity-midi-capture-and-checkpoint-contract.md";
const RECORDING_CONTINUITY_ACCEPTANCE_TASK: &str = "effigy acceptance:recording-continuity";
const OFFLINE_RENDER_CONTINUITY_BOUNDARY: &str =
    "signal.runtime.offline-render-continuity-boundary";
const OFFLINE_RENDER_CONTINUITY_CONTRACT_PATH: &str =
    "docs/contracts/015-offline-render-recovery-and-resumability-contract.md";
const OFFLINE_RENDER_CONTINUITY_ACCEPTANCE_TASK: &str =
    "effigy acceptance:offline-render-continuity";
const PLUGIN_CONTINUITY_BOUNDARY: &str = "signal.runtime.plugin-continuity-boundary";
const PLUGIN_CONTINUITY_CONTRACT_PATH: &str =
    "docs/contracts/014-plugin-isolation-policy-transport-rebind-and-shared-sandbox-continuity-contract.md";
const PLUGIN_CONTINUITY_ACCEPTANCE_TASK: &str = "effigy acceptance:plugin-continuity";
const VST3_BOUNDARY: &str = "signal.runtime.vst3-boundary";
const VST3_CONTRACT_PATH: &str =
    "docs/contracts/020-vst3-adapter-baseline-and-runtime-owned-lifecycle-contract.md";
const VST3_ACCEPTANCE_TASK: &str = "effigy acceptance:vst3-boundary";
const AU_BOUNDARY: &str = "signal.runtime.au-boundary";
const AU_CONTRACT_PATH: &str =
    "docs/contracts/021-au-adapter-baseline-and-runtime-owned-lifecycle-contract.md";
const AU_ACCEPTANCE_TASK: &str = "effigy acceptance:au-boundary";
const LV2_BOUNDARY: &str = "signal.runtime.lv2-boundary";
const LV2_CONTRACT_PATH: &str =
    "docs/contracts/055-lv2-worker-urid-patch-and-extension-negotiation-contract.md";
const LV2_ACCEPTANCE_TASK: &str = "effigy acceptance:lv2-boundary";
const CROSS_ADAPTER_PARITY_BOUNDARY: &str = "signal.runtime.cross-adapter-parity-boundary";
const CROSS_ADAPTER_PARITY_CONTRACT_PATH: &str =
    "docs/contracts/022-backend-capability-parity-linux-plugin-support-and-cross-adapter-conformance-contract.md";
const CROSS_ADAPTER_PARITY_ACCEPTANCE_TASK: &str =
    "effigy acceptance:cross-adapter-parity-boundary";
const LINUX_PLUGIN_PARITY_BOUNDARY: &str = "signal.runtime.linux-plugin-parity-boundary";
const LINUX_PLUGIN_PARITY_CONTRACT_PATH: &str =
    "docs/contracts/039-linux-cross-adapter-plugin-parity-and-sandbox-policy-contract.md";
const LINUX_PLUGIN_PARITY_ACCEPTANCE_TASK: &str = "effigy acceptance:linux-plugin-parity-boundary";
const LINUX_AUDIO_BACKEND_BOUNDARY: &str = "signal.runtime.linux-audio-backend-boundary";
const LINUX_AUDIO_BACKEND_CONTRACT_PATH: &str =
    "docs/contracts/040-linux-audio-backend-portability-across-alsa-jack-and-pipewire-contract.md";
const LINUX_AUDIO_BACKEND_ACCEPTANCE_TASK: &str = "effigy acceptance:linux-audio-backend-boundary";
const LINUX_LIVE_OWNERSHIP_BOUNDARY: &str = "signal.runtime.linux-live-ownership-boundary";
const LINUX_LIVE_OWNERSHIP_CONTRACT_PATH: &str =
    "docs/contracts/052-live-linux-audio-backend-ownership-and-session-lifecycle-contract.md";
const LINUX_LIVE_OWNERSHIP_ACCEPTANCE_TASK: &str =
    "effigy acceptance:linux-live-ownership-boundary";
const JACK_COORDINATION_BOUNDARY: &str = "signal.runtime.jack-coordination-boundary";
const JACK_COORDINATION_CONTRACT_PATH: &str =
    "docs/contracts/053-jack-transport-graph-and-backend-native-coordination-contract.md";
const JACK_COORDINATION_ACCEPTANCE_TASK: &str = "effigy acceptance:jack-coordination-boundary";
const PIPEWIRE_ALSA_PARITY_BOUNDARY: &str = "signal.runtime.pipewire-alsa-parity-boundary";
const PIPEWIRE_ALSA_PARITY_CONTRACT_PATH: &str =
    "docs/contracts/054-pipewire-and-alsa-session-role-device-claim-and-stream-policy-parity-contract.md";
const PIPEWIRE_ALSA_PARITY_ACCEPTANCE_TASK: &str =
    "effigy acceptance:pipewire-alsa-parity-boundary";
const LINUX_BACKEND_CLOCK_TOPOLOGY_BOUNDARY: &str =
    "signal.runtime.linux-backend-clock-topology-boundary";
const LINUX_BACKEND_CLOCK_TOPOLOGY_CONTRACT_PATH: &str =
    "docs/contracts/041-linux-backend-clocking-duplex-and-endpoint-topology-parity-contract.md";
const LINUX_BACKEND_CLOCK_TOPOLOGY_ACCEPTANCE_TASK: &str =
    "effigy acceptance:linux-backend-clock-topology-boundary";
const EXTERNAL_MIDI_BOUNDARY: &str = "signal.runtime.external-midi-boundary";
const EXTERNAL_MIDI_CONTRACT_PATH: &str =
    "docs/contracts/065-live-external-midi-device-ownership-and-backend-parity-contract.md";
const EXTERNAL_MIDI_ACCEPTANCE_TASK: &str = "effigy acceptance:external-midi-boundary";
const GENERIC_EVENT_BOUNDARY: &str = "signal.runtime.generic-event-boundary";
const GENERIC_EVENT_CONTRACT_PATH: &str =
    "docs/contracts/023-generic-midi-note-expression-and-plugin-event-model-contract.md";
const GENERIC_EVENT_ACCEPTANCE_TASK: &str = "effigy acceptance:generic-event-boundary";
const CONTROLLER_EXPRESSION_BOUNDARY: &str = "signal.runtime.controller-expression-boundary";
const CONTROLLER_EXPRESSION_CONTRACT_PATH: &str =
    "docs/contracts/043-midi-2-0-mpe-and-richer-controller-expression-contract.md";
const CONTROLLER_EXPRESSION_ACCEPTANCE_TASK: &str =
    "effigy acceptance:controller-expression-boundary";
const CONTROL_SURFACE_BOUNDARY: &str = "signal.runtime.control-surface-boundary";
const CONTROL_SURFACE_CONTRACT_PATH: &str =
    "docs/contracts/044-control-surface-transport-mapping-and-feedback-contract.md";
const CONTROL_SURFACE_ACCEPTANCE_TASK: &str = "effigy acceptance:control-surface-boundary";
const ADVANCED_HARDWARE_BOUNDARY: &str = "signal.runtime.advanced-hardware-boundary";
const ADVANCED_HARDWARE_CONTRACT_PATH: &str =
    "docs/contracts/061-control-surface-scene-mapping-feedback-pages-and-safe-action-graph-contract.md";
const ADVANCED_HARDWARE_ACCEPTANCE_TASK: &str = "effigy acceptance:advanced-hardware-boundary";
const RECALL_PORTABILITY_BOUNDARY: &str = "signal.runtime.recall-portability-boundary";
const RECALL_PORTABILITY_CONTRACT_PATH: &str =
    "docs/contracts/024-plugin-preset-state-interchange-portable-recall-and-ara-context-contract.md";
const RECALL_PORTABILITY_ACCEPTANCE_TASK: &str = "effigy acceptance:recall-portability-boundary";
const DEVICE_SUPERVISION_BOUNDARY: &str = "signal.runtime.device-supervision-boundary";
const DEVICE_SUPERVISION_CONTRACT_PATH: &str =
    "docs/contracts/025-device-supervision-restart-state-machine-and-fault-boundary-contract.md";
const DEVICE_SUPERVISION_ACCEPTANCE_TASK: &str = "effigy acceptance:device-supervision-boundary";
const CLOCK_TOPOLOGY_BOUNDARY: &str = "signal.runtime.clock-topology-boundary";
const CLOCK_TOPOLOGY_CONTRACT_PATH: &str =
    "docs/contracts/026-clock-domain-drift-duplex-mismatch-and-endpoint-topology-contract.md";
const CLOCK_TOPOLOGY_ACCEPTANCE_TASK: &str = "effigy acceptance:clock-topology-boundary";
const EXTERNAL_IO_BOUNDARY: &str = "signal.runtime.external-io-boundary";
const EXTERNAL_IO_CONTRACT_PATH: &str =
    "docs/contracts/027-external-io-monitoring-tap-point-and-loopback-measurement-contract.md";
const EXTERNAL_IO_ACCEPTANCE_TASK: &str = "effigy acceptance:external-io-boundary";
const MEDIA_SERVICE_BOUNDARY: &str = "signal.runtime.media-service-boundary";
const MEDIA_SERVICE_CONTRACT_PATH: &str =
    "docs/contracts/028-media-indexing-waveform-analysis-and-preview-service-contract.md";
const MEDIA_SERVICE_ACCEPTANCE_TASK: &str = "effigy acceptance:media-service-boundary";
const ANALYSIS_METADATA_BOUNDARY: &str = "signal.runtime.analysis-metadata-boundary";
const ANALYSIS_METADATA_CONTRACT_PATH: &str =
    "docs/contracts/029-analysis-metadata-extraction-and-library-service-contract.md";
const ANALYSIS_METADATA_ACCEPTANCE_TASK: &str = "effigy acceptance:analysis-metadata-boundary";
const MULTICHANNEL_BOUNDARY: &str = "signal.runtime.multichannel-boundary";
const MULTICHANNEL_CONTRACT_PATH: &str =
    "docs/contracts/032-canonical-multichannel-layout-and-channel-role-contract.md";
const MULTICHANNEL_ACCEPTANCE_TASK: &str = "effigy acceptance:multichannel-boundary";
const MULTI_BUS_BOUNDARY: &str = "signal.runtime.multi-bus-boundary";
const MULTI_BUS_CONTRACT_PATH: &str =
    "docs/contracts/034-multi-bus-graph-execution-and-auxiliary-topology-contract.md";
const MULTI_BUS_ACCEPTANCE_TASK: &str = "effigy acceptance:multi-bus-boundary";
const SIDECHAIN_BOUNDARY: &str = "signal.runtime.sidechain-boundary";
const SIDECHAIN_CONTRACT_PATH: &str =
    "docs/contracts/033-sidechain-routing-and-secondary-input-execution-contract.md";
const SIDECHAIN_ACCEPTANCE_TASK: &str = "effigy acceptance:sidechain-boundary";
const COMPLEX_IO_BOUNDARY: &str = "signal.runtime.complex-io-boundary";
const COMPLEX_IO_CONTRACT_PATH: &str =
    "docs/contracts/056-complex-plugin-pin-matrix-and-dynamic-bus-negotiation-contract.md";
const COMPLEX_IO_ACCEPTANCE_TASK: &str = "effigy acceptance:complex-io-boundary";
const SPATIAL_BOUNDARY: &str = "signal.runtime.spatial-boundary";
const SPATIAL_CONTRACT_PATH: &str =
    "docs/contracts/059-renderer-capability-negotiation-and-immersive-export-contract.md";
const SPATIAL_ACCEPTANCE_TASK: &str = "effigy acceptance:spatial-boundary";
const STRETCH_BOUNDARY: &str = "signal.runtime.stretch-boundary";
const STRETCH_CONTRACT_PATH: &str =
    "docs/contracts/046-sample-domain-time-stretch-engine-contract.md";
const STRETCH_ACCEPTANCE_TASK: &str = "effigy acceptance:stretch-boundary";
const MARKER_ANALYSIS_BOUNDARY: &str = "signal.runtime.marker-analysis-boundary";
const MARKER_ANALYSIS_CONTRACT_PATH: &str =
    "docs/contracts/047-warp-marker-transient-anchor-and-tempo-assist-analysis-contract.md";
const MARKER_ANALYSIS_ACCEPTANCE_TASK: &str = "effigy acceptance:marker-analysis-boundary";
const TRANSFORM_ARTIFACT_BOUNDARY: &str = "signal.runtime.transform-artifact-boundary";
const TRANSFORM_ARTIFACT_CONTRACT_PATH: &str =
    "docs/contracts/064-asset-session-transform-persistence-retention-and-cache-placement-policy-contract.md";
const TRANSFORM_ARTIFACT_ACCEPTANCE_TASK: &str = "effigy acceptance:transform-artifact-boundary";
const PREVIEW_TRANSFORM_BOUNDARY: &str = "signal.runtime.preview-transform-boundary";
const PREVIEW_TRANSFORM_CONTRACT_PATH: &str =
    "docs/contracts/063-preview-browser-queue-media-audition-and-transform-scheduling-contract.md";
const PREVIEW_TRANSFORM_ACCEPTANCE_TASK: &str = "effigy acceptance:preview-transform-boundary";
const INTEGRATED_ACCEPTANCE_LANE: &str = "signal.runtime.integrated-acceptance-lane";
const INTEGRATED_ACCEPTANCE_CONTRACT_PATH: &str =
    "docs/contracts/030-fault-injection-harness-and-multi-backend-acceptance-contract.md";
const INTEGRATED_ACCEPTANCE_TASK: &str = "effigy acceptance:integrated-acceptance-lane";
const G07_ACCEPTANCE_LANE: &str = "signal.runtime.g07-integrated-acceptance-lane";
const G07_ACCEPTANCE_CONTRACT_PATH: &str =
    "docs/contracts/050-multichannel-linux-time-stretch-and-control-surface-acceptance-contract.md";
const G07_ACCEPTANCE_TASK: &str = "effigy acceptance:g07-integrated-acceptance-lane";
const DEVICE_WORKFLOW_ACCEPTANCE_LANE: &str = "signal.runtime.device-workflow-acceptance-lane";
const DEVICE_WORKFLOW_ACCEPTANCE_CONTRACT_PATH: &str =
    "docs/contracts/066-cross-backend-device-protocol-and-live-workflow-acceptance-contract.md";
const DEVICE_WORKFLOW_ACCEPTANCE_TASK: &str = "effigy acceptance:device-workflow-acceptance-lane";
const LINUX_LIVE_ACCEPTANCE_LANE: &str = "signal.runtime.linux-live-acceptance-lane";
const LINUX_LIVE_ACCEPTANCE_CONTRACT_PATH: &str =
    "docs/contracts/067-live-linux-backend-acceptance-and-failure-injection-contract.md";
const LINUX_LIVE_ACCEPTANCE_TASK: &str = "effigy acceptance:linux-live-acceptance-lane";
const IMMERSIVE_ACCEPTANCE_LANE: &str = "signal.runtime.immersive-acceptance-lane";
const IMMERSIVE_ACCEPTANCE_CONTRACT_PATH: &str =
    "docs/contracts/068-immersive-render-and-monitoring-acceptance-contract.md";
const IMMERSIVE_ACCEPTANCE_TASK: &str = "effigy acceptance:immersive-acceptance-lane";
const CONTROL_PREVIEW_WORKFLOW_ACCEPTANCE_LANE: &str =
    "signal.runtime.control-preview-workflow-acceptance-lane";
const CONTROL_PREVIEW_WORKFLOW_ACCEPTANCE_CONTRACT_PATH: &str =
    "docs/contracts/069-control-surface-and-preview-workflow-acceptance-contract.md";
const CONTROL_PREVIEW_WORKFLOW_ACCEPTANCE_TASK: &str =
    "effigy acceptance:control-preview-workflow-acceptance-lane";
const INTEGRATED_LIVE_WORKFLOW_ACCEPTANCE_LANE: &str =
    "signal.runtime.integrated-live-ownership-and-workflow-acceptance-lane";
const INTEGRATED_LIVE_WORKFLOW_ACCEPTANCE_CONTRACT_PATH: &str =
    "docs/contracts/070-integrated-live-ownership-and-workflow-acceptance-contract.md";
const INTEGRATED_LIVE_WORKFLOW_ACCEPTANCE_TASK: &str =
    "effigy acceptance:integrated-live-ownership-and-workflow-acceptance-lane";
const G06_SOAK_LANE: &str = "signal.g06.long-session-soak-lane";
const G06_SOAK_CONTRACT_PATH: &str =
    "docs/contracts/031-long-session-soak-promotion-gate-and-loophole-readiness-contract.md";
const G06_SOAK_ACCEPTANCE_TASK: &str = "effigy acceptance:g06-soak-lane";
const INTEGRATED_ACCEPTANCE_REQUIRED_TASKS: &[&str] = &[
    INTERRUPTION_ACCEPTANCE_TASK,
    FAULT_DIAGNOSTIC_ACCEPTANCE_TASK,
    CRITICAL_PATH_ACCEPTANCE_TASK,
    DEFERRED_WORK_POLICY_ACCEPTANCE_TASK,
    PLUGIN_CONTINUITY_ACCEPTANCE_TASK,
    CROSS_ADAPTER_PARITY_ACCEPTANCE_TASK,
    DEVICE_SUPERVISION_ACCEPTANCE_TASK,
    CLOCK_TOPOLOGY_ACCEPTANCE_TASK,
    EXTERNAL_IO_ACCEPTANCE_TASK,
    MEDIA_SERVICE_ACCEPTANCE_TASK,
    ANALYSIS_METADATA_ACCEPTANCE_TASK,
];
const INTEGRATED_ACCEPTANCE_ADVISORY_TASKS: &[&str] = &[
    RECORDING_CONTINUITY_ACCEPTANCE_TASK,
    OFFLINE_RENDER_CONTINUITY_ACCEPTANCE_TASK,
    VST3_ACCEPTANCE_TASK,
    AU_ACCEPTANCE_TASK,
    GENERIC_EVENT_ACCEPTANCE_TASK,
    RECALL_PORTABILITY_ACCEPTANCE_TASK,
];
const RECOVERY_AND_FAULT_REQUIRED_TASKS: &[&str] = &[
    INTERRUPTION_ACCEPTANCE_TASK,
    FAULT_DIAGNOSTIC_ACCEPTANCE_TASK,
    DEVICE_SUPERVISION_ACCEPTANCE_TASK,
];
const RECOVERY_AND_FAULT_ADVISORY_TASKS: &[&str] = &[
    RECORDING_CONTINUITY_ACCEPTANCE_TASK,
    OFFLINE_RENDER_CONTINUITY_ACCEPTANCE_TASK,
];
const SCHEDULING_AND_PRESSURE_REQUIRED_TASKS: &[&str] = &[
    CRITICAL_PATH_ACCEPTANCE_TASK,
    DEFERRED_WORK_POLICY_ACCEPTANCE_TASK,
];
const SCHEDULING_AND_PRESSURE_ADVISORY_TASKS: &[&str] = &[BLOCK_TIMING_ACCEPTANCE_TASK];
const ADAPTER_AND_PORTABILITY_REQUIRED_TASKS: &[&str] = &[
    PLUGIN_CONTINUITY_ACCEPTANCE_TASK,
    CROSS_ADAPTER_PARITY_ACCEPTANCE_TASK,
];
const ADAPTER_AND_PORTABILITY_ADVISORY_TASKS: &[&str] = &[
    VST3_ACCEPTANCE_TASK,
    AU_ACCEPTANCE_TASK,
    GENERIC_EVENT_ACCEPTANCE_TASK,
    RECALL_PORTABILITY_ACCEPTANCE_TASK,
];
const HARDWARE_AND_EXTERNAL_IO_REQUIRED_TASKS: &[&str] = &[
    DEVICE_SUPERVISION_ACCEPTANCE_TASK,
    CLOCK_TOPOLOGY_ACCEPTANCE_TASK,
    EXTERNAL_IO_ACCEPTANCE_TASK,
];
const HARDWARE_AND_EXTERNAL_IO_ADVISORY_TASKS: &[&str] = &[];
const MEDIA_AND_LIBRARY_REQUIRED_TASKS: &[&str] = &[
    MEDIA_SERVICE_ACCEPTANCE_TASK,
    ANALYSIS_METADATA_ACCEPTANCE_TASK,
];
const MEDIA_AND_LIBRARY_ADVISORY_TASKS: &[&str] = &[];
const G07_ACCEPTANCE_REQUIRED_TASKS: &[&str] = &[
    MULTICHANNEL_ACCEPTANCE_TASK,
    SIDECHAIN_ACCEPTANCE_TASK,
    MULTI_BUS_ACCEPTANCE_TASK,
    SPATIAL_ACCEPTANCE_TASK,
    LINUX_PLUGIN_PARITY_ACCEPTANCE_TASK,
    LINUX_AUDIO_BACKEND_ACCEPTANCE_TASK,
    LINUX_BACKEND_CLOCK_TOPOLOGY_ACCEPTANCE_TASK,
    EXTERNAL_MIDI_ACCEPTANCE_TASK,
    CONTROLLER_EXPRESSION_ACCEPTANCE_TASK,
    CONTROL_SURFACE_ACCEPTANCE_TASK,
    ADVANCED_HARDWARE_ACCEPTANCE_TASK,
    STRETCH_ACCEPTANCE_TASK,
    MARKER_ANALYSIS_ACCEPTANCE_TASK,
    TRANSFORM_ARTIFACT_ACCEPTANCE_TASK,
    PREVIEW_TRANSFORM_ACCEPTANCE_TASK,
];
const G07_ACCEPTANCE_ADVISORY_TASKS: &[&str] = &[COMPLEX_IO_ACCEPTANCE_TASK, LV2_ACCEPTANCE_TASK];
const G07_ROUTING_REQUIRED_TASKS: &[&str] = &[
    MULTICHANNEL_ACCEPTANCE_TASK,
    SIDECHAIN_ACCEPTANCE_TASK,
    MULTI_BUS_ACCEPTANCE_TASK,
    SPATIAL_ACCEPTANCE_TASK,
];
const G07_ROUTING_ADVISORY_TASKS: &[&str] = &[COMPLEX_IO_ACCEPTANCE_TASK];
const G07_LINUX_REQUIRED_TASKS: &[&str] = &[
    LINUX_PLUGIN_PARITY_ACCEPTANCE_TASK,
    LINUX_AUDIO_BACKEND_ACCEPTANCE_TASK,
    LINUX_BACKEND_CLOCK_TOPOLOGY_ACCEPTANCE_TASK,
];
const G07_LINUX_ADVISORY_TASKS: &[&str] = &[LV2_ACCEPTANCE_TASK];
const G07_CONTROL_REQUIRED_TASKS: &[&str] = &[
    EXTERNAL_MIDI_ACCEPTANCE_TASK,
    CONTROLLER_EXPRESSION_ACCEPTANCE_TASK,
    CONTROL_SURFACE_ACCEPTANCE_TASK,
    ADVANCED_HARDWARE_ACCEPTANCE_TASK,
];
const G07_CONTROL_ADVISORY_TASKS: &[&str] = &[];
const G07_STRETCH_REQUIRED_TASKS: &[&str] = &[
    STRETCH_ACCEPTANCE_TASK,
    MARKER_ANALYSIS_ACCEPTANCE_TASK,
    TRANSFORM_ARTIFACT_ACCEPTANCE_TASK,
    PREVIEW_TRANSFORM_ACCEPTANCE_TASK,
];
const G07_STRETCH_ADVISORY_TASKS: &[&str] = &[];
const DEVICE_WORKFLOW_ACCEPTANCE_REQUIRED_TASKS: &[&str] = &[
    EXTERNAL_MIDI_ACCEPTANCE_TASK,
    CONTROLLER_EXPRESSION_ACCEPTANCE_TASK,
    CONTROL_SURFACE_ACCEPTANCE_TASK,
    ADVANCED_HARDWARE_ACCEPTANCE_TASK,
];
const DEVICE_WORKFLOW_ACCEPTANCE_ADVISORY_TASKS: &[&str] = &[];
const DEVICE_WORKFLOW_LIVE_PROTOCOL_REQUIRED_TASKS: &[&str] = &[
    EXTERNAL_MIDI_ACCEPTANCE_TASK,
    CONTROLLER_EXPRESSION_ACCEPTANCE_TASK,
];
const DEVICE_WORKFLOW_LIVE_PROTOCOL_ADVISORY_TASKS: &[&str] = &[];
const DEVICE_WORKFLOW_CONTROL_REQUIRED_TASKS: &[&str] = &[
    CONTROL_SURFACE_ACCEPTANCE_TASK,
    ADVANCED_HARDWARE_ACCEPTANCE_TASK,
];
const DEVICE_WORKFLOW_CONTROL_ADVISORY_TASKS: &[&str] = &[];
const DEVICE_WORKFLOW_HOST_EDGE_REQUIRED_TASKS: &[&str] = &[
    EXTERNAL_MIDI_ACCEPTANCE_TASK,
    CONTROL_SURFACE_ACCEPTANCE_TASK,
    ADVANCED_HARDWARE_ACCEPTANCE_TASK,
];
const DEVICE_WORKFLOW_HOST_EDGE_ADVISORY_TASKS: &[&str] = &[];
const LINUX_LIVE_ACCEPTANCE_REQUIRED_TASKS: &[&str] = &[
    LINUX_LIVE_OWNERSHIP_ACCEPTANCE_TASK,
    JACK_COORDINATION_ACCEPTANCE_TASK,
    PIPEWIRE_ALSA_PARITY_ACCEPTANCE_TASK,
    LINUX_BACKEND_CLOCK_TOPOLOGY_ACCEPTANCE_TASK,
];
const LINUX_LIVE_ACCEPTANCE_ADVISORY_TASKS: &[&str] = &[];
const LINUX_LIVE_OWNERSHIP_REQUIRED_TASKS: &[&str] = &[
    LINUX_LIVE_OWNERSHIP_ACCEPTANCE_TASK,
    LINUX_BACKEND_CLOCK_TOPOLOGY_ACCEPTANCE_TASK,
];
const LINUX_LIVE_OWNERSHIP_ADVISORY_TASKS: &[&str] = &[];
const LINUX_LIVE_BACKEND_PROTOCOL_REQUIRED_TASKS: &[&str] = &[
    JACK_COORDINATION_ACCEPTANCE_TASK,
    PIPEWIRE_ALSA_PARITY_ACCEPTANCE_TASK,
];
const LINUX_LIVE_BACKEND_PROTOCOL_ADVISORY_TASKS: &[&str] = &[];
const LINUX_LIVE_HOST_EDGE_REQUIRED_TASKS: &[&str] = &[
    LINUX_LIVE_OWNERSHIP_ACCEPTANCE_TASK,
    JACK_COORDINATION_ACCEPTANCE_TASK,
    PIPEWIRE_ALSA_PARITY_ACCEPTANCE_TASK,
];
const LINUX_LIVE_HOST_EDGE_ADVISORY_TASKS: &[&str] = &[];
const IMMERSIVE_ACCEPTANCE_REQUIRED_TASKS: &[&str] = &[SPATIAL_ACCEPTANCE_TASK];
const IMMERSIVE_ACCEPTANCE_ADVISORY_TASKS: &[&str] = &[];
const IMMERSIVE_RENDER_REQUIRED_TASKS: &[&str] = &[SPATIAL_ACCEPTANCE_TASK];
const IMMERSIVE_RENDER_ADVISORY_TASKS: &[&str] = &[];
const IMMERSIVE_MONITORING_REQUIRED_TASKS: &[&str] = &[SPATIAL_ACCEPTANCE_TASK];
const IMMERSIVE_MONITORING_ADVISORY_TASKS: &[&str] = &[];
const IMMERSIVE_HOST_EDGE_REQUIRED_TASKS: &[&str] = &[SPATIAL_ACCEPTANCE_TASK];
const IMMERSIVE_HOST_EDGE_ADVISORY_TASKS: &[&str] = &[];
const CONTROL_PREVIEW_WORKFLOW_ACCEPTANCE_REQUIRED_TASKS: &[&str] = &[
    ADVANCED_HARDWARE_ACCEPTANCE_TASK,
    PREVIEW_TRANSFORM_ACCEPTANCE_TASK,
];
const CONTROL_PREVIEW_WORKFLOW_ACCEPTANCE_ADVISORY_TASKS: &[&str] = &[];
const CONTROL_WORKFLOW_REQUIRED_TASKS: &[&str] = &[ADVANCED_HARDWARE_ACCEPTANCE_TASK];
const CONTROL_WORKFLOW_ADVISORY_TASKS: &[&str] = &[];
const PREVIEW_WORKFLOW_REQUIRED_TASKS: &[&str] = &[PREVIEW_TRANSFORM_ACCEPTANCE_TASK];
const PREVIEW_WORKFLOW_ADVISORY_TASKS: &[&str] = &[];
const CONTROL_PREVIEW_HOST_EDGE_REQUIRED_TASKS: &[&str] = &[
    ADVANCED_HARDWARE_ACCEPTANCE_TASK,
    PREVIEW_TRANSFORM_ACCEPTANCE_TASK,
];
const CONTROL_PREVIEW_HOST_EDGE_ADVISORY_TASKS: &[&str] = &[];
const INTEGRATED_LIVE_WORKFLOW_ACCEPTANCE_REQUIRED_TASKS: &[&str] = &[
    LINUX_LIVE_ACCEPTANCE_TASK,
    DEVICE_WORKFLOW_ACCEPTANCE_TASK,
    IMMERSIVE_ACCEPTANCE_TASK,
    CONTROL_PREVIEW_WORKFLOW_ACCEPTANCE_TASK,
];
const INTEGRATED_LIVE_WORKFLOW_ACCEPTANCE_ADVISORY_TASKS: &[&str] = &[];
const INTEGRATED_LIVE_AND_DEVICE_REQUIRED_TASKS: &[&str] =
    &[LINUX_LIVE_ACCEPTANCE_TASK, DEVICE_WORKFLOW_ACCEPTANCE_TASK];
const INTEGRATED_LIVE_AND_DEVICE_ADVISORY_TASKS: &[&str] = &[];
const INTEGRATED_IMMERSIVE_AND_PREVIEW_REQUIRED_TASKS: &[&str] = &[
    IMMERSIVE_ACCEPTANCE_TASK,
    CONTROL_PREVIEW_WORKFLOW_ACCEPTANCE_TASK,
];
const INTEGRATED_IMMERSIVE_AND_PREVIEW_ADVISORY_TASKS: &[&str] = &[];
const INTEGRATED_CROSS_SURFACE_REQUIRED_TASKS: &[&str] = &[
    LINUX_LIVE_ACCEPTANCE_TASK,
    DEVICE_WORKFLOW_ACCEPTANCE_TASK,
    IMMERSIVE_ACCEPTANCE_TASK,
    CONTROL_PREVIEW_WORKFLOW_ACCEPTANCE_TASK,
];
const INTEGRATED_CROSS_SURFACE_ADVISORY_TASKS: &[&str] = &[];
const INTEGRATED_GROUPED_EXPORT_REQUIRED_TASKS: &[&str] = &[
    LINUX_LIVE_ACCEPTANCE_TASK,
    DEVICE_WORKFLOW_ACCEPTANCE_TASK,
    IMMERSIVE_ACCEPTANCE_TASK,
    CONTROL_PREVIEW_WORKFLOW_ACCEPTANCE_TASK,
];
const INTEGRATED_GROUPED_EXPORT_ADVISORY_TASKS: &[&str] = &[];
const HOST_EDGE_BOUNDARY: &str = "signal.host.edge.boundary";
const HOST_EDGE_CONTRACT_PATH: &str =
    "docs/contracts/009-shared-host-convenience-api-and-consumer-edge-contract.md";
const HOST_EDGE_ACCEPTANCE_TASK: &str = "effigy acceptance:host-edge-consumer";
const RELEASE_BOUNDARY: &str = "signal.release.boundary";
const RELEASE_VERSION_SOURCE: &str = "workspace.package.version";
const RELEASE_CHANGELOG_PATH: &str = "CHANGELOG.md";
const RELEASE_CONFORMANCE_TASK: &str = "effigy acceptance:conformance";
const PACKAGING_MANIFEST: &str = "signal.release.packaging-manifest";
const PACKAGING_MANIFEST_CONTRACT_PATH: &str =
    "docs/contracts/010-publication-grade-packaging-manifest-and-release-receipt-contract.md";
const PACKAGING_MANIFEST_ACCEPTANCE_TASK: &str = "effigy acceptance:release-packaging-consumer";
const DOWNSTREAM_AUTOMATION_BOUNDARY: &str = "signal.downstream.automation";
const DOWNSTREAM_AUTOMATION_CONTRACT_PATH: &str =
    "docs/contracts/011-shared-downstream-conformance-and-release-acceptance-automation-contract.md";
const DOWNSTREAM_AUTOMATION_MANDATORY_TASK: &str = "effigy acceptance:downstream-release";
const DOWNSTREAM_AUTOMATION_OPTIONAL_TASK: &str = "effigy acceptance:downstream-depth";
const DOWNSTREAM_AUTOMATION_COMBINED_TASK: &str = "effigy acceptance:downstream-automation";
const DOWNSTREAM_FAIL_GATES: &str = "signal.downstream.fail-gates";
const DOWNSTREAM_FAIL_GATE_TASK: &str = "effigy acceptance:downstream-gate";
const GENERATION_CLOSEOUT: &str = "signal.generation.closeout";
const GENERATION_CLOSEOUT_GENERATION: &str = "g08";
const GENERATION_CLOSEOUT_TASK: &str = "effigy acceptance:g08-closeout";
const GENERATION_CLOSEOUT_CONTRACT_PATH: &str =
    "docs/contracts/071-generation-closeout-and-downstream-workflow-readiness-gate-contract.md";
const GENERATION_CLOSEOUT_ROADMAP_PATH: &str =
    "docs/roadmaps/g08/020-generation-closeout-and-downstream-workflow-readiness-gate.md";
const GENERATION_CLOSEOUT_BACKLOG_PATH: &str =
    "docs/roadmaps/backlog/post-g08-repeated-run-environment-matrices-and-downstream-workflow-depth.md";
const G08_INTEGRATED_ACCEPTANCE_LANE_COMMAND: &str =
    "cargo run -p signal-supervisor-tools -- --describe-integrated-live-workflow-acceptance-lane --format=json";
const GENERATION_CLOSEOUT_NEXT_QUEUE_PATH: &str = GENERATION_CLOSEOUT_BACKLOG_PATH;
const GENERATION_CLOSEOUT_GATE_STATUS: &str = "complete";
const GENERATION_CLOSEOUT_NEXT_QUEUE_STATUS: &str = "backlog";
const GENERATION_CLOSEOUT_PROMOTION_DECISION: &str = "close-g08-and-handoff-to-post-g08-backlog";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ConformanceMatrixEntryKind {
    PublicBoundaryTest,
    ExportConsumerTest,
    Example,
    Introspection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ConformanceMatrixEntry {
    id: &'static str,
    kind: ConformanceMatrixEntryKind,
    crate_name: &'static str,
    surface: &'static str,
    command: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct IntegratedAcceptanceFamily {
    id: &'static str,
    title: &'static str,
    required_tasks: &'static [&'static str],
    advisory_tasks: &'static [&'static str],
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct IntegratedAcceptanceValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct G06SoakLaneScenarioRecord {
    id: &'static str,
    status: &'static str,
    command: &'static str,
    typed_output: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct G06SoakLaneValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct GenerationReadinessArea {
    id: &'static str,
    status: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct GenerationCloseoutValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ExportDebugOptions {
    payload: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CliArgs {
    format: OutputFormat,
    debug: ExportDebugOptions,
    mode: CliMode,
}

impl HostProfile {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "local" => Ok(Self::Local),
            "server" => Ok(Self::Server),
            _ => Err(format!(
                "unknown profile {value:?}; expected one of: local, server"
            )),
        }
    }
}

impl Scenario {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "default" => Ok(Self::Default),
            "timeout" => Ok(Self::Timeout),
            "crash" => Ok(Self::Crash),
            "heartbeat" => Ok(Self::Heartbeat),
            "soak" => Ok(Self::Soak),
            "mixed" => Ok(Self::Mixed),
            _ => Err(format!(
                "unknown scenario {value:?}; expected one of: default, timeout, crash, heartbeat, soak, mixed"
            )),
        }
    }
}

impl OutputFormat {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "text" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            _ => Err(format!(
                "unknown format {value:?}; expected one of: text, json"
            )),
        }
    }
}

fn print_usage() {
    eprintln!(
        "usage: signal-supervisor-tools [--format text|json] [--include-payload] [--describe-export|--describe-conformance-matrix|--describe-interruption-boundary|--describe-fault-diagnostic-boundary|--describe-critical-path-boundary|--describe-block-timing-boundary|--describe-deferred-work-policy-boundary|--describe-recording-continuity-boundary|--describe-offline-render-continuity-boundary|--describe-plugin-continuity-boundary|--describe-vst3-boundary|--describe-au-boundary|--describe-lv2-boundary|--describe-cross-adapter-parity-boundary|--describe-linux-plugin-parity-boundary|--describe-linux-audio-backend-boundary|--describe-linux-live-ownership-boundary|--describe-jack-coordination-boundary|--describe-pipewire-alsa-parity-boundary|--describe-linux-backend-clock-topology-boundary|--describe-external-midi-boundary|--describe-generic-event-boundary|--describe-controller-expression-boundary|--describe-control-surface-boundary|--describe-advanced-hardware-boundary|--describe-recall-portability-boundary|--describe-device-supervision-boundary|--describe-clock-topology-boundary|--describe-external-io-boundary|--describe-media-service-boundary|--describe-analysis-metadata-boundary|--describe-multichannel-boundary|--describe-multi-bus-boundary|--describe-sidechain-boundary|--describe-complex-io-boundary|--describe-spatial-boundary|--describe-stretch-boundary|--describe-marker-analysis-boundary|--describe-transform-artifact-boundary|--describe-preview-transform-boundary|--describe-integrated-acceptance-lane|--describe-g07-acceptance-lane|--describe-device-workflow-acceptance-lane|--describe-linux-live-acceptance-lane|--describe-immersive-acceptance-lane|--describe-control-preview-workflow-acceptance-lane|--describe-integrated-live-workflow-acceptance-lane|--describe-g06-soak-lane|--describe-host-edge-boundary|--describe-release-boundary|--describe-packaging-manifest|--describe-downstream-automation|--describe-downstream-fail-gates|--describe-generation-closeout] <local|server> <default|timeout|crash|heartbeat|soak|mixed>"
    );
}

impl HostSummaryDebugSection {
    fn label(self) -> &'static str {
        match self {
            Self::Payload => "payload",
        }
    }
}

impl ExportDebugOptions {
    fn supports(self, section: HostSummaryDebugSection) -> bool {
        match section {
            HostSummaryDebugSection::Payload => self.payload,
        }
    }
}

impl ConformanceMatrixEntryKind {
    fn label(self) -> &'static str {
        match self {
            Self::PublicBoundaryTest => "public-boundary-test",
            Self::ExportConsumerTest => "export-consumer-test",
            Self::Example => "example",
            Self::Introspection => "introspection",
        }
    }
}

fn conformance_matrix_entries() -> &'static [ConformanceMatrixEntry] {
    &[
        ConformanceMatrixEntry {
            id: "runtime-public-contract-boundary",
            kind: ConformanceMatrixEntryKind::PublicBoundaryTest,
            crate_name: "signal-runtime",
            surface: "SignalRuntime, RuntimeObservationReport, RuntimeSupervisorReport public reexports",
            command:
                "cargo test -p signal-runtime public_runtime_contract_boundary_is_consumable_from_reexports",
            rationale:
                "Proves a downstream-style consumer can capture runtime/export/plugin receipts without private internals.",
        },
        ConformanceMatrixEntry {
            id: "supervisor-export-discovery-consumer",
            kind: ConformanceMatrixEntryKind::ExportConsumerTest,
            crate_name: "signal-supervisor-tools",
            surface: "signal.supervisor.export JSON carrying runtime-owned plugin discovery catalog",
            command:
                "cargo test -p signal-supervisor-tools export_json_carries_runtime_owned_plugin_discovery_catalog",
            rationale:
                "Proves the versioned supervisor export carries the widened discovery boundary without host-local reconstruction.",
        },
        ConformanceMatrixEntry {
            id: "plugin-backend-breadth-coverage",
            kind: ConformanceMatrixEntryKind::ExportConsumerTest,
            crate_name: "signal-runtime + signal-supervisor-tools",
            surface: "runtime reexports and supervisor export carrying backend-neutral plugin discovery coverage aggregates",
            command: "effigy acceptance:plugin-backend-breadth",
            rationale:
                "Proves widened multi-format discovery and capability coverage stays consumable through Signal-owned runtime and export surfaces.",
        },
        ConformanceMatrixEntry {
            id: "shared-host-edge-consumer",
            kind: ConformanceMatrixEntryKind::PublicBoundaryTest,
            crate_name: "signal-host-local + signal-host-server",
            surface: "shared-stable host constructors, RuntimeSupervisorApi, and supervisor_report()",
            command: "effigy acceptance:host-edge-consumer",
            rationale:
                "Proves the shared stable host edge remains consumable without private host internals or unstable summary helpers.",
        },
        ConformanceMatrixEntry {
            id: "runtime-supervisor-report-demo",
            kind: ConformanceMatrixEntryKind::Example,
            crate_name: "signal-runtime",
            surface: "supervisor_report_demo example",
            command: "cargo run -p signal-runtime --example supervisor_report_demo",
            rationale:
                "Provides a host-free runnable example that emits the stabilized supervisor report surface.",
        },
        ConformanceMatrixEntry {
            id: "supervisor-export-schema-description",
            kind: ConformanceMatrixEntryKind::Introspection,
            crate_name: "signal-supervisor-tools",
            surface: "signal-supervisor-tools export/conformance schema description",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-conformance-matrix --format=json",
            rationale:
                "Lets consumers inspect the runnable conformance matrix without reading private implementation detail.",
        },
    ]
}

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

fn print_report(
    format: OutputFormat,
    profile: HostProfile,
    scenario: Scenario,
    summary: String,
    profiling: &RuntimeProfilingReceipt,
    soak: &RuntimeSoakReceipt,
    report: RuntimeSupervisorReport,
) {
    match format {
        OutputFormat::Text => println!(
            "signal-supervisor-tools profile={profile:?} scenario={scenario:?}\n{summary}\nprofiling:\n{}\nsoak:\n{}\nsupervisor:\n{}",
            profiling.render_multiline(),
            soak.render_multiline(),
            report.render_multiline()
        ),
        OutputFormat::Json => println!(
            "{}",
            render_supervisor_export_json(profile, scenario, summary, profiling, soak, &report)
        ),
    }
}

fn run_local(
    format: OutputFormat,
    debug: ExportDebugOptions,
    scenario: Scenario,
) -> Result<(), String> {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    let summary = match scenario {
        Scenario::Default => host.boot_default(),
        Scenario::Timeout => host.boot_with_timeout_recovery(),
        Scenario::Crash => host.boot_with_crash_recovery(),
        Scenario::Heartbeat => host.boot_with_heartbeat_miss_recovery(),
        Scenario::Soak => host.boot_with_watchdog_soak(),
        Scenario::Mixed => host.boot_with_mixed_watchdog_soak(),
    }
    .map_err(|error| error.message)?;
    let report = host.supervisor_report();
    let host_report = host.host_supervisor_report();
    let profiling = host_report.profiling_receipt();
    let soak = host_report.soak_receipt();
    print_report(
        format,
        HostProfile::Local,
        scenario,
        match format {
            OutputFormat::Text => render_local_summary(&summary, debug),
            OutputFormat::Json => render_local_summary_json(&summary, debug),
        },
        &profiling,
        &soak,
        report,
    );
    Ok(())
}

fn run_server(
    format: OutputFormat,
    debug: ExportDebugOptions,
    scenario: Scenario,
) -> Result<(), String> {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);
    let summary = match scenario {
        Scenario::Default => host.boot_default(),
        Scenario::Timeout => host.boot_with_timeout_recovery(),
        Scenario::Crash => host.boot_with_crash_recovery(),
        Scenario::Heartbeat => host.boot_with_heartbeat_miss_recovery(),
        Scenario::Soak => host.boot_with_watchdog_soak(),
        Scenario::Mixed => host.boot_with_mixed_watchdog_soak(),
    }
    .map_err(|error| error.message)?;
    let report = host.supervisor_report();
    let profiling = report.profiling_receipt();
    let soak = report.soak_receipt();
    print_report(
        format,
        HostProfile::Server,
        scenario,
        match format {
            OutputFormat::Text => render_server_summary(&summary, debug),
            OutputFormat::Json => render_server_summary_json(&summary, debug),
        },
        &profiling,
        &soak,
        report,
    );
    Ok(())
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<CliArgs, String> {
    let mut format = OutputFormat::Text;
    let mut debug = ExportDebugOptions::default();
    let mut describe_export = false;
    let mut describe_conformance_matrix = false;
    let mut describe_interruption_boundary = false;
    let mut describe_fault_diagnostic_boundary = false;
    let mut describe_critical_path_boundary = false;
    let mut describe_block_timing_boundary = false;
    let mut describe_deferred_work_policy_boundary = false;
    let mut describe_recording_continuity_boundary = false;
    let mut describe_offline_render_continuity_boundary = false;
    let mut describe_plugin_continuity_boundary = false;
    let mut describe_vst3_boundary = false;
    let mut describe_au_boundary = false;
    let mut describe_lv2_boundary = false;
    let mut describe_cross_adapter_parity_boundary = false;
    let mut describe_linux_plugin_parity_boundary = false;
    let mut describe_linux_audio_backend_boundary = false;
    let mut describe_linux_live_ownership_boundary = false;
    let mut describe_jack_coordination_boundary = false;
    let mut describe_pipewire_alsa_parity_boundary = false;
    let mut describe_linux_backend_clock_topology_boundary = false;
    let mut describe_external_midi_boundary = false;
    let mut describe_generic_event_boundary = false;
    let mut describe_controller_expression_boundary = false;
    let mut describe_control_surface_boundary = false;
    let mut describe_advanced_hardware_boundary = false;
    let mut describe_recall_portability_boundary = false;
    let mut describe_device_supervision_boundary = false;
    let mut describe_clock_topology_boundary = false;
    let mut describe_external_io_boundary = false;
    let mut describe_media_service_boundary = false;
    let mut describe_analysis_metadata_boundary = false;
    let mut describe_multichannel_boundary = false;
    let mut describe_multi_bus_boundary = false;
    let mut describe_sidechain_boundary = false;
    let mut describe_complex_io_boundary = false;
    let mut describe_spatial_boundary = false;
    let mut describe_stretch_boundary = false;
    let mut describe_marker_analysis_boundary = false;
    let mut describe_transform_artifact_boundary = false;
    let mut describe_preview_transform_boundary = false;
    let mut describe_integrated_acceptance_lane = false;
    let mut describe_g07_acceptance_lane = false;
    let mut describe_device_workflow_acceptance_lane = false;
    let mut describe_linux_live_acceptance_lane = false;
    let mut describe_immersive_acceptance_lane = false;
    let mut describe_control_preview_workflow_acceptance_lane = false;
    let mut describe_integrated_live_workflow_acceptance_lane = false;
    let mut describe_g06_soak_lane = false;
    let mut describe_host_edge_boundary = false;
    let mut describe_release_boundary = false;
    let mut describe_packaging_manifest = false;
    let mut describe_downstream_automation = false;
    let mut describe_downstream_fail_gates = false;
    let mut describe_generation_closeout = false;
    let mut positional = Vec::new();

    for arg in args {
        if arg == "--json" {
            format = OutputFormat::Json;
            continue;
        }
        if arg == "--text" {
            format = OutputFormat::Text;
            continue;
        }
        if arg == "--include-payload" {
            debug.payload = true;
            continue;
        }
        if arg == "--describe-export" {
            describe_export = true;
            continue;
        }
        if arg == "--describe-conformance-matrix" {
            describe_conformance_matrix = true;
            continue;
        }
        if arg == "--describe-interruption-boundary" {
            describe_interruption_boundary = true;
            continue;
        }
        if arg == "--describe-fault-diagnostic-boundary" {
            describe_fault_diagnostic_boundary = true;
            continue;
        }
        if arg == "--describe-critical-path-boundary" {
            describe_critical_path_boundary = true;
            continue;
        }
        if arg == "--describe-block-timing-boundary" {
            describe_block_timing_boundary = true;
            continue;
        }
        if arg == "--describe-deferred-work-policy-boundary" {
            describe_deferred_work_policy_boundary = true;
            continue;
        }
        if arg == "--describe-recording-continuity-boundary" {
            describe_recording_continuity_boundary = true;
            continue;
        }
        if arg == "--describe-offline-render-continuity-boundary" {
            describe_offline_render_continuity_boundary = true;
            continue;
        }
        if arg == "--describe-plugin-continuity-boundary" {
            describe_plugin_continuity_boundary = true;
            continue;
        }
        if arg == "--describe-vst3-boundary" {
            describe_vst3_boundary = true;
            continue;
        }
        if arg == "--describe-au-boundary" {
            describe_au_boundary = true;
            continue;
        }
        if arg == "--describe-lv2-boundary" {
            describe_lv2_boundary = true;
            continue;
        }
        if arg == "--describe-cross-adapter-parity-boundary" {
            describe_cross_adapter_parity_boundary = true;
            continue;
        }
        if arg == "--describe-linux-plugin-parity-boundary" {
            describe_linux_plugin_parity_boundary = true;
            continue;
        }
        if arg == "--describe-linux-audio-backend-boundary" {
            describe_linux_audio_backend_boundary = true;
            continue;
        }
        if arg == "--describe-linux-live-ownership-boundary" {
            describe_linux_live_ownership_boundary = true;
            continue;
        }
        if arg == "--describe-jack-coordination-boundary" {
            describe_jack_coordination_boundary = true;
            continue;
        }
        if arg == "--describe-pipewire-alsa-parity-boundary" {
            describe_pipewire_alsa_parity_boundary = true;
            continue;
        }
        if arg == "--describe-linux-backend-clock-topology-boundary" {
            describe_linux_backend_clock_topology_boundary = true;
            continue;
        }
        if arg == "--describe-external-midi-boundary" {
            describe_external_midi_boundary = true;
            continue;
        }
        if arg == "--describe-generic-event-boundary" {
            describe_generic_event_boundary = true;
            continue;
        }
        if arg == "--describe-controller-expression-boundary" {
            describe_controller_expression_boundary = true;
            continue;
        }
        if arg == "--describe-control-surface-boundary" {
            describe_control_surface_boundary = true;
            continue;
        }
        if arg == "--describe-advanced-hardware-boundary" {
            describe_advanced_hardware_boundary = true;
            continue;
        }
        if arg == "--describe-recall-portability-boundary" {
            describe_recall_portability_boundary = true;
            continue;
        }
        if arg == "--describe-device-supervision-boundary" {
            describe_device_supervision_boundary = true;
            continue;
        }
        if arg == "--describe-clock-topology-boundary" {
            describe_clock_topology_boundary = true;
            continue;
        }
        if arg == "--describe-external-io-boundary" {
            describe_external_io_boundary = true;
            continue;
        }
        if arg == "--describe-media-service-boundary" {
            describe_media_service_boundary = true;
            continue;
        }
        if arg == "--describe-analysis-metadata-boundary" {
            describe_analysis_metadata_boundary = true;
            continue;
        }
        if arg == "--describe-multichannel-boundary" {
            describe_multichannel_boundary = true;
            continue;
        }
        if arg == "--describe-multi-bus-boundary" {
            describe_multi_bus_boundary = true;
            continue;
        }
        if arg == "--describe-sidechain-boundary" {
            describe_sidechain_boundary = true;
            continue;
        }
        if arg == "--describe-complex-io-boundary" {
            describe_complex_io_boundary = true;
            continue;
        }
        if arg == "--describe-spatial-boundary" {
            describe_spatial_boundary = true;
            continue;
        }
        if arg == "--describe-stretch-boundary" {
            describe_stretch_boundary = true;
            continue;
        }
        if arg == "--describe-marker-analysis-boundary" {
            describe_marker_analysis_boundary = true;
            continue;
        }
        if arg == "--describe-transform-artifact-boundary" {
            describe_transform_artifact_boundary = true;
            continue;
        }
        if arg == "--describe-preview-transform-boundary" {
            describe_preview_transform_boundary = true;
            continue;
        }
        if arg == "--describe-integrated-acceptance-lane" {
            describe_integrated_acceptance_lane = true;
            continue;
        }
        if arg == "--describe-g07-acceptance-lane" {
            describe_g07_acceptance_lane = true;
            continue;
        }
        if arg == "--describe-device-workflow-acceptance-lane" {
            describe_device_workflow_acceptance_lane = true;
            continue;
        }
        if arg == "--describe-linux-live-acceptance-lane" {
            describe_linux_live_acceptance_lane = true;
            continue;
        }
        if arg == "--describe-immersive-acceptance-lane" {
            describe_immersive_acceptance_lane = true;
            continue;
        }
        if arg == "--describe-control-preview-workflow-acceptance-lane" {
            describe_control_preview_workflow_acceptance_lane = true;
            continue;
        }
        if arg == "--describe-integrated-live-workflow-acceptance-lane" {
            describe_integrated_live_workflow_acceptance_lane = true;
            continue;
        }
        if arg == "--describe-g06-soak-lane" {
            describe_g06_soak_lane = true;
            continue;
        }
        if arg == "--describe-host-edge-boundary" {
            describe_host_edge_boundary = true;
            continue;
        }
        if arg == "--describe-release-boundary" {
            describe_release_boundary = true;
            continue;
        }
        if arg == "--describe-packaging-manifest" {
            describe_packaging_manifest = true;
            continue;
        }
        if arg == "--describe-downstream-automation" {
            describe_downstream_automation = true;
            continue;
        }
        if arg == "--describe-downstream-fail-gates" {
            describe_downstream_fail_gates = true;
            continue;
        }
        if arg == "--describe-generation-closeout" {
            describe_generation_closeout = true;
            continue;
        }
        if let Some(value) = arg.strip_prefix("--format=") {
            format = OutputFormat::parse(value)?;
            continue;
        }
        positional.push(arg);
    }

    let describe_mode_count = [
        describe_export,
        describe_conformance_matrix,
        describe_interruption_boundary,
        describe_fault_diagnostic_boundary,
        describe_critical_path_boundary,
        describe_block_timing_boundary,
        describe_deferred_work_policy_boundary,
        describe_recording_continuity_boundary,
        describe_offline_render_continuity_boundary,
        describe_plugin_continuity_boundary,
        describe_vst3_boundary,
        describe_au_boundary,
        describe_lv2_boundary,
        describe_cross_adapter_parity_boundary,
        describe_linux_plugin_parity_boundary,
        describe_linux_audio_backend_boundary,
        describe_linux_live_ownership_boundary,
        describe_pipewire_alsa_parity_boundary,
        describe_linux_backend_clock_topology_boundary,
        describe_external_midi_boundary,
        describe_generic_event_boundary,
        describe_controller_expression_boundary,
        describe_control_surface_boundary,
        describe_advanced_hardware_boundary,
        describe_recall_portability_boundary,
        describe_device_supervision_boundary,
        describe_clock_topology_boundary,
        describe_external_io_boundary,
        describe_media_service_boundary,
        describe_analysis_metadata_boundary,
        describe_multichannel_boundary,
        describe_multi_bus_boundary,
        describe_sidechain_boundary,
        describe_complex_io_boundary,
        describe_spatial_boundary,
        describe_stretch_boundary,
        describe_marker_analysis_boundary,
        describe_transform_artifact_boundary,
        describe_preview_transform_boundary,
        describe_integrated_acceptance_lane,
        describe_g07_acceptance_lane,
        describe_device_workflow_acceptance_lane,
        describe_linux_live_acceptance_lane,
        describe_immersive_acceptance_lane,
        describe_control_preview_workflow_acceptance_lane,
        describe_integrated_live_workflow_acceptance_lane,
        describe_g06_soak_lane,
        describe_host_edge_boundary,
        describe_release_boundary,
        describe_packaging_manifest,
        describe_downstream_automation,
        describe_downstream_fail_gates,
        describe_generation_closeout,
    ]
    .into_iter()
    .filter(|enabled| *enabled)
    .count();
    if describe_mode_count > 1 {
        return Err("describe modes are mutually exclusive".into());
    }

    if describe_export {
        if !positional.is_empty() {
            return Err(
                "`--describe-export` does not accept <profile> <scenario> positionals".into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeExport,
        });
    }

    if describe_conformance_matrix {
        if !positional.is_empty() {
            return Err(
                "`--describe-conformance-matrix` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeConformanceMatrix,
        });
    }

    if describe_interruption_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-interruption-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeInterruptionBoundary,
        });
    }

    if describe_fault_diagnostic_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-fault-diagnostic-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeFaultDiagnosticBoundary,
        });
    }

    if describe_critical_path_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-critical-path-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeCriticalPathBoundary,
        });
    }

    if describe_block_timing_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-block-timing-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeBlockTimingBoundary,
        });
    }

    if describe_deferred_work_policy_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-deferred-work-policy-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeDeferredWorkPolicyBoundary,
        });
    }

    if describe_recording_continuity_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-recording-continuity-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeRecordingContinuityBoundary,
        });
    }

    if describe_offline_render_continuity_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-offline-render-continuity-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeOfflineRenderContinuityBoundary,
        });
    }

    if describe_plugin_continuity_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-plugin-continuity-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribePluginContinuityBoundary,
        });
    }

    if describe_vst3_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-vst3-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeVst3Boundary,
        });
    }

    if describe_au_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-au-boundary` does not accept <profile> <scenario> positionals".into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeAuBoundary,
        });
    }

    if describe_lv2_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-lv2-boundary` does not accept <profile> <scenario> positionals".into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeLv2Boundary,
        });
    }

    if describe_cross_adapter_parity_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-cross-adapter-parity-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeCrossAdapterParityBoundary,
        });
    }

    if describe_linux_plugin_parity_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-linux-plugin-parity-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeLinuxPluginParityBoundary,
        });
    }

    if describe_linux_audio_backend_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-linux-audio-backend-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeLinuxAudioBackendBoundary,
        });
    }

    if describe_linux_live_ownership_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-linux-live-ownership-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeLinuxLiveOwnershipBoundary,
        });
    }

    if describe_jack_coordination_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-jack-coordination-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeJackCoordinationBoundary,
        });
    }

    if describe_pipewire_alsa_parity_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-pipewire-alsa-parity-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribePipeWireAlsaParityBoundary,
        });
    }

    if describe_linux_backend_clock_topology_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-linux-backend-clock-topology-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeLinuxBackendClockTopologyBoundary,
        });
    }

    if describe_external_midi_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-external-midi-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeExternalMidiBoundary,
        });
    }

    if describe_generic_event_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-generic-event-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeGenericEventBoundary,
        });
    }

    if describe_controller_expression_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-controller-expression-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeControllerExpressionBoundary,
        });
    }

    if describe_control_surface_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-control-surface-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeControlSurfaceBoundary,
        });
    }

    if describe_advanced_hardware_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-advanced-hardware-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeAdvancedHardwareBoundary,
        });
    }

    if describe_recall_portability_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-recall-portability-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeRecallPortabilityBoundary,
        });
    }

    if describe_device_supervision_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-device-supervision-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeDeviceSupervisionBoundary,
        });
    }

    if describe_clock_topology_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-clock-topology-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeClockTopologyBoundary,
        });
    }

    if describe_external_io_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-external-io-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeExternalIoBoundary,
        });
    }

    if describe_media_service_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-media-service-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeMediaServiceBoundary,
        });
    }

    if describe_analysis_metadata_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-analysis-metadata-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeAnalysisMetadataBoundary,
        });
    }

    if describe_multichannel_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-multichannel-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeMultichannelBoundary,
        });
    }

    if describe_multi_bus_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-multi-bus-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeMultiBusBoundary,
        });
    }

    if describe_sidechain_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-sidechain-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeSidechainBoundary,
        });
    }

    if describe_complex_io_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-complex-io-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeComplexIoBoundary,
        });
    }

    if describe_spatial_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-spatial-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeSpatialBoundary,
        });
    }

    if describe_stretch_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-stretch-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeStretchBoundary,
        });
    }

    if describe_marker_analysis_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-marker-analysis-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeMarkerAnalysisBoundary,
        });
    }

    if describe_transform_artifact_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-transform-artifact-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeTransformArtifactBoundary,
        });
    }

    if describe_preview_transform_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-preview-transform-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribePreviewTransformBoundary,
        });
    }

    if describe_integrated_acceptance_lane {
        if !positional.is_empty() {
            return Err(
                "`--describe-integrated-acceptance-lane` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeIntegratedAcceptanceLane,
        });
    }

    if describe_g07_acceptance_lane {
        if !positional.is_empty() {
            return Err(
                "`--describe-g07-acceptance-lane` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeG07AcceptanceLane,
        });
    }

    if describe_device_workflow_acceptance_lane {
        if !positional.is_empty() {
            return Err(
                "`--describe-device-workflow-acceptance-lane` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeDeviceWorkflowAcceptanceLane,
        });
    }

    if describe_linux_live_acceptance_lane {
        if !positional.is_empty() {
            return Err(
                "`--describe-linux-live-acceptance-lane` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeLinuxLiveAcceptanceLane,
        });
    }

    if describe_immersive_acceptance_lane {
        if !positional.is_empty() {
            return Err(
                "`--describe-immersive-acceptance-lane` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeImmersiveAcceptanceLane,
        });
    }

    if describe_control_preview_workflow_acceptance_lane {
        if !positional.is_empty() {
            return Err(
                "`--describe-control-preview-workflow-acceptance-lane` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeControlPreviewWorkflowAcceptanceLane,
        });
    }

    if describe_integrated_live_workflow_acceptance_lane {
        if !positional.is_empty() {
            return Err(
                "`--describe-integrated-live-workflow-acceptance-lane` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeIntegratedLiveWorkflowAcceptanceLane,
        });
    }

    if describe_g06_soak_lane {
        if !positional.is_empty() {
            return Err(
                "`--describe-g06-soak-lane` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeG06SoakLane,
        });
    }

    if describe_host_edge_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-host-edge-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeHostEdgeBoundary,
        });
    }

    if describe_release_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-release-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeReleaseBoundary,
        });
    }

    if describe_downstream_automation {
        if !positional.is_empty() {
            return Err(
                "`--describe-downstream-automation` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeDownstreamAutomation,
        });
    }

    if describe_downstream_fail_gates {
        if !positional.is_empty() {
            return Err(
                "`--describe-downstream-fail-gates` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeDownstreamFailGates,
        });
    }

    if describe_generation_closeout {
        if !positional.is_empty() {
            return Err(
                "`--describe-generation-closeout` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeGenerationCloseout,
        });
    }

    if describe_packaging_manifest {
        if !positional.is_empty() {
            return Err(
                "`--describe-packaging-manifest` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribePackagingManifest,
        });
    }

    if positional.len() != 2 {
        return Err("expected <profile> <scenario>".into());
    }

    Ok(CliArgs {
        format,
        debug,
        mode: CliMode::Run {
            profile: HostProfile::parse(&positional[0])?,
            scenario: Scenario::parse(&positional[1])?,
        },
    })
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
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

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
        render_device_workflow_acceptance_lane_text, render_downstream_automation_json,
        render_downstream_automation_text, render_downstream_fail_gates_json,
        render_downstream_fail_gates_text, render_export_description_json,
        render_export_description_text, render_external_io_boundary_json,
        render_external_io_boundary_text, render_external_midi_boundary_json,
        render_external_midi_boundary_text, render_fault_diagnostic_boundary_json,
        render_fault_diagnostic_boundary_text, render_g06_soak_lane_json,
        render_g06_soak_lane_text, render_g07_acceptance_lane_json,
        render_g07_acceptance_lane_text, render_generation_closeout_json,
        render_generation_closeout_text, render_generic_event_boundary_json,
        render_generic_event_boundary_text, render_host_edge_boundary_json,
        render_host_edge_boundary_text, render_immersive_acceptance_lane_json,
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
        render_offline_render_continuity_boundary_text, render_packaging_manifest_json,
        render_packaging_manifest_text, render_pipewire_alsa_parity_boundary_json,
        render_pipewire_alsa_parity_boundary_text, render_plugin_continuity_boundary_json,
        render_plugin_continuity_boundary_text, render_preview_transform_boundary_json,
        render_preview_transform_boundary_text, render_recall_portability_boundary_json,
        render_recall_portability_boundary_text, render_recording_continuity_boundary_json,
        render_recording_continuity_boundary_text, render_release_boundary_json,
        render_release_boundary_text, render_sidechain_boundary_json,
        render_sidechain_boundary_text, render_spatial_boundary_json, render_spatial_boundary_text,
        render_stretch_boundary_json, render_stretch_boundary_text, render_supervisor_export_json,
        render_transform_artifact_boundary_json, render_transform_artifact_boundary_text,
        render_vst3_boundary_json, render_vst3_boundary_text, CliArgs, CliMode, ExportDebugOptions,
        HostProfile, HostSummaryDebugSection, OutputFormat, Scenario,
    };
    use signal_graph::{GraphNodeExecutionClass, GraphNodeTopologyRole, GraphStageSpec};
    use signal_hardware::{
        AudioSampleFormat, BackendHealth, HardwareBackendIdentity, HardwareDiagnosticsSnapshot,
        HardwareLifecycleContract, HardwareLifecycleOwnership, HardwareRestartPolicy,
        LinuxAudioBackendKind,
    };
    use signal_host_local::host::{
        LocalAudioPumpSummary, LocalAudioStreamState, LocalAudioTransferPolicy,
        LocalExecutionSummary, LocalFaultSummary, LocalHardwareSummary, LocalPayloadSummary,
        LocalTransportSummary,
    };
    use signal_host_local::{LocalRuntimeHostSummary, RecoveryRestartIntent};
    use signal_plugin::{
        CompletionState, PluginFeature, PluginFormat, PluginIoLayout, PluginLifecycleContract,
        PluginProcessingContract, PluginStateContract, WatchdogTriggerReason,
    };
    use signal_primitives::{ChannelCount, ChannelLayout};
    use signal_runtime::{
        BlockDispatchStage, BrokerFailureStage, BrokerInvalidationStage, CompletionSlotStage,
        HandshakeRequest, HeartbeatCycleStage, PluginSandboxLifecycleStage, PluginSandboxSpec,
        PluginSandboxTransportStage, PluginScanRequest, RuntimeConfig, RuntimeConfigRequest,
        RuntimeEvent, RuntimeEventRecorder, RuntimeEventSink, RuntimeExecutionTopologySummary,
        RuntimeHostAudioPumpSummary, RuntimeHostAudioStreamState, RuntimeHostAudioTransferPolicy,
        RuntimeHostClockDiscontinuityState, RuntimeHostClockDomain, RuntimeHostClockDriftState,
        RuntimeHostClockFallbackState, RuntimeHostClockSource, RuntimeHostClockTransitionState,
        RuntimeHostClockingSummary, RuntimeHostDuplexMismatchState, RuntimeHostEndpointTopology,
        RuntimeHostHardwareSummary, RuntimeHostIoSummary, RuntimeHostLatencySummary,
        RuntimeHostLifecycleOwnership, RuntimeHostRestartPolicy, RuntimeLifecycleApi,
        RuntimeLinuxAudioBackendClockingParityBand, RuntimeLinuxAudioBackendDuplexParityState,
        RuntimeLinuxAudioBackendEndpointTopologyParityState, RuntimeMediaAssetRegistration,
        RuntimeObservationApi, RuntimeOfflineRenderPurgeRequest, RuntimeOfflineRenderRequest,
        RuntimePluginComplexIoSummary, RuntimePluginDiscoveredTypeRecord,
        RuntimePluginFormatPlatformCoverageRecord, RuntimePluginHostPlatform,
        RuntimePluginIsolationOutcome, RuntimePluginParityBand, RuntimeProjectionApi,
        RuntimeSupervisorReport, RuntimeWatchdogTrigger, SafeModeRequest,
        SandboxOperationFailureStage, SignalRuntime, StopReason, TransportDispatchState,
        TransportHeartbeatFreshness, TransportSessionState, WatchdogRestartRecord,
    };

    fn sample_discovered_type_record() -> RuntimePluginDiscoveredTypeRecord {
        RuntimePluginDiscoveredTypeRecord {
            plugin_type_id: "plugin:clap:export-consumer".into(),
            plugin_id: "com.signal.export-consumer".into(),
            vendor: "Signal".into(),
            name: "Signal Export Consumer".into(),
            format: PluginFormat::Clap,
            version: Some("1.0.0".into()),
            features: vec![PluginFeature::AudioEffect, PluginFeature::Utility],
            default_io_layout: PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 1,
            },
            default_multichannel_io: signal_runtime::RuntimeMultichannelIoSummary::for_plugin_io(
                PluginIoLayout {
                    audio_inputs: 2,
                    audio_outputs: 2,
                    midi_inputs: 1,
                    midi_outputs: 1,
                },
            ),
            complex_io_summary: RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
                &[PluginFeature::AudioEffect, PluginFeature::Utility],
                PluginIoLayout {
                    audio_inputs: 2,
                    audio_outputs: 2,
                    midi_inputs: 1,
                    midi_outputs: 1,
                },
            ),
            audio_bus_count: 2,
            parameter_count: 12,
            state_contract: PluginStateContract {
                supports_snapshot: true,
                supports_reset: true,
                supports_bypass: true,
                exposes_latency: true,
                exposes_tail: true,
            },
            processing_contract: PluginProcessingContract {
                max_block_frames: 4096,
                sample_accurate_automation: true,
                accepts_midi: true,
                accepts_note_events: true,
                supports_note_expression: true,
                produces_midi: true,
                silence_aware: true,
            },
            lifecycle_contract: PluginLifecycleContract {
                requires_main_thread_for_state: false,
                supports_prepare: true,
                supports_activate: true,
                supports_reset_while_active: true,
            },
            lv2_extension_capabilities: None,
            summary: "supervisor export discovered plugin".into(),
        }
    }

    fn sample_backend_breadth_record() -> RuntimePluginDiscoveredTypeRecord {
        RuntimePluginDiscoveredTypeRecord {
            plugin_type_id: "plugin:vst3:export-instrument".into(),
            plugin_id: "com.signal.export-instrument".into(),
            vendor: "Signal".into(),
            name: "Signal Export Instrument".into(),
            format: PluginFormat::Vst3,
            version: Some("2.0.0".into()),
            features: vec![PluginFeature::Instrument, PluginFeature::Analyzer],
            default_io_layout: PluginIoLayout {
                audio_inputs: 0,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 0,
            },
            default_multichannel_io: signal_runtime::RuntimeMultichannelIoSummary::for_plugin_io(
                PluginIoLayout {
                    audio_inputs: 0,
                    audio_outputs: 2,
                    midi_inputs: 1,
                    midi_outputs: 0,
                },
            ),
            complex_io_summary: RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
                &[PluginFeature::Instrument, PluginFeature::Analyzer],
                PluginIoLayout {
                    audio_inputs: 0,
                    audio_outputs: 2,
                    midi_inputs: 1,
                    midi_outputs: 0,
                },
            ),
            audio_bus_count: 1,
            parameter_count: 24,
            state_contract: PluginStateContract {
                supports_snapshot: false,
                supports_reset: true,
                supports_bypass: false,
                exposes_latency: false,
                exposes_tail: true,
            },
            processing_contract: PluginProcessingContract {
                max_block_frames: 2048,
                sample_accurate_automation: false,
                accepts_midi: true,
                accepts_note_events: true,
                supports_note_expression: true,
                produces_midi: false,
                silence_aware: false,
            },
            lifecycle_contract: PluginLifecycleContract {
                requires_main_thread_for_state: true,
                supports_prepare: true,
                supports_activate: false,
                supports_reset_while_active: false,
            },
            lv2_extension_capabilities: None,
            summary: "supervisor export backend breadth plugin".into(),
        }
    }

    fn sample_au_breadth_record() -> RuntimePluginDiscoveredTypeRecord {
        RuntimePluginDiscoveredTypeRecord {
            plugin_type_id: "plugin:au:export-au".into(),
            plugin_id: "com.signal.export-au".into(),
            vendor: "Signal".into(),
            name: "Signal Export AU".into(),
            format: PluginFormat::Au,
            version: Some("1.0.0".into()),
            features: vec![PluginFeature::Instrument, PluginFeature::Analyzer],
            default_io_layout: PluginIoLayout {
                audio_inputs: 0,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 0,
            },
            default_multichannel_io: signal_runtime::RuntimeMultichannelIoSummary::for_plugin_io(
                PluginIoLayout {
                    audio_inputs: 0,
                    audio_outputs: 2,
                    midi_inputs: 1,
                    midi_outputs: 0,
                },
            ),
            complex_io_summary: RuntimePluginComplexIoSummary::from_plugin_features_and_layout(
                &[PluginFeature::Instrument, PluginFeature::Analyzer],
                PluginIoLayout {
                    audio_inputs: 0,
                    audio_outputs: 2,
                    midi_inputs: 1,
                    midi_outputs: 0,
                },
            ),
            audio_bus_count: 1,
            parameter_count: 10,
            state_contract: PluginStateContract {
                supports_snapshot: true,
                supports_reset: true,
                supports_bypass: true,
                exposes_latency: false,
                exposes_tail: true,
            },
            processing_contract: PluginProcessingContract {
                max_block_frames: 2048,
                sample_accurate_automation: false,
                accepts_midi: true,
                accepts_note_events: true,
                supports_note_expression: true,
                produces_midi: false,
                silence_aware: false,
            },
            lifecycle_contract: PluginLifecycleContract {
                requires_main_thread_for_state: true,
                supports_prepare: true,
                supports_activate: true,
                supports_reset_while_active: false,
            },
            lv2_extension_capabilities: None,
            summary: "supervisor export au breadth plugin".into(),
        }
    }

    fn sample_integrated_acceptance_host_io() -> RuntimeHostIoSummary {
        RuntimeHostIoSummary {
            hardware: RuntimeHostHardwareSummary {
                backend_identity: HardwareBackendIdentity::CoreAudio,
                backend_name: "coreaudio".into(),
                linux_backend_identity: RuntimeHostHardwareSummary::classify_linux_backend_identity(
                    HardwareBackendIdentity::CoreAudio,
                ),
                linux_backend_portability:
                    RuntimeHostHardwareSummary::classify_linux_backend_portability(
                        HardwareBackendIdentity::CoreAudio,
                        false,
                        BackendHealth::Healthy,
                        0,
                        0,
                        0,
                    ),
                device_id: "device:integrated-acceptance".into(),
                device_name: "Integrated Acceptance Device".into(),
                sample_rate: 48_000,
                buffer_size: 256,
                input_channels: 0,
                output_channels: 2,
                sample_format: AudioSampleFormat::F32,
                simulated: false,
                backend_health: BackendHealth::Healthy,
                xrun_count: 0,
                callback_overrun_count: 0,
                device_loss_count: 0,
                restart_attempt_count: 0,
                restart_failure_count: 0,
            },
            audio_pump: RuntimeHostAudioPumpSummary {
                stream_state: RuntimeHostAudioStreamState::Running,
                transfer_policy: RuntimeHostAudioTransferPolicy {
                    max_callback_frames: 256,
                    max_transfer_channels: 2,
                    zero_fill_unwritten_output: true,
                },
                callback_count: 12,
                total_callback_frames: 3_072,
                total_runtime_output_frames: 3_072,
                copied_output_samples: 6_144,
                zero_filled_output_samples: 0,
                dropped_output_samples: 0,
                last_callback_output_peak: Some(0.35),
                last_runtime_graph_id: Some("graph:integrated-acceptance".into()),
            },
            clocking: RuntimeHostClockingSummary {
                clock_source: RuntimeHostClockSource::Internal,
                ownership: RuntimeHostLifecycleOwnership::HostDrivenCallback,
                restart_policy: RuntimeHostRestartPolicy::HostMustRestart,
                processing_sample_rate_hz: 44_100,
                hardware_sample_rate_hz: 48_000,
                clock_domain: RuntimeHostClockDomain::CrossClock,
                fallback_state: RuntimeHostClockFallbackState::RuntimeResampled,
                transition_state: RuntimeHostClockTransitionState::EnteredCrossClockFallback,
                drift_state: RuntimeHostClockDriftState::CrossClockManaged,
                discontinuity_state: RuntimeHostClockDiscontinuityState::Reconfigured,
                duplex_mismatch_state: RuntimeHostDuplexMismatchState::CrossClockDiverged,
                endpoint_topology: RuntimeHostEndpointTopology::Duplex,
                linux_clocking_parity: RuntimeLinuxAudioBackendClockingParityBand::Unsupported,
                linux_duplex_parity: RuntimeLinuxAudioBackendDuplexParityState::Unsupported,
                linux_endpoint_topology_parity:
                    RuntimeLinuxAudioBackendEndpointTopologyParityState::Unsupported,
                partial_availability: false,
                crossing_required: true,
                callback_interval_ms: 5.333,
            },
            latency: RuntimeHostLatencySummary {
                input_latency_samples: Some(128),
                output_latency_samples: 256,
                round_trip_latency_samples: Some(384),
                graph_latency_samples: 24,
                estimated_output_latency_samples: 280,
                estimated_round_trip_latency_samples: Some(408),
                output_latency_ms: 5.333,
                graph_latency_ms: 0.5,
                estimated_output_latency_ms: 5.833,
                estimated_round_trip_latency_ms: Some(8.5),
            },
            runtime_graph_id_matches_pump: true,
        }
    }

    fn integrated_acceptance_media_fixture_path(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic enough for media fixture paths")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "signal-supervisor-tools-integrated-{label}-{}-{unique}.wav",
            std::process::id()
        ))
    }

    fn write_integrated_acceptance_test_wav(path: &Path) {
        let channels = 1u16;
        let sample_rate = 48_000u32;
        let bits_per_sample = 16u16;
        let frame_count = 128u32;
        let block_align = channels * (bits_per_sample / 8);
        let byte_rate = sample_rate * block_align as u32;
        let data_size = frame_count * block_align as u32;
        let riff_size = 36 + data_size;
        let mut bytes = Vec::with_capacity((44 + data_size) as usize);
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&riff_size.to_le_bytes());
        bytes.extend_from_slice(b"WAVE");
        bytes.extend_from_slice(b"fmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&channels.to_le_bytes());
        bytes.extend_from_slice(&sample_rate.to_le_bytes());
        bytes.extend_from_slice(&byte_rate.to_le_bytes());
        bytes.extend_from_slice(&block_align.to_le_bytes());
        bytes.extend_from_slice(&bits_per_sample.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&data_size.to_le_bytes());
        for index in 0..frame_count {
            let sample =
                (((index as f32 / (frame_count - 1) as f32) * 2.0) - 1.0) * i16::MAX as f32 * 0.5;
            bytes.extend_from_slice(&(sample as i16).to_le_bytes());
        }
        fs::write(path, bytes).expect("integrated acceptance media fixture should be written");
    }

    fn write_g07_acceptance_transient_wav(path: &Path) {
        let channels = 1u16;
        let sample_rate = 48_000u32;
        let bits_per_sample = 16u16;
        let frame_count = 48_000u32;
        let block_align = channels * (bits_per_sample / 8);
        let byte_rate = sample_rate * block_align as u32;
        let data_size = frame_count * block_align as u32;
        let riff_size = 36 + data_size;
        let mut bytes = Vec::with_capacity((44 + data_size) as usize);
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&riff_size.to_le_bytes());
        bytes.extend_from_slice(b"WAVE");
        bytes.extend_from_slice(b"fmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&channels.to_le_bytes());
        bytes.extend_from_slice(&sample_rate.to_le_bytes());
        bytes.extend_from_slice(&byte_rate.to_le_bytes());
        bytes.extend_from_slice(&block_align.to_le_bytes());
        bytes.extend_from_slice(&bits_per_sample.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&data_size.to_le_bytes());
        for frame in 0..frame_count {
            let sample = if frame % 6_000 == 0 { i16::MAX } else { 0 };
            bytes.extend_from_slice(&sample.to_le_bytes());
        }
        fs::write(path, bytes).expect("g07 acceptance transient wav should be written");
    }

    fn sample_g07_acceptance_host_io() -> RuntimeHostIoSummary {
        let linux_backend_identity = RuntimeHostHardwareSummary::classify_linux_backend_identity(
            HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa),
        );
        RuntimeHostIoSummary {
            hardware: RuntimeHostHardwareSummary {
                backend_identity: HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa),
                backend_name: "alsa".into(),
                linux_backend_identity,
                linux_backend_portability:
                    RuntimeHostHardwareSummary::classify_linux_backend_portability(
                        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa),
                        false,
                        BackendHealth::Healthy,
                        0,
                        0,
                        0,
                    ),
                device_id: "alsa:g07-integrated".into(),
                device_name: "ALSA Integrated Acceptance Device".into(),
                sample_rate: 48_000,
                buffer_size: 256,
                input_channels: 2,
                output_channels: 2,
                sample_format: AudioSampleFormat::F32,
                simulated: false,
                backend_health: BackendHealth::Healthy,
                xrun_count: 0,
                callback_overrun_count: 0,
                device_loss_count: 0,
                restart_attempt_count: 0,
                restart_failure_count: 0,
            },
            audio_pump: RuntimeHostAudioPumpSummary {
                stream_state: RuntimeHostAudioStreamState::Running,
                transfer_policy: RuntimeHostAudioTransferPolicy {
                    max_callback_frames: 256,
                    max_transfer_channels: 2,
                    zero_fill_unwritten_output: true,
                },
                callback_count: 16,
                total_callback_frames: 4_096,
                total_runtime_output_frames: 4_096,
                copied_output_samples: 8_192,
                zero_filled_output_samples: 0,
                dropped_output_samples: 0,
                last_callback_output_peak: Some(0.42),
                last_runtime_graph_id: Some("graph:g07-integrated-acceptance".into()),
            },
            clocking: RuntimeHostClockingSummary {
                clock_source: RuntimeHostClockSource::Internal,
                ownership: RuntimeHostLifecycleOwnership::HostDrivenCallback,
                restart_policy: RuntimeHostRestartPolicy::HostMustRestart,
                processing_sample_rate_hz: 48_000,
                hardware_sample_rate_hz: 48_000,
                clock_domain: RuntimeHostClockDomain::SameClock,
                fallback_state: RuntimeHostClockFallbackState::Direct,
                transition_state: RuntimeHostClockTransitionState::Stable,
                drift_state: RuntimeHostClockDriftState::Stable,
                discontinuity_state: RuntimeHostClockDiscontinuityState::Continuous,
                duplex_mismatch_state: RuntimeHostDuplexMismatchState::Aligned,
                endpoint_topology: RuntimeHostEndpointTopology::Duplex,
                linux_clocking_parity: RuntimeHostIoSummary::classify_linux_clocking_parity(
                    linux_backend_identity,
                    BackendHealth::Healthy,
                    RuntimeHostAudioStreamState::Running,
                    RuntimeHostClockDomain::SameClock,
                    RuntimeHostClockFallbackState::Direct,
                    RuntimeHostClockTransitionState::Stable,
                    RuntimeHostClockDriftState::Stable,
                    RuntimeHostClockDiscontinuityState::Continuous,
                ),
                linux_duplex_parity: RuntimeHostIoSummary::classify_linux_duplex_parity(
                    linux_backend_identity,
                    BackendHealth::Healthy,
                    RuntimeHostAudioStreamState::Running,
                    RuntimeHostClockDomain::SameClock,
                    RuntimeHostClockFallbackState::Direct,
                    RuntimeHostClockTransitionState::Stable,
                    RuntimeHostDuplexMismatchState::Aligned,
                    RuntimeHostEndpointTopology::Duplex,
                    false,
                ),
                linux_endpoint_topology_parity:
                    RuntimeHostIoSummary::classify_linux_endpoint_topology_parity(
                        linux_backend_identity,
                        BackendHealth::Healthy,
                        RuntimeHostClockTransitionState::Stable,
                        RuntimeHostClockDiscontinuityState::Continuous,
                        RuntimeHostDuplexMismatchState::Aligned,
                        RuntimeHostEndpointTopology::Duplex,
                        false,
                    ),
                partial_availability: false,
                crossing_required: false,
                callback_interval_ms: 5.333,
            },
            latency: RuntimeHostLatencySummary {
                input_latency_samples: Some(128),
                output_latency_samples: 128,
                round_trip_latency_samples: Some(256),
                graph_latency_samples: 24,
                estimated_output_latency_samples: 152,
                estimated_round_trip_latency_samples: Some(280),
                output_latency_ms: 2.667,
                graph_latency_ms: 0.5,
                estimated_output_latency_ms: 3.167,
                estimated_round_trip_latency_ms: Some(5.833),
            },
            runtime_graph_id_matches_pump: true,
        }
    }

    fn sample_g07_external_midi_snapshot(
    ) -> signal_runtime::RuntimeExternalMidiEndpointGraphSnapshot {
        signal_runtime::RuntimeExternalMidiEndpointGraphSnapshot {
            discovery_state: signal_runtime::RuntimeExternalMidiDiscoveryState::Enumerated,
            graph_state: signal_runtime::RuntimeExternalMidiGraphState::Stable,
            live_ownership:
                signal_runtime::RuntimeExternalMidiLiveOwnershipSummary::detached_without_backend_context(),
            provider_name: "signal-host-local".into(),
            device_count: 1,
            endpoint_count: 1,
            input_endpoint_count: 1,
            output_endpoint_count: 1,
            duplex_endpoint_count: 1,
            active_route_count: 1,
            guarded_route_count: 0,
            devices: vec![signal_runtime::RuntimeExternalMidiDeviceDescriptor {
                device_id: "device:controller:main".into(),
                device_name: "Signal Controller".into(),
                lifecycle_state: signal_runtime::RuntimeExternalMidiDeviceLifecycleState::Discovered,
                endpoint_count: 1,
                summary: "device Signal Controller lifecycle=Discovered endpoints=1".into(),
            }],
            endpoints: vec![signal_runtime::RuntimeExternalMidiEndpointDescriptor {
                endpoint_id: "endpoint:controller:duplex".into(),
                endpoint_name: "Signal Controller Duplex".into(),
                device_id: "device:controller:main".into(),
                direction: signal_runtime::RuntimeExternalMidiEndpointDirection::Duplex,
                lifecycle_state: signal_runtime::RuntimeExternalMidiEndpointLifecycleState::Active,
                route_state: signal_runtime::RuntimeExternalMidiRouteState::DuplexObserved,
                capability: signal_runtime::RuntimeExternalMidiEndpointCapabilitySummary {
                    supports_bounded_midi_input: true,
                    supports_bounded_midi_output: true,
                    supports_transport_clock: true,
                    supports_note_events: true,
                    supports_controller_events: true,
                    supports_note_pressure_expression: true,
                    supports_note_timbre_expression: true,
                    supports_note_tuning_expression: false,
                    supports_mpe: true,
                    midi2_posture:
                        signal_runtime::RuntimeControllerExpressionMidi2Posture::Guarded,
                    control_surface_guarded: false,
                    summary: "midi-input=true midi-output=true transport-clock=true note-events=true controller-events=true pressure=true timbre=true tuning=false mpe=true midi2=Guarded control-surface=portable".into(),
                },
                summary: "endpoint Signal Controller Duplex direction=Duplex route=DuplexObserved lifecycle=Active".into(),
            }],
            summary: "discovery=Ready graph=Ready provider=signal-host-local devices=1 endpoints=1 routes=1".into(),
        }
    }

    fn sample_control_preview_workflow_external_midi_snapshot(
    ) -> signal_runtime::RuntimeExternalMidiEndpointGraphSnapshot {
        let capability = signal_runtime::RuntimeExternalMidiEndpointCapabilitySummary {
            supports_bounded_midi_input: true,
            supports_bounded_midi_output: true,
            supports_transport_clock: true,
            supports_note_events: true,
            supports_controller_events: true,
            supports_note_pressure_expression: true,
            supports_note_timbre_expression: false,
            supports_note_tuning_expression: false,
            supports_mpe: false,
            midi2_posture: signal_runtime::RuntimeControllerExpressionMidi2Posture::Unsupported,
            control_surface_guarded: true,
            summary: "midi-input=true midi-output=true transport-clock=true note-events=true controller-events=true pressure=true timbre=false tuning=false mpe=false midi2=Unsupported control-surface=guarded".into(),
        };
        signal_runtime::RuntimeExternalMidiEndpointGraphSnapshot {
            discovery_state: signal_runtime::RuntimeExternalMidiDiscoveryState::Enumerated,
            graph_state: signal_runtime::RuntimeExternalMidiGraphState::Stable,
            live_ownership:
                signal_runtime::RuntimeExternalMidiLiveOwnershipSummary::detached_without_backend_context(),
            provider_name: "public-control-preview-workflow".into(),
            device_count: 1,
            endpoint_count: 2,
            input_endpoint_count: 1,
            output_endpoint_count: 1,
            duplex_endpoint_count: 1,
            active_route_count: 1,
            guarded_route_count: 1,
            devices: vec![signal_runtime::RuntimeExternalMidiDeviceDescriptor {
                device_id: "device:control-preview-workflow:1".into(),
                device_name: "Control Preview Workflow Surface".into(),
                lifecycle_state:
                    signal_runtime::RuntimeExternalMidiDeviceLifecycleState::Discovered,
                endpoint_count: 2,
                summary: "device=Control Preview Workflow Surface endpoints=2".into(),
            }],
            endpoints: vec![
                signal_runtime::RuntimeExternalMidiEndpointDescriptor {
                    endpoint_id: "endpoint:control-preview-workflow:input".into(),
                    endpoint_name: "Control Preview Workflow Input".into(),
                    device_id: "device:control-preview-workflow:1".into(),
                    direction: signal_runtime::RuntimeExternalMidiEndpointDirection::Input,
                    lifecycle_state:
                        signal_runtime::RuntimeExternalMidiEndpointLifecycleState::Active,
                    route_state: signal_runtime::RuntimeExternalMidiRouteState::InputObserved,
                    capability: capability.clone(),
                    summary: "input".into(),
                },
                signal_runtime::RuntimeExternalMidiEndpointDescriptor {
                    endpoint_id: "endpoint:control-preview-workflow:output".into(),
                    endpoint_name: "Control Preview Workflow Output".into(),
                    device_id: "device:control-preview-workflow:1".into(),
                    direction: signal_runtime::RuntimeExternalMidiEndpointDirection::Output,
                    lifecycle_state:
                        signal_runtime::RuntimeExternalMidiEndpointLifecycleState::Active,
                    route_state: signal_runtime::RuntimeExternalMidiRouteState::OutputObserved,
                    capability,
                    summary: "output".into(),
                },
            ],
            summary: "provider=public-control-preview-workflow state=Stable devices=1 endpoints=2 routes=1 guarded-routes=1".into(),
        }
    }

    fn sample_local_summary() -> LocalRuntimeHostSummary {
        LocalRuntimeHostSummary {
            backend_name: "coreaudio",
            hardware: LocalHardwareSummary {
                device_id: "coreaudio:default-output".into(),
                device_name: "CoreAudio Default Output".into(),
                sample_rate: 48_000,
                buffer_size: 512,
                input_channels: 0,
                output_channels: 2,
                sample_format: AudioSampleFormat::F32,
                lifecycle: HardwareLifecycleContract {
                    ownership: HardwareLifecycleOwnership::HostDrivenCallback,
                    restart_policy: HardwareRestartPolicy::HostMustRestart,
                },
                simulated: false,
                backend_diagnostics: HardwareDiagnosticsSnapshot::healthy(),
            },
            audio_pump: LocalAudioPumpSummary {
                stream_state: LocalAudioStreamState::Running,
                transfer_policy: LocalAudioTransferPolicy {
                    max_callback_frames: 512,
                    max_transfer_channels: 2,
                    zero_fill_unwritten_output: true,
                },
                callback_count: 3,
                last_callback_index: Some(2),
                total_callback_frames: 1536,
                total_runtime_output_frames: 1536,
                copied_output_samples: 3072,
                zero_filled_output_samples: 0,
                dropped_output_samples: 0,
                last_callback_output_peak: Some(0.8),
                last_runtime_graph_id: Some("signal.host.local.demo".into()),
            },
            scan_roots: vec!["/plugins".into()],
            execution: LocalExecutionSummary {
                control_requests: 4,
                control_responses: 4,
                heartbeat_responses: 2,
                processed_blocks: 3,
                engine_processed_blocks: 3,
                last_control_message: "activateInstance".into(),
                last_completion_state: CompletionState::Completed,
                last_block_sequence: 7,
                last_engine_graph_id: Some("signal.host.local.demo".into()),
                last_engine_output_peak: Some(0.8),
                last_engine_output_rms: Some(0.42),
                processing_epoch: 2,
                restart_count: 1,
                teardown_count: 1,
                last_recovery_intent: Some(RecoveryRestartIntent::WatchdogRecovery),
                last_stop_reason: Some(StopReason::DegradedModeRecovery),
                last_plugin_state: None,
            },
            transport: LocalTransportSummary {
                sandbox_id: "sandbox-1".into(),
                shared_memory_lease_id: "lease-1".into(),
                shared_memory_region_id: "region-1".into(),
                shared_memory_path: "/tmp/signal-region-1".into(),
                shared_memory_bytes: 4096,
            },
            topology: RuntimeExecutionTopologySummary::default(),
            plugin_dispatch: None,
            last_payload: LocalPayloadSummary {
                event_count: 6,
                parameter_event_count: 2,
                parameter_gesture_event_count: 2,
                parameter_modulation_event_count: 1,
                note_event_count: 1,
                note_expression_event_count: 1,
                midi_event_count: 1,
                generated_event_bytes: 128,
                first_output_sample: Some(0.5),
            },
            faults: LocalFaultSummary {
                deadline_misses: 1,
                heartbeat_misses: 0,
                watchdog_triggered: true,
                watchdog_trigger_reason: Some(WatchdogTriggerReason::DeadlineMisses),
            },
        }
    }

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
    fn parse_args_supports_short_json_flag() {
        assert_eq!(
            parse_args(["--json".into(), "server".into(), "soak".into()]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::Run {
                    profile: HostProfile::Server,
                    scenario: Scenario::Soak,
                },
            })
        );
    }

    #[test]
    fn parse_args_supports_include_payload_flag() {
        assert_eq!(
            parse_args([
                "--json".into(),
                "--include-payload".into(),
                "local".into(),
                "default".into(),
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: true },
                mode: CliMode::Run {
                    profile: HostProfile::Local,
                    scenario: Scenario::Default,
                },
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_export_mode() {
        assert_eq!(
            parse_args(["--format=json".into(), "--describe-export".into()]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeExport,
            })
        );
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_export() {
        let error =
            parse_args(["--describe-export".into(), "local".into(), "default".into()]).unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_supports_describe_conformance_matrix_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-conformance-matrix".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeConformanceMatrix,
            })
        );
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_conformance_matrix() {
        let error = parse_args([
            "--describe-conformance-matrix".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_supports_describe_interruption_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-interruption-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeInterruptionBoundary,
            })
        );
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_interruption_boundary() {
        let error = parse_args([
            "--describe-interruption-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_supports_describe_fault_diagnostic_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-fault-diagnostic-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeFaultDiagnosticBoundary,
            })
        );
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_fault_diagnostic_boundary() {
        let error = parse_args([
            "--describe-fault-diagnostic-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_supports_describe_critical_path_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-critical-path-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeCriticalPathBoundary,
            })
        );
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_critical_path_boundary() {
        let error = parse_args([
            "--describe-critical-path-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_supports_describe_block_timing_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-block-timing-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeBlockTimingBoundary,
            })
        );
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_block_timing_boundary() {
        let error = parse_args([
            "--describe-block-timing-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_supports_describe_deferred_work_policy_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-deferred-work-policy-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeDeferredWorkPolicyBoundary,
            })
        );
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_deferred_work_policy_boundary() {
        let error = parse_args([
            "--describe-deferred-work-policy-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_supports_describe_recording_continuity_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-recording-continuity-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeRecordingContinuityBoundary,
            })
        );
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_recording_continuity_boundary() {
        let error = parse_args([
            "--describe-recording-continuity-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_supports_describe_offline_render_continuity_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-offline-render-continuity-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeOfflineRenderContinuityBoundary,
            })
        );
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_offline_render_continuity_boundary() {
        let error = parse_args([
            "--describe-offline-render-continuity-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_supports_describe_plugin_continuity_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-plugin-continuity-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribePluginContinuityBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_vst3_boundary_mode() {
        assert_eq!(
            parse_args(["--format=json".into(), "--describe-vst3-boundary".into()]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeVst3Boundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_au_boundary_mode() {
        assert_eq!(
            parse_args(["--format=json".into(), "--describe-au-boundary".into()]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeAuBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_lv2_boundary_mode() {
        assert_eq!(
            parse_args(["--format=json".into(), "--describe-lv2-boundary".into()]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeLv2Boundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_cross_adapter_parity_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-cross-adapter-parity-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeCrossAdapterParityBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_linux_plugin_parity_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-linux-plugin-parity-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeLinuxPluginParityBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_linux_audio_backend_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-linux-audio-backend-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeLinuxAudioBackendBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_linux_live_ownership_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-linux-live-ownership-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeLinuxLiveOwnershipBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_jack_coordination_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-jack-coordination-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeJackCoordinationBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_pipewire_alsa_parity_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-pipewire-alsa-parity-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribePipeWireAlsaParityBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_linux_backend_clock_topology_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-linux-backend-clock-topology-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeLinuxBackendClockTopologyBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_external_midi_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-external-midi-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeExternalMidiBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_generic_event_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-generic-event-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeGenericEventBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_controller_expression_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-controller-expression-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeControllerExpressionBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_control_surface_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-control-surface-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeControlSurfaceBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_advanced_hardware_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-advanced-hardware-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeAdvancedHardwareBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_recall_portability_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-recall-portability-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeRecallPortabilityBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_device_supervision_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-device-supervision-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeDeviceSupervisionBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_clock_topology_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-clock-topology-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeClockTopologyBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_external_io_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-external-io-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeExternalIoBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_media_service_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-media-service-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeMediaServiceBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_analysis_metadata_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-analysis-metadata-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeAnalysisMetadataBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_multichannel_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-multichannel-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeMultichannelBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_multi_bus_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-multi-bus-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeMultiBusBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_sidechain_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-sidechain-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeSidechainBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_complex_io_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-complex-io-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeComplexIoBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_spatial_boundary_mode() {
        assert_eq!(
            parse_args(["--format=json".into(), "--describe-spatial-boundary".into()]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeSpatialBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_stretch_boundary_mode() {
        assert_eq!(
            parse_args(["--format=json".into(), "--describe-stretch-boundary".into()]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeStretchBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_marker_analysis_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-marker-analysis-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeMarkerAnalysisBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_transform_artifact_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-transform-artifact-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeTransformArtifactBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_preview_transform_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-preview-transform-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribePreviewTransformBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_integrated_acceptance_lane_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-integrated-acceptance-lane".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeIntegratedAcceptanceLane,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_g07_acceptance_lane_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-g07-acceptance-lane".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeG07AcceptanceLane,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_device_workflow_acceptance_lane_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-device-workflow-acceptance-lane".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeDeviceWorkflowAcceptanceLane,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_linux_live_acceptance_lane_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-linux-live-acceptance-lane".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeLinuxLiveAcceptanceLane,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_immersive_acceptance_lane_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-immersive-acceptance-lane".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeImmersiveAcceptanceLane,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_control_preview_workflow_acceptance_lane_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-control-preview-workflow-acceptance-lane".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeControlPreviewWorkflowAcceptanceLane,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_integrated_live_workflow_acceptance_lane_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-integrated-live-workflow-acceptance-lane".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeIntegratedLiveWorkflowAcceptanceLane,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_g06_soak_lane_mode() {
        assert_eq!(
            parse_args(["--format=json".into(), "--describe-g06-soak-lane".into()]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeG06SoakLane,
            })
        );
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_device_supervision_boundary() {
        let error = parse_args([
            "--describe-device-supervision-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_clock_topology_boundary() {
        let error = parse_args([
            "--describe-clock-topology-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_external_io_boundary() {
        let error = parse_args([
            "--describe-external-io-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_controller_expression_boundary() {
        let error = parse_args([
            "--describe-controller-expression-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_control_surface_boundary() {
        let error = parse_args([
            "--describe-control-surface-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_advanced_hardware_boundary() {
        let error = parse_args([
            "--describe-advanced-hardware-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_media_service_boundary() {
        let error = parse_args([
            "--describe-media-service-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_analysis_metadata_boundary() {
        let error = parse_args([
            "--describe-analysis-metadata-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_multichannel_boundary() {
        let error = parse_args([
            "--describe-multichannel-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_multi_bus_boundary() {
        let error = parse_args([
            "--describe-multi-bus-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_sidechain_boundary() {
        let error = parse_args([
            "--describe-sidechain-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_complex_io_boundary() {
        let error = parse_args([
            "--describe-complex-io-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_spatial_boundary() {
        let error = parse_args([
            "--describe-spatial-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_stretch_boundary() {
        let error = parse_args([
            "--describe-stretch-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_marker_analysis_boundary() {
        let error = parse_args([
            "--describe-marker-analysis-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_transform_artifact_boundary() {
        let error = parse_args([
            "--describe-transform-artifact-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_preview_transform_boundary() {
        let error = parse_args([
            "--describe-preview-transform-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_integrated_acceptance_lane() {
        let error = parse_args([
            "--describe-integrated-acceptance-lane".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_g07_acceptance_lane() {
        let error = parse_args([
            "--describe-g07-acceptance-lane".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_device_workflow_acceptance_lane() {
        let error = parse_args([
            "--describe-device-workflow-acceptance-lane".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_linux_live_acceptance_lane() {
        let error = parse_args([
            "--describe-linux-live-acceptance-lane".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_immersive_acceptance_lane() {
        let error = parse_args([
            "--describe-immersive-acceptance-lane".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_control_preview_workflow_acceptance_lane() {
        let error = parse_args([
            "--describe-control-preview-workflow-acceptance-lane".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_integrated_live_workflow_acceptance_lane() {
        let error = parse_args([
            "--describe-integrated-live-workflow-acceptance-lane".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_g06_soak_lane() {
        let error = parse_args([
            "--describe-g06-soak-lane".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_plugin_continuity_boundary() {
        let error = parse_args([
            "--describe-plugin-continuity-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_rejects_multiple_describe_modes() {
        let error = parse_args([
            "--describe-export".into(),
            "--describe-conformance-matrix".into(),
        ])
        .unwrap_err();
        assert!(error.contains("mutually exclusive"));
    }

    #[test]
    fn parse_args_supports_describe_host_edge_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-host-edge-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeHostEdgeBoundary,
            })
        );
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_host_edge_boundary() {
        let error = parse_args([
            "--describe-host-edge-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_supports_describe_release_boundary_mode() {
        assert_eq!(
            parse_args(["--format=json".into(), "--describe-release-boundary".into()]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeReleaseBoundary,
            })
        );
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_release_boundary() {
        let error = parse_args([
            "--describe-release-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_supports_describe_packaging_manifest_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-packaging-manifest".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribePackagingManifest,
            })
        );
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_packaging_manifest() {
        let error = parse_args([
            "--describe-packaging-manifest".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_supports_describe_downstream_automation_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-downstream-automation".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeDownstreamAutomation,
            })
        );
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_downstream_automation() {
        let error = parse_args([
            "--describe-downstream-automation".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_supports_describe_downstream_fail_gates_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-downstream-fail-gates".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeDownstreamFailGates,
            })
        );
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_downstream_fail_gates() {
        let error = parse_args([
            "--describe-downstream-fail-gates".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_supports_describe_generation_closeout_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-generation-closeout".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeGenerationCloseout,
            })
        );
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_generation_closeout() {
        let error = parse_args([
            "--describe-generation-closeout".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
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
        assert!(rendered.contains("interruption_boundary: signal.runtime.interruption-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:interruption-boundary"));
        assert!(rendered.contains("surface: RuntimeObservationReport::fault_status"));
        assert!(rendered.contains("surface: RuntimeDeferredServiceReceipt::interruption_class"));
        assert!(rendered.contains("surface: supervisor_report() -> RuntimeSupervisorReport"));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_interruption_boundary_reports_restartable_runtime_state"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-interruption-boundary --format=json"
        ));
    }

    #[test]
    fn interruption_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_interruption_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.interruption-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/012-runtime-interruption-taxonomy-and-resumability-contract.md\""
        ));
        assert!(
            rendered.contains("\"acceptance_task\":\"effigy acceptance:interruption-boundary\"")
        );
        assert!(rendered.contains("\"id\":\"runtime-fault-status\""));
        assert!(rendered.contains("\"id\":\"offline-render-execution-interruption-receipt\""));
        assert!(rendered.contains("\"id\":\"shared-host-supervisor-report\""));
        assert!(rendered.contains("\"id\":\"runtime-resumable-deferred-proof\""));
    }

    #[test]
    fn fault_diagnostic_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_fault_diagnostic_boundary_text();
        assert!(rendered
            .contains("fault_diagnostic_boundary: signal.runtime.fault-diagnostic-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:fault-diagnostic-boundary"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::fault_diagnostic_receipt and RuntimeSupervisorReport::observation.fault_diagnostic_receipt"
        ));
        assert!(rendered.contains("surface: RuntimeProfilingReceipt::fault_diagnostic_receipt"));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_fault_diagnostic_boundary_reports_canonical_runtime_receipts"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-fault-diagnostic-boundary --format=json"
        ));
    }

    #[test]
    fn fault_diagnostic_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_fault_diagnostic_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.fault-diagnostic-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/016-runtime-fault-cause-attribution-and-diagnostic-receipt-contract.md\""
        ));
        assert!(rendered
            .contains("\"acceptance_task\":\"effigy acceptance:fault-diagnostic-boundary\""));
        assert!(rendered.contains("\"id\":\"runtime-observation-fault-diagnostic\""));
        assert!(rendered.contains("\"id\":\"runtime-profiling-fault-diagnostic\""));
        assert!(rendered.contains("\"id\":\"shared-host-fault-diagnostic-report\""));
        assert!(rendered.contains("\"id\":\"runtime-public-fault-diagnostic-proof\""));
    }

    #[test]
    fn critical_path_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_critical_path_boundary_text();
        assert!(rendered.contains("critical_path_boundary: signal.runtime.critical-path-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:critical-path-boundary"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::performance_snapshot() and RuntimeSupervisorReport::performance_snapshot()"
        ));
        assert!(rendered.contains("surface: RuntimePerformanceTraceReceipt"));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_critical_path_boundary_reports_bounded_hotspot_receipts"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-critical-path-boundary --format=json"
        ));
    }

    #[test]
    fn critical_path_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_critical_path_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.critical-path-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/018-graph-critical-path-hot-node-and-worker-lane-instrumentation-contract.md\""
        ));
        assert!(
            rendered.contains("\"acceptance_task\":\"effigy acceptance:critical-path-boundary\"")
        );
        assert!(rendered.contains("\"id\":\"runtime-performance-hotspot-report\""));
        assert!(rendered.contains("\"id\":\"runtime-performance-trace-digest\""));
        assert!(rendered.contains("\"id\":\"shared-host-critical-path-report\""));
        assert!(rendered.contains("\"id\":\"runtime-public-critical-path-proof\""));
    }

    #[test]
    fn block_timing_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_block_timing_boundary_text();
        assert!(rendered.contains("block_timing_boundary: signal.runtime.block-timing-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:block-timing-boundary"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::engine_block_snapshot and RuntimeSupervisorReport::observation.engine_block_snapshot"
        ));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::performance_snapshot(), RuntimeSupervisorReport::performance_snapshot(), and RuntimePerformanceTraceReceipt"
        ));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_block_timing_boundary_reports_bounded_runtime_measurements"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-block-timing-boundary --format=json"
        ));
    }

    #[test]
    fn block_timing_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_block_timing_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.block-timing-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/017-per-block-execution-timing-and-pressure-snapshot-contract.md\""
        ));
        assert!(
            rendered.contains("\"acceptance_task\":\"effigy acceptance:block-timing-boundary\"")
        );
        assert!(rendered.contains("\"id\":\"runtime-engine-block-snapshot\""));
        assert!(rendered.contains("\"id\":\"runtime-performance-digests\""));
        assert!(rendered.contains("\"id\":\"shared-host-block-timing-report\""));
        assert!(rendered.contains("\"id\":\"runtime-public-block-timing-proof\""));
    }

    #[test]
    fn deferred_work_policy_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_deferred_work_policy_boundary_text();
        assert!(rendered.contains(
            "deferred_work_policy_boundary: signal.runtime.deferred-work-policy-boundary"
        ));
        assert!(
            rendered.contains("acceptance_task: effigy acceptance:deferred-work-policy-boundary")
        );
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::last_deferred_service_receipt and RuntimeSupervisorReport::observation.last_deferred_service_receipt"
        ));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::performance_snapshot(), RuntimeSupervisorReport::performance_snapshot(), and RuntimePerformanceTraceReceipt"
        ));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_deferred_work_policy_boundary_reports_runtime_owned_scheduler_receipts"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-deferred-work-policy-boundary --format=json"
        ));
    }

    #[test]
    fn deferred_work_policy_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_deferred_work_policy_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.deferred-work-policy-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/019-deferred-work-scheduler-priority-backpressure-and-cancellation-contract.md\""
        ));
        assert!(rendered
            .contains("\"acceptance_task\":\"effigy acceptance:deferred-work-policy-boundary\""));
        assert!(rendered.contains("\"id\":\"runtime-deferred-service-policy-receipt\""));
        assert!(rendered.contains("\"id\":\"runtime-performance-policy-digests\""));
        assert!(rendered.contains("\"id\":\"shared-host-deferred-policy-report\""));
        assert!(rendered.contains("\"id\":\"runtime-public-deferred-policy-proof\""));
    }

    #[test]
    fn recording_continuity_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_recording_continuity_boundary_text();
        assert!(rendered.contains(
            "recording_continuity_boundary: signal.runtime.recording-continuity-boundary"
        ));
        assert!(rendered.contains("acceptance_task: effigy acceptance:recording-continuity"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::recording_capture_snapshot and RuntimeSupervisorReport::observation.recording_capture_snapshot"
        ));
        assert!(rendered
            .contains("surface: RuntimeRecordingCaptureCommitReceipt::committed_checkpoint"));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_recording_continuity_boundary_reports_resumable_restartable_and_terminal_states"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-recording-continuity-boundary --format=json"
        ));
    }

    #[test]
    fn recording_continuity_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_recording_continuity_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.recording-continuity-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/013-recording-continuity-midi-capture-and-checkpoint-contract.md\""
        ));
        assert!(rendered.contains("\"acceptance_task\":\"effigy acceptance:recording-continuity\""));
        assert!(rendered.contains("\"id\":\"runtime-recording-capture-snapshot\""));
        assert!(rendered.contains("\"id\":\"runtime-recording-capture-commit-receipt\""));
        assert!(rendered.contains("\"id\":\"shared-host-recording-supervisor-report\""));
        assert!(rendered.contains("\"id\":\"runtime-terminal-capture-proof\""));
    }

    #[test]
    fn offline_render_continuity_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_offline_render_continuity_boundary_text();
        assert!(rendered.contains(
            "offline_render_continuity_boundary: signal.runtime.offline-render-continuity-boundary"
        ));
        assert!(rendered.contains("acceptance_task: effigy acceptance:offline-render-continuity"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::offline_render_session_snapshot and RuntimeSupervisorReport::observation.offline_render_session_snapshot"
        ));
        assert!(rendered
            .contains("surface: RuntimeObservationApi::get_offline_render_session_snapshot()"));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_offline_render_continuity_boundary_reports_resumable_restartable_and_terminal_states"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-offline-render-continuity-boundary --format=json"
        ));
    }

    #[test]
    fn offline_render_continuity_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_offline_render_continuity_boundary_json();
        assert!(
            rendered.contains("\"boundary\":\"signal.runtime.offline-render-continuity-boundary\"")
        );
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/015-offline-render-recovery-and-resumability-contract.md\""
        ));
        assert!(rendered
            .contains("\"acceptance_task\":\"effigy acceptance:offline-render-continuity\""));
        assert!(rendered.contains("\"id\":\"runtime-offline-render-session-snapshot\""));
        assert!(rendered.contains("\"id\":\"runtime-offline-render-observation-api\""));
        assert!(rendered.contains("\"id\":\"shared-host-offline-render-supervisor-report\""));
        assert!(rendered.contains("\"id\":\"runtime-terminal-render-proof\""));
    }

    #[test]
    fn plugin_continuity_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_plugin_continuity_boundary_text();
        assert!(rendered
            .contains("plugin_continuity_boundary: signal.runtime.plugin-continuity-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:plugin-continuity"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::plugin_lifecycle_snapshot and RuntimeSupervisorReport::observation.plugin_lifecycle_snapshot"
        ));
        assert!(rendered.contains("surface: RuntimeObservationApi::get_plugin_chain_snapshot()"));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_plugin_continuity_boundary_reports_shared_boundary_and_policy_truth"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-plugin-continuity-boundary --format=json"
        ));
    }

    #[test]
    fn plugin_continuity_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_plugin_continuity_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.plugin-continuity-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/014-plugin-isolation-policy-transport-rebind-and-shared-sandbox-continuity-contract.md\""
        ));
        assert!(rendered.contains("\"acceptance_task\":\"effigy acceptance:plugin-continuity\""));
        assert!(rendered.contains("\"id\":\"runtime-plugin-lifecycle-snapshot\""));
        assert!(rendered.contains("\"id\":\"runtime-plugin-chain-snapshot\""));
        assert!(rendered.contains("\"id\":\"shared-host-plugin-supervisor-report\""));
        assert!(rendered.contains("\"id\":\"runtime-placement-policy-proof\""));
    }

    #[test]
    fn vst3_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_vst3_boundary_text();
        assert!(rendered.contains("vst3_boundary: signal.runtime.vst3-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:vst3-boundary"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::plugin_discovery_snapshot and RuntimeSupervisorReport::observation.plugin_discovery_snapshot"
        ));
        assert!(
            rendered.contains("surface: RuntimeObservationApi::get_plugin_lifecycle_snapshot()")
        );
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_vst3_boundary_reports_runtime_owned_discovery_and_lifecycle_truth"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-vst3-boundary --format=json"
        ));
    }

    #[test]
    fn vst3_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_vst3_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.vst3-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/020-vst3-adapter-baseline-and-runtime-owned-lifecycle-contract.md\""
        ));
        assert!(rendered.contains("\"acceptance_task\":\"effigy acceptance:vst3-boundary\""));
        assert!(rendered.contains("\"id\":\"runtime-vst3-discovery-report\""));
        assert!(rendered.contains("\"id\":\"runtime-vst3-lifecycle-snapshot\""));
        assert!(rendered.contains("\"id\":\"shared-host-vst3-supervisor-report\""));
        assert!(rendered.contains("\"id\":\"server-host-vst3-proof\""));
    }

    #[test]
    fn au_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_au_boundary_text();
        assert!(rendered.contains("au_boundary: signal.runtime.au-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:au-boundary"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::plugin_discovery_snapshot and RuntimeSupervisorReport::observation.plugin_discovery_snapshot"
        ));
        assert!(
            rendered.contains("surface: RuntimeObservationApi::get_plugin_lifecycle_snapshot()")
        );
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_au_boundary_reports_runtime_owned_discovery_and_lifecycle_truth"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-au-boundary --format=json"
        ));
    }

    #[test]
    fn au_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_au_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.au-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/021-au-adapter-baseline-and-runtime-owned-lifecycle-contract.md\""
        ));
        assert!(rendered.contains("\"acceptance_task\":\"effigy acceptance:au-boundary\""));
        assert!(rendered.contains("\"id\":\"runtime-au-discovery-report\""));
        assert!(rendered.contains("\"id\":\"runtime-au-lifecycle-snapshot\""));
        assert!(rendered.contains("\"id\":\"shared-host-au-supervisor-report\""));
        assert!(rendered.contains("\"id\":\"server-host-au-proof\""));
    }

    #[test]
    fn lv2_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_lv2_boundary_text();
        assert!(rendered.contains("lv2_boundary: signal.runtime.lv2-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:lv2-boundary"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::lv2_extension_snapshot and RuntimeSupervisorReport::observation.lv2_extension_snapshot"
        ));
        assert!(
            rendered.contains("surface: RuntimeObservationApi::get_plugin_lifecycle_snapshot()")
        );
        assert!(rendered.contains("crate: signal-host-local"));
        assert!(rendered.contains("crate: signal-host-server"));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_lv2_boundary_reports_runtime_owned_discovery_and_lifecycle_truth"
        ));
        assert!(rendered.contains(
            "cargo test -p signal-host-local local_shared_host_edge_exports_runtime_lv2_extension_truth"
        ));
        assert!(rendered.contains(
            "cargo test -p signal-host-server server_shared_host_edge_exports_runtime_lv2_extension_truth"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-lv2-boundary --format=json"
        ));
    }

    #[test]
    fn lv2_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_lv2_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.lv2-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/055-lv2-worker-urid-patch-and-extension-negotiation-contract.md\""
        ));
        assert!(rendered.contains("\"acceptance_task\":\"effigy acceptance:lv2-boundary\""));
        assert!(rendered.contains("\"id\":\"runtime-lv2-extension-report\""));
        assert!(rendered.contains("\"id\":\"runtime-lv2-lifecycle-snapshot\""));
        assert!(rendered.contains("\"id\":\"local-host-lv2-supervisor-report\""));
        assert!(rendered.contains("\"id\":\"server-host-lv2-supervisor-report\""));
        assert!(rendered.contains("\"id\":\"local-host-lv2-proof\""));
        assert!(rendered.contains("\"id\":\"server-host-lv2-proof\""));
    }

    #[test]
    fn cross_adapter_parity_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_cross_adapter_parity_boundary_text();
        assert!(rendered.contains(
            "cross_adapter_parity_boundary: signal.runtime.cross-adapter-parity-boundary"
        ));
        assert!(
            rendered.contains("acceptance_task: effigy acceptance:cross-adapter-parity-boundary")
        );
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::plugin_discovery_snapshot and RuntimeSupervisorReport::observation.plugin_discovery_snapshot"
        ));
        assert!(
            rendered.contains("surface: RuntimeObservationApi::get_plugin_lifecycle_snapshot()")
        );
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_cross_adapter_parity_boundary_reports_runtime_owned_portability_truth"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-cross-adapter-parity-boundary --format=json"
        ));
    }

    #[test]
    fn cross_adapter_parity_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_cross_adapter_parity_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.cross-adapter-parity-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/022-backend-capability-parity-linux-plugin-support-and-cross-adapter-conformance-contract.md\""
        ));
        assert!(rendered
            .contains("\"acceptance_task\":\"effigy acceptance:cross-adapter-parity-boundary\""));
        assert!(rendered.contains("\"id\":\"runtime-cross-adapter-discovery-report\""));
        assert!(rendered.contains("\"id\":\"runtime-cross-adapter-lifecycle-snapshot\""));
        assert!(rendered.contains("\"id\":\"shared-host-cross-adapter-supervisor-report\""));
        assert!(rendered.contains("\"id\":\"server-host-cross-adapter-proof\""));
    }

    #[test]
    fn linux_plugin_parity_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_linux_plugin_parity_boundary_text();
        assert!(rendered
            .contains("linux_plugin_parity_boundary: signal.runtime.linux-plugin-parity-boundary"));
        assert!(
            rendered.contains("acceptance_task: effigy acceptance:linux-plugin-parity-boundary")
        );
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::plugin_discovery_snapshot and RuntimeSupervisorReport::observation.plugin_discovery_snapshot"
        ));
        assert!(
            rendered.contains("surface: RuntimeObservationApi::get_plugin_lifecycle_snapshot()")
        );
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_linux_plugin_parity_boundary_reports_runtime_owned_linux_policy_truth"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-linux-plugin-parity-boundary --format=json"
        ));
    }

    #[test]
    fn linux_plugin_parity_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_linux_plugin_parity_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.linux-plugin-parity-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/039-linux-cross-adapter-plugin-parity-and-sandbox-policy-contract.md\""
        ));
        assert!(rendered
            .contains("\"acceptance_task\":\"effigy acceptance:linux-plugin-parity-boundary\""));
        assert!(rendered.contains("\"id\":\"runtime-linux-parity-discovery-report\""));
        assert!(rendered.contains("\"id\":\"runtime-linux-parity-lifecycle-snapshot\""));
        assert!(rendered.contains("\"id\":\"server-host-linux-parity-supervisor-report\""));
        assert!(rendered.contains("\"id\":\"server-host-linux-parity-proof\""));
    }

    #[test]
    fn linux_audio_backend_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_linux_audio_backend_boundary_text();
        assert!(rendered
            .contains("linux_audio_backend_boundary: signal.runtime.linux-audio-backend-boundary"));
        assert!(
            rendered.contains("acceptance_task: effigy acceptance:linux-audio-backend-boundary")
        );
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::external_io_snapshot and RuntimeSupervisorReport::observation.external_io_snapshot"
        ));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_linux_audio_backend_boundary_reports_runtime_owned_backend_identity_truth"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-linux-audio-backend-boundary --format=json"
        ));
    }

    #[test]
    fn linux_audio_backend_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_linux_audio_backend_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.linux-audio-backend-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/040-linux-audio-backend-portability-across-alsa-jack-and-pipewire-contract.md\""
        ));
        assert!(rendered
            .contains("\"acceptance_task\":\"effigy acceptance:linux-audio-backend-boundary\""));
        assert!(rendered.contains("\"id\":\"runtime-linux-audio-observation-report\""));
        assert!(rendered.contains("\"id\":\"server-host-linux-audio-supervisor-report\""));
        assert!(rendered.contains("\"id\":\"server-host-linux-audio-proof\""));
    }

    #[test]
    fn linux_live_ownership_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_linux_live_ownership_boundary_text();
        assert!(rendered.contains(
            "linux_live_ownership_boundary: signal.runtime.linux-live-ownership-boundary"
        ));
        assert!(
            rendered.contains("acceptance_task: effigy acceptance:linux-live-ownership-boundary")
        );
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::linux_backend_session_snapshot and RuntimeSupervisorReport::observation.linux_backend_session_snapshot"
        ));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_linux_live_ownership_boundary_reports_runtime_owned_session_truth"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-linux-live-ownership-boundary --format=json"
        ));
    }

    #[test]
    fn linux_live_ownership_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_linux_live_ownership_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.linux-live-ownership-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/052-live-linux-audio-backend-ownership-and-session-lifecycle-contract.md\""
        ));
        assert!(rendered
            .contains("\"acceptance_task\":\"effigy acceptance:linux-live-ownership-boundary\""));
        assert!(rendered.contains("\"id\":\"runtime-linux-live-session-report\""));
        assert!(rendered.contains("\"id\":\"local-host-linux-live-session-report\""));
        assert!(rendered.contains("\"id\":\"server-host-linux-live-session-report\""));
    }

    #[test]
    fn jack_coordination_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_jack_coordination_boundary_text();
        assert!(rendered
            .contains("jack_coordination_boundary: signal.runtime.jack-coordination-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:jack-coordination-boundary"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::jack_coordination_snapshot and RuntimeSupervisorReport::observation.jack_coordination_snapshot"
        ));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_jack_coordination_boundary_reports_runtime_owned_transport_graph_truth"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-jack-coordination-boundary --format=json"
        ));
    }

    #[test]
    fn jack_coordination_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_jack_coordination_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.jack-coordination-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/053-jack-transport-graph-and-backend-native-coordination-contract.md\""
        ));
        assert!(rendered
            .contains("\"acceptance_task\":\"effigy acceptance:jack-coordination-boundary\""));
        assert!(rendered.contains("\"id\":\"runtime-jack-coordination-report\""));
        assert!(rendered.contains("\"id\":\"runtime-transport-session-report\""));
        assert!(rendered.contains("\"id\":\"shared-host-jack-supervisor-report\""));
    }

    #[test]
    fn pipewire_alsa_parity_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_pipewire_alsa_parity_boundary_text();
        assert!(rendered.contains(
            "pipewire_alsa_parity_boundary: signal.runtime.pipewire-alsa-parity-boundary"
        ));
        assert!(
            rendered.contains("acceptance_task: effigy acceptance:pipewire-alsa-parity-boundary")
        );
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::pipewire_alsa_parity_snapshot and RuntimeSupervisorReport::observation.pipewire_alsa_parity_snapshot"
        ));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_pipewire_alsa_parity_boundary_reports_runtime_owned_claim_and_policy_truth"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-pipewire-alsa-parity-boundary --format=json"
        ));
    }

    #[test]
    fn pipewire_alsa_parity_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_pipewire_alsa_parity_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.pipewire-alsa-parity-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/054-pipewire-and-alsa-session-role-device-claim-and-stream-policy-parity-contract.md\""
        ));
        assert!(rendered
            .contains("\"acceptance_task\":\"effigy acceptance:pipewire-alsa-parity-boundary\""));
        assert!(rendered.contains("\"id\":\"runtime-pipewire-alsa-parity-report\""));
        assert!(rendered.contains("\"id\":\"shared-host-pipewire-alsa-supervisor-report\""));
    }

    #[test]
    fn linux_backend_clock_topology_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_linux_backend_clock_topology_boundary_text();
        assert!(rendered.contains(
            "linux_backend_clock_topology_boundary: signal.runtime.linux-backend-clock-topology-boundary"
        ));
        assert!(rendered
            .contains("acceptance_task: effigy acceptance:linux-backend-clock-topology-boundary"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::external_io_snapshot and RuntimeSupervisorReport::observation.external_io_snapshot"
        ));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_linux_backend_clock_topology_boundary_reports_runtime_owned_linux_parity_truth"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-linux-backend-clock-topology-boundary --format=json"
        ));
    }

    #[test]
    fn linux_backend_clock_topology_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_linux_backend_clock_topology_boundary_json();
        assert!(rendered
            .contains("\"boundary\":\"signal.runtime.linux-backend-clock-topology-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/041-linux-backend-clocking-duplex-and-endpoint-topology-parity-contract.md\""
        ));
        assert!(rendered.contains(
            "\"acceptance_task\":\"effigy acceptance:linux-backend-clock-topology-boundary\""
        ));
        assert!(rendered.contains("\"id\":\"runtime-linux-backend-clock-topology-report\""));
        assert!(rendered.contains("\"id\":\"local-host-linux-backend-clock-topology-report\""));
        assert!(rendered.contains("\"id\":\"server-host-linux-backend-clock-topology-report\""));
    }

    #[test]
    fn external_midi_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_external_midi_boundary_text();
        assert!(rendered.contains("external_midi_boundary: signal.runtime.external-midi-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:external-midi-boundary"));
        assert!(rendered.contains(
            "contract_path: docs/contracts/065-live-external-midi-device-ownership-and-backend-parity-contract.md"
        ));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::external_midi_snapshot and RuntimeSupervisorReport::observation.external_midi_snapshot"
        ));
        assert!(rendered.contains("live_ownership"));
        assert!(rendered.contains("ownership_posture"));
        assert!(rendered.contains("attach_continuity"));
        assert!(rendered.contains("backend_parity"));
        assert!(rendered.contains("guarded_parity_outcome"));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_external_midi_boundary_reports_runtime_owned_endpoint_graph_truth"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-external-midi-boundary --format=json"
        ));
    }

    #[test]
    fn external_midi_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_external_midi_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.external-midi-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/065-live-external-midi-device-ownership-and-backend-parity-contract.md\""
        ));
        assert!(
            rendered.contains("\"acceptance_task\":\"effigy acceptance:external-midi-boundary\"")
        );
        assert!(rendered.contains("\"id\":\"runtime-external-midi-report\""));
        assert!(rendered.contains("\"id\":\"shared-host-external-midi-report\""));
        assert!(rendered.contains("\"id\":\"runtime-external-midi-public-proof\""));
        assert!(rendered.contains("live_ownership"));
        assert!(rendered.contains("ownership_posture"));
        assert!(rendered.contains("attach_continuity"));
        assert!(rendered.contains("backend_parity"));
        assert!(rendered.contains("guarded_parity_outcome"));
    }

    #[test]
    fn generic_event_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_generic_event_boundary_text();
        assert!(rendered.contains("generic_event_boundary: signal.runtime.generic-event-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:generic-event-boundary"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::plugin_event_snapshot and RuntimeSupervisorReport::observation.plugin_event_snapshot"
        ));
        assert!(rendered.contains(
            "surface: RuntimeObservationApi::get_plugin_discovery_snapshot() capability_coverage.supports_note_expression_count"
        ));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_generic_event_boundary_reports_runtime_owned_event_and_capability_truth"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-generic-event-boundary --format=json"
        ));
    }

    #[test]
    fn generic_event_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_generic_event_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.generic-event-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/023-generic-midi-note-expression-and-plugin-event-model-contract.md\""
        ));
        assert!(
            rendered.contains("\"acceptance_task\":\"effigy acceptance:generic-event-boundary\"")
        );
        assert!(rendered.contains("\"id\":\"runtime-generic-event-report\""));
        assert!(rendered.contains("\"id\":\"runtime-generic-event-capability-coverage\""));
        assert!(rendered.contains("\"id\":\"shared-host-generic-event-supervisor-report\""));
    }

    #[test]
    fn controller_expression_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_controller_expression_boundary_text();
        assert!(rendered.contains(
            "controller_expression_boundary: signal.runtime.controller-expression-boundary"
        ));
        assert!(
            rendered.contains("acceptance_task: effigy acceptance:controller-expression-boundary")
        );
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::plugin_event_snapshot and RuntimeSupervisorReport::observation.plugin_event_snapshot"
        ));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::external_midi_snapshot.endpoints[*].capability and RuntimeSupervisorReport::observation.external_midi_snapshot.endpoints[*].capability"
        ));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_controller_expression_boundary_reports_runtime_owned_expression_truth"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-controller-expression-boundary --format=json"
        ));
    }

    #[test]
    fn controller_expression_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_controller_expression_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.controller-expression-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/043-midi-2-0-mpe-and-richer-controller-expression-contract.md\""
        ));
        assert!(rendered
            .contains("\"acceptance_task\":\"effigy acceptance:controller-expression-boundary\""));
        assert!(rendered.contains("\"id\":\"runtime-controller-expression-report\""));
        assert!(rendered.contains("\"id\":\"runtime-controller-expression-device-capability\""));
        assert!(rendered.contains("\"id\":\"shared-host-controller-expression-report\""));
    }

    #[test]
    fn control_surface_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_control_surface_boundary_text();
        assert!(
            rendered.contains("control_surface_boundary: signal.runtime.control-surface-boundary")
        );
        assert!(rendered.contains("acceptance_task: effigy acceptance:control-surface-boundary"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::control_surface_snapshot and RuntimeSupervisorReport::observation.control_surface_snapshot"
        ));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::external_midi_snapshot and RuntimeSupervisorReport::observation.external_midi_snapshot"
        ));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_control_surface_boundary_reports_runtime_owned_transport_and_feedback_truth"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-control-surface-boundary --format=json"
        ));
    }

    #[test]
    fn control_surface_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_control_surface_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.control-surface-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/044-control-surface-transport-mapping-and-feedback-contract.md\""
        ));
        assert!(
            rendered.contains("\"acceptance_task\":\"effigy acceptance:control-surface-boundary\"")
        );
        assert!(rendered.contains("\"id\":\"runtime-control-surface-report\""));
        assert!(rendered.contains("\"id\":\"runtime-control-surface-external-midi-anchor\""));
        assert!(rendered.contains("\"id\":\"shared-host-control-surface-report\""));
    }

    #[test]
    fn advanced_hardware_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_advanced_hardware_boundary_text();
        assert!(rendered
            .contains("advanced_hardware_boundary: signal.runtime.advanced-hardware-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:advanced-hardware-boundary"));
        assert!(rendered.contains(
            "contract_path: docs/contracts/061-control-surface-scene-mapping-feedback-pages-and-safe-action-graph-contract.md"
        ));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::advanced_hardware_snapshot and RuntimeSupervisorReport::observation.advanced_hardware_snapshot"
        ));
        assert!(rendered.contains("display_transport_device_count"));
        assert!(rendered.contains("motor_transport_device_count"));
        assert!(rendered.contains("haptic_transport_device_count"));
        assert!(rendered.contains("scene_mapping_device_count"));
        assert!(rendered.contains("feedback_page_device_count"));
        assert!(rendered.contains("safe_action_graph_device_count"));
        assert!(rendered.contains("display_transport_posture"));
        assert!(rendered.contains("scene_mapping_posture"));
        assert!(rendered.contains("feedback_page_posture"));
        assert!(rendered.contains("feedback_page_class"));
        assert!(rendered.contains("safe_action_graph_posture"));
        assert!(rendered.contains("action_authority"));
        assert!(rendered.contains("safe_action_outcome"));
        assert!(rendered.contains("feedback_outcome"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::control_surface_snapshot and RuntimeSupervisorReport::observation.control_surface_snapshot"
        ));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_advanced_hardware_boundary_reports_runtime_owned_policy_and_feedback_truth"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-advanced-hardware-boundary --format=json"
        ));
    }

    #[test]
    fn advanced_hardware_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_advanced_hardware_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.advanced-hardware-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/061-control-surface-scene-mapping-feedback-pages-and-safe-action-graph-contract.md\""
        ));
        assert!(rendered
            .contains("\"acceptance_task\":\"effigy acceptance:advanced-hardware-boundary\""));
        assert!(rendered.contains("\"id\":\"runtime-advanced-hardware-report\""));
        assert!(rendered.contains("\"id\":\"runtime-advanced-hardware-control-surface-anchor\""));
        assert!(rendered.contains("\"id\":\"shared-host-advanced-hardware-report\""));
        assert!(rendered.contains("display_transport_device_count"));
        assert!(rendered.contains("motor_transport_device_count"));
        assert!(rendered.contains("haptic_transport_device_count"));
        assert!(rendered.contains("scene_mapping_device_count"));
        assert!(rendered.contains("feedback_page_device_count"));
        assert!(rendered.contains("safe_action_graph_device_count"));
        assert!(rendered.contains("display_content_class"));
        assert!(rendered.contains("scene_mapping_posture"));
        assert!(rendered.contains("feedback_page_posture"));
        assert!(rendered.contains("feedback_page_class"));
        assert!(rendered.contains("safe_action_graph_posture"));
        assert!(rendered.contains("action_authority"));
        assert!(rendered.contains("safe_action_outcome"));
        assert!(rendered.contains("feedback_outcome"));
    }

    #[test]
    fn recall_portability_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_recall_portability_boundary_text();
        assert!(rendered
            .contains("recall_portability_boundary: signal.runtime.recall-portability-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:recall-portability-boundary"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::plugin_chain_snapshot and RuntimeSupervisorReport::observation.plugin_chain_snapshot"
        ));
        assert!(rendered
            .contains("surface: RuntimeObservationApi::get_plugin_recall_handoff_snapshot()"));
        assert!(rendered.contains(
            "cargo test -p signal-runtime --test public_contract_boundary public_runtime_recall_interchange_and_ara_context_truth_is_consumable_from_reexports"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-recall-portability-boundary --format=json"
        ));
    }

    #[test]
    fn recall_portability_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_recall_portability_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.recall-portability-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/024-plugin-preset-state-interchange-portable-recall-and-ara-context-contract.md\""
        ));
        assert!(rendered
            .contains("\"acceptance_task\":\"effigy acceptance:recall-portability-boundary\""));
        assert!(rendered.contains("\"id\":\"runtime-plugin-chain-recall-report\""));
        assert!(rendered.contains("\"id\":\"runtime-plugin-recall-handoff\""));
        assert!(rendered.contains("\"id\":\"shared-host-recall-supervisor-report\""));
        assert!(rendered.contains("\"id\":\"runtime-recall-portability-public-proof\""));
    }

    #[test]
    fn device_supervision_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_device_supervision_boundary_text();
        assert!(rendered
            .contains("device_supervision_boundary: signal.runtime.device-supervision-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:device-supervision-boundary"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::device_supervision_snapshot and RuntimeSupervisorReport::observation.device_supervision_snapshot"
        ));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::fault_status and RuntimeObservationReport::interruption_summary"
        ));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_device_supervision_boundary_reports_recovering_and_faulted_runtime_states"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-device-supervision-boundary --format=json"
        ));
    }

    #[test]
    fn device_supervision_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_device_supervision_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.device-supervision-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/025-device-supervision-restart-state-machine-and-fault-boundary-contract.md\""
        ));
        assert!(rendered
            .contains("\"acceptance_task\":\"effigy acceptance:device-supervision-boundary\""));
        assert!(rendered.contains("\"id\":\"runtime-device-supervision-report\""));
        assert!(rendered.contains("\"id\":\"runtime-supervision-fault-alignment\""));
        assert!(rendered.contains("\"id\":\"shared-host-device-supervision-supervisor-report\""));
        assert!(rendered.contains("\"id\":\"runtime-device-supervision-public-proof\""));
    }

    #[test]
    fn clock_topology_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_clock_topology_boundary_text();
        assert!(
            rendered.contains("clock_topology_boundary: signal.runtime.clock-topology-boundary")
        );
        assert!(rendered.contains("acceptance_task: effigy acceptance:clock-topology-boundary"));
        assert!(rendered.contains(
            "surface: RuntimeHostObservationReport::host_io and RuntimeHostSupervisorReport::observation.host_io"
        ));
        assert!(rendered.contains(
            "surface: LocalRuntimeHost::host_supervisor_report() -> RuntimeHostSupervisorReport"
        ));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_clock_topology_boundary_reports_drift_duplex_and_endpoint_receipts"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-clock-topology-boundary --format=json"
        ));
    }

    #[test]
    fn clock_topology_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_clock_topology_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.clock-topology-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/026-clock-domain-drift-duplex-mismatch-and-endpoint-topology-contract.md\""
        ));
        assert!(
            rendered.contains("\"acceptance_task\":\"effigy acceptance:clock-topology-boundary\"")
        );
        assert!(rendered.contains("\"id\":\"runtime-host-clocking-report\""));
        assert!(rendered.contains("\"id\":\"runtime-external-io-alignment\""));
        assert!(rendered.contains("\"id\":\"shared-local-host-clock-topology-report\""));
        assert!(rendered.contains("\"id\":\"runtime-clock-topology-public-proof\""));
    }

    #[test]
    fn external_io_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_external_io_boundary_text();
        assert!(rendered.contains("external_io_boundary: signal.runtime.external-io-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:external-io-boundary"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::external_io_snapshot and RuntimeSupervisorReport::observation.external_io_snapshot"
        ));
        assert!(rendered.contains(
            "surface: ServerRuntimeHost::supervisor_report() -> RuntimeSupervisorReport"
        ));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_external_io_boundary_reports_runtime_owned_monitor_and_loopback_truth"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-external-io-boundary --format=json"
        ));
    }

    #[test]
    fn external_io_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_external_io_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.external-io-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/027-external-io-monitoring-tap-point-and-loopback-measurement-contract.md\""
        ));
        assert!(rendered.contains("\"acceptance_task\":\"effigy acceptance:external-io-boundary\""));
        assert!(rendered.contains("\"id\":\"runtime-external-io-report\""));
        assert!(rendered.contains("\"id\":\"runtime-host-external-io-report\""));
        assert!(rendered.contains("\"id\":\"shared-local-host-external-io-report\""));
        assert!(rendered.contains("\"id\":\"shared-server-host-external-io-report\""));
        assert!(rendered.contains("\"id\":\"runtime-external-io-public-proof\""));
    }

    #[test]
    fn media_service_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_media_service_boundary_text();
        assert!(rendered.contains("media_service_boundary: signal.runtime.media-service-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:media-service-boundary"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::media_pipeline_snapshot, RuntimeObservationReport::media_service_snapshot, and RuntimeSupervisorReport::observation.{media_pipeline_snapshot,media_service_snapshot}"
        ));
        assert!(rendered.contains("surface: supervisor_report() -> RuntimeSupervisorReport"));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_media_service_boundary_reports_runtime_owned_readiness_and_invalidation_truth"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-media-service-boundary --format=json"
        ));
    }

    #[test]
    fn media_service_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_media_service_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.media-service-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/028-media-indexing-waveform-analysis-and-preview-service-contract.md\""
        ));
        assert!(
            rendered.contains("\"acceptance_task\":\"effigy acceptance:media-service-boundary\"")
        );
        assert!(rendered.contains("\"id\":\"runtime-media-service-report\""));
        assert!(rendered.contains("\"id\":\"runtime-media-service-snapshot\""));
        assert!(rendered.contains("\"id\":\"shared-host-media-service-report\""));
        assert!(rendered.contains("\"id\":\"runtime-media-service-public-proof\""));
    }

    #[test]
    fn analysis_metadata_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_analysis_metadata_boundary_text();
        assert!(rendered
            .contains("analysis_metadata_boundary: signal.runtime.analysis-metadata-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:analysis-metadata-boundary"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::media_library_snapshot and RuntimeSupervisorReport::observation.media_library_snapshot"
        ));
        assert!(rendered
            .contains("surface: RuntimeObservationApi::get_media_library_service_snapshot()"));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_analysis_metadata_boundary_reports_runtime_owned_library_descriptors"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-analysis-metadata-boundary --format=json"
        ));
    }

    #[test]
    fn analysis_metadata_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_analysis_metadata_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.analysis-metadata-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/029-analysis-metadata-extraction-and-library-service-contract.md\""
        ));
        assert!(rendered
            .contains("\"acceptance_task\":\"effigy acceptance:analysis-metadata-boundary\""));
        assert!(rendered.contains("\"id\":\"runtime-analysis-metadata-report\""));
        assert!(rendered.contains("\"id\":\"runtime-analysis-metadata-snapshot\""));
        assert!(rendered.contains("\"id\":\"shared-host-analysis-metadata-report\""));
        assert!(rendered.contains("\"id\":\"runtime-analysis-metadata-public-proof\""));
    }

    #[test]
    fn multichannel_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_multichannel_boundary_text();
        assert!(rendered.contains("multichannel_boundary: signal.runtime.multichannel-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:multichannel-boundary"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::execution_topology_summary, RuntimeObservationReport::external_io_snapshot, and RuntimeSupervisorReport::observation.{execution_topology_summary,external_io_snapshot}"
        ));
        assert!(
            rendered.contains("surface: RuntimeObservationApi::get_plugin_discovery_snapshot()")
        );
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_multichannel_boundary_reports_runtime_owned_layout_and_role_truth"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-multichannel-boundary --format=json"
        ));
    }

    #[test]
    fn multichannel_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_multichannel_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.multichannel-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/032-canonical-multichannel-layout-and-channel-role-contract.md\""
        ));
        assert!(
            rendered.contains("\"acceptance_task\":\"effigy acceptance:multichannel-boundary\"")
        );
        assert!(rendered.contains("\"id\":\"runtime-multichannel-topology-report\""));
        assert!(rendered.contains("\"id\":\"runtime-multichannel-plugin-discovery-snapshot\""));
        assert!(rendered.contains("\"id\":\"shared-host-multichannel-report\""));
        assert!(rendered.contains("\"id\":\"runtime-multichannel-public-proof\""));
    }

    #[test]
    fn multi_bus_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_multi_bus_boundary_text();
        assert!(rendered.contains("multi_bus_boundary: signal.runtime.multi-bus-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:multi-bus-boundary"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::execution_topology_summary, RuntimeObservationReport::metering_snapshot, and RuntimeSupervisorReport::observation.{execution_topology_summary,metering_snapshot}"
        ));
        assert!(rendered.contains("surface: RuntimeOfflineRenderContractPreview::chain_contract"));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_multi_bus_boundary_reports_runtime_owned_connection_truth"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-multi-bus-boundary --format=json"
        ));
    }

    #[test]
    fn multi_bus_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_multi_bus_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.multi-bus-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/034-multi-bus-graph-execution-and-auxiliary-topology-contract.md\""
        ));
        assert!(rendered.contains("\"acceptance_task\":\"effigy acceptance:multi-bus-boundary\""));
        assert!(rendered.contains("\"id\":\"runtime-multi-bus-topology-report\""));
        assert!(rendered.contains("\"id\":\"runtime-multi-bus-render-contract-preview\""));
        assert!(rendered.contains("\"id\":\"shared-host-multi-bus-report\""));
        assert!(rendered.contains("\"id\":\"runtime-multi-bus-public-proof\""));
    }

    #[test]
    fn sidechain_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_sidechain_boundary_text();
        assert!(rendered.contains("sidechain_boundary: signal.runtime.sidechain-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:sidechain-boundary"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::execution_topology_summary, RuntimeSupervisorReport::observation.{execution_topology_summary,plugin_chain_snapshot}, and RuntimeOfflineRenderContractPreview::chain_contract"
        ));
        assert!(rendered.contains("surface: GraphNodeBufferContractProjection::secondary_input"));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_sidechain_boundary_reports_runtime_owned_secondary_input_truth"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-sidechain-boundary --format=json"
        ));
    }

    #[test]
    fn sidechain_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_sidechain_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.sidechain-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/033-sidechain-routing-and-secondary-input-execution-contract.md\""
        ));
        assert!(rendered.contains("\"acceptance_task\":\"effigy acceptance:sidechain-boundary\""));
        assert!(rendered.contains("\"id\":\"runtime-sidechain-topology-report\""));
        assert!(rendered.contains("\"id\":\"runtime-sidechain-contract-projection\""));
        assert!(rendered.contains("\"id\":\"shared-host-sidechain-report\""));
        assert!(rendered.contains("\"id\":\"runtime-sidechain-public-proof\""));
    }

    #[test]
    fn complex_io_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_complex_io_boundary_text();
        assert!(rendered.contains("complex_io_boundary: signal.runtime.complex-io-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:complex-io-boundary"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::plugin_discovery_snapshot and RuntimeSupervisorReport::observation.plugin_discovery_snapshot"
        ));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::plugin_pin_matrix_snapshot and RuntimeSupervisorReport::observation.plugin_pin_matrix_snapshot"
        ));
        assert!(rendered.contains("surface: RuntimeOfflineRenderContractPreview::chain_contract"));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_complex_io_boundary_reports_runtime_owned_topology_truth"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-complex-io-boundary --format=json"
        ));
    }

    #[test]
    fn complex_io_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_complex_io_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.complex-io-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/056-complex-plugin-pin-matrix-and-dynamic-bus-negotiation-contract.md\""
        ));
        assert!(rendered.contains("\"acceptance_task\":\"effigy acceptance:complex-io-boundary\""));
        assert!(rendered.contains("\"id\":\"runtime-complex-io-discovery-report\""));
        assert!(rendered.contains("\"id\":\"runtime-plugin-pin-matrix-report\""));
        assert!(rendered.contains("\"id\":\"runtime-complex-io-plugin-chain-snapshot\""));
        assert!(rendered.contains("\"id\":\"runtime-complex-io-render-contract-preview\""));
        assert!(rendered.contains("\"id\":\"shared-host-complex-io-report\""));
        assert!(rendered.contains("\"id\":\"runtime-complex-io-public-proof\""));
    }

    #[test]
    fn spatial_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_spatial_boundary_text();
        assert!(rendered.contains("spatial_boundary: signal.runtime.spatial-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:spatial-boundary"));
        assert!(rendered.contains(
            "contract_path: docs/contracts/059-renderer-capability-negotiation-and-immersive-export-contract.md"
        ));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::execution_topology_summary and RuntimeSupervisorReport::observation.execution_topology_summary"
        ));
        assert!(rendered.contains("immersive_spatial_node_count"));
        assert!(rendered.contains("deployment_spatial_node_count"));
        assert!(rendered.contains("fallback_monitoring_scene_spatial_node_count"));
        assert!(rendered.contains("renderer_capability_spatial_node_count"));
        assert!(rendered.contains("immersive_export_spatial_node_count"));
        assert!(rendered.contains(
            "spatial_execution.{immersive_room_policy,deployment_monitoring,renderer_export}"
        ));
        assert!(rendered.contains("fallback_room_policy_spatial_stage_count"));
        assert!(rendered.contains("deployment_spatial_stage_count"));
        assert!(rendered.contains("fallback_monitoring_scene_spatial_stage_count"));
        assert!(rendered.contains("renderer_capability_spatial_stage_count"));
        assert!(rendered.contains("fallback_immersive_export_spatial_stage_count"));
        assert!(rendered.contains("surface: RuntimeOfflineRenderContractPreview::chain_contract"));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_spatial_boundary_reports_runtime_owned_execution_truth"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-spatial-boundary --format=json"
        ));
    }

    #[test]
    fn spatial_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_spatial_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.spatial-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/059-renderer-capability-negotiation-and-immersive-export-contract.md\""
        ));
        assert!(rendered.contains("\"acceptance_task\":\"effigy acceptance:spatial-boundary\""));
        assert!(rendered.contains("\"id\":\"runtime-spatial-topology-report\""));
        assert!(rendered.contains("\"id\":\"runtime-spatial-plugin-chain-snapshot\""));
        assert!(rendered.contains("\"id\":\"runtime-spatial-render-contract-preview\""));
        assert!(rendered.contains("\"id\":\"shared-host-spatial-report\""));
        assert!(rendered.contains("\"id\":\"runtime-spatial-public-proof\""));
        assert!(rendered.contains("immersive_spatial_stage_count"));
        assert!(rendered.contains("fallback_room_policy_spatial_node_count"));
        assert!(rendered.contains("deployment_spatial_node_count"));
        assert!(rendered.contains("folded_down_spatial_stage_count"));
        assert!(rendered.contains("fallback_monitoring_scene_spatial_stage_count"));
        assert!(rendered.contains("renderer_capability_spatial_node_count"));
        assert!(rendered.contains("immersive_export_spatial_stage_count"));
        assert!(rendered.contains("deployment_monitoring"));
        assert!(rendered.contains("renderer_export"));
    }

    #[test]
    fn stretch_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_stretch_boundary_text();
        assert!(rendered.contains("stretch_boundary: signal.runtime.stretch-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:stretch-boundary"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::stretch_engine_snapshot and RuntimeSupervisorReport::observation.stretch_engine_snapshot"
        ));
        assert!(rendered.contains(
            "surface: RuntimeClipRenderResult::stretch_engine_snapshot and RuntimeOfflineRenderContractPreview::stretch_engine_snapshot"
        ));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_stretch_boundary_reports_runtime_owned_engine_truth"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-stretch-boundary --format=json"
        ));
    }

    #[test]
    fn stretch_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_stretch_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.stretch-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/046-sample-domain-time-stretch-engine-contract.md\""
        ));
        assert!(rendered.contains("\"acceptance_task\":\"effigy acceptance:stretch-boundary\""));
        assert!(rendered.contains("\"id\":\"runtime-stretch-observation-report\""));
        assert!(rendered.contains("\"id\":\"runtime-stretch-render-preview-snapshot\""));
        assert!(rendered.contains("\"id\":\"shared-host-stretch-report\""));
        assert!(rendered.contains("\"id\":\"runtime-stretch-public-proof\""));
    }

    #[test]
    fn marker_analysis_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_marker_analysis_boundary_text();
        assert!(
            rendered.contains("marker_analysis_boundary: signal.runtime.marker-analysis-boundary")
        );
        assert!(rendered.contains("acceptance_task: effigy acceptance:marker-analysis-boundary"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::marker_analysis_snapshot and RuntimeSupervisorReport::observation.marker_analysis_snapshot"
        ));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_marker_analysis_boundary_reports_runtime_owned_analysis_truth"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-marker-analysis-boundary --format=json"
        ));
    }

    #[test]
    fn marker_analysis_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_marker_analysis_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.marker-analysis-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/047-warp-marker-transient-anchor-and-tempo-assist-analysis-contract.md\""
        ));
        assert!(
            rendered.contains("\"acceptance_task\":\"effigy acceptance:marker-analysis-boundary\"")
        );
        assert!(rendered.contains("\"id\":\"runtime-marker-analysis-observation-report\""));
        assert!(rendered.contains("\"id\":\"shared-host-marker-analysis-report\""));
        assert!(rendered.contains("\"id\":\"runtime-marker-analysis-public-proof\""));
    }

    #[test]
    fn transform_artifact_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_transform_artifact_boundary_text();
        assert!(rendered
            .contains("transform_artifact_boundary: signal.runtime.transform-artifact-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:transform-artifact-boundary"));
        assert!(rendered.contains(
            "contract_path: docs/contracts/064-asset-session-transform-persistence-retention-and-cache-placement-policy-contract.md"
        ));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::transform_artifact_snapshot and RuntimeSupervisorReport::observation.transform_artifact_snapshot"
        ));
        assert!(rendered.contains("transform_persistence"));
        assert!(rendered.contains("persistence_posture"));
        assert!(rendered.contains("retention_outcome"));
        assert!(rendered.contains("cache_placement_outcome"));
        assert!(rendered.contains(
            "surface: RuntimeClipRenderResult::transform_artifact_snapshot and RuntimeOfflineRenderContractPreview::transform_artifact_snapshot"
        ));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_transform_artifact_boundary_reports_runtime_owned_artifact_truth"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-transform-artifact-boundary --format=json"
        ));
    }

    #[test]
    fn transform_artifact_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_transform_artifact_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.transform-artifact-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/064-asset-session-transform-persistence-retention-and-cache-placement-policy-contract.md\""
        ));
        assert!(rendered
            .contains("\"acceptance_task\":\"effigy acceptance:transform-artifact-boundary\""));
        assert!(rendered.contains("\"id\":\"runtime-transform-artifact-observation-report\""));
        assert!(rendered.contains("\"id\":\"runtime-transform-artifact-render-preview-snapshot\""));
        assert!(rendered.contains("\"id\":\"shared-host-transform-artifact-report\""));
        assert!(rendered.contains("\"id\":\"runtime-transform-artifact-public-proof\""));
        assert!(rendered.contains("transform_persistence"));
        assert!(rendered.contains("persistence_posture"));
        assert!(rendered.contains("retention_outcome"));
        assert!(rendered.contains("cache_placement_outcome"));
    }

    #[test]
    fn preview_transform_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_preview_transform_boundary_text();
        assert!(rendered
            .contains("preview_transform_boundary: signal.runtime.preview-transform-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:preview-transform-boundary"));
        assert!(rendered.contains(
            "contract_path: docs/contracts/063-preview-browser-queue-media-audition-and-transform-scheduling-contract.md"
        ));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::preview_transform_snapshot and RuntimeSupervisorReport::observation.preview_transform_snapshot"
        ));
        assert!(rendered.contains(
            "surface: RuntimeClipRenderResult::preview_transform_snapshot and RuntimeOfflineRenderContractPreview::preview_transform_snapshot"
        ));
        assert!(rendered.contains("preview_device_policy"));
        assert!(rendered.contains("preview_workflow"));
        assert!(rendered.contains("queue_posture"));
        assert!(rendered.contains("audition_continuity_outcome"));
        assert!(rendered.contains("transform_scheduling_outcome"));
        assert!(rendered.contains("routing_posture"));
        assert!(rendered.contains("audition_sink_class"));
        assert!(rendered.contains("low_latency_device_policy_outcome"));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_preview_transform_boundary_reports_runtime_owned_preview_truth"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-preview-transform-boundary --format=json"
        ));
    }

    #[test]
    fn preview_transform_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_preview_transform_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.preview-transform-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/063-preview-browser-queue-media-audition-and-transform-scheduling-contract.md\""
        ));
        assert!(rendered
            .contains("\"acceptance_task\":\"effigy acceptance:preview-transform-boundary\""));
        assert!(rendered.contains("\"id\":\"runtime-preview-transform-observation-report\""));
        assert!(rendered.contains("\"id\":\"runtime-preview-transform-render-preview-snapshot\""));
        assert!(rendered.contains("\"id\":\"shared-host-preview-transform-report\""));
        assert!(rendered.contains("\"id\":\"runtime-preview-transform-public-proof\""));
        assert!(rendered.contains("preview_device_policy"));
        assert!(rendered.contains("preview_workflow"));
        assert!(rendered.contains("queue_posture"));
        assert!(rendered.contains("audition_continuity_outcome"));
        assert!(rendered.contains("transform_scheduling_outcome"));
        assert!(rendered.contains("routing_posture"));
        assert!(rendered.contains("audition_sink_class"));
        assert!(rendered.contains("low_latency_device_policy_outcome"));
    }

    #[test]
    fn integrated_acceptance_lane_text_reports_required_and_advisory_policy() {
        let rendered = render_integrated_acceptance_lane_text();
        assert!(rendered
            .contains("integrated_acceptance_lane: signal.runtime.integrated-acceptance-lane"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:integrated-acceptance-lane"));
        assert!(rendered.contains("- effigy acceptance:interruption-boundary"));
        assert!(rendered.contains("- effigy acceptance:analysis-metadata-boundary"));
        assert!(rendered.contains("- effigy acceptance:recording-continuity"));
        assert!(rendered.contains("- effigy acceptance:vst3-boundary"));
        assert!(rendered.contains("title: Adapter And Portability Breadth"));
        assert!(rendered.contains("id: cross-family-export-proof"));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-integrated-acceptance-lane --format=json"
        ));
    }

    #[test]
    fn integrated_acceptance_lane_json_reports_required_and_advisory_policy() {
        let rendered = render_integrated_acceptance_lane_json();
        assert!(rendered.contains("\"lane\":\"signal.runtime.integrated-acceptance-lane\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/030-fault-injection-harness-and-multi-backend-acceptance-contract.md\""
        ));
        assert!(rendered
            .contains("\"acceptance_task\":\"effigy acceptance:integrated-acceptance-lane\""));
        assert!(rendered.contains("\"required_task_count\":11"));
        assert!(rendered.contains("\"advisory_task_count\":6"));
        assert!(rendered.contains("\"id\":\"recovery-and-fault-attribution\""));
        assert!(rendered.contains("\"id\":\"adapter-and-portability-breadth\""));
        assert!(rendered.contains("\"id\":\"cross-family-export-proof\""));
        assert!(rendered.contains(
            "\"command\":\"cargo test -p signal-supervisor-tools export_json_carries_cross_family_integrated_acceptance_evidence\""
        ));
        assert!(rendered.contains("\"command\":\"effigy acceptance:integrated-acceptance-lane\""));
    }

    #[test]
    fn g07_acceptance_lane_text_reports_required_and_advisory_policy() {
        let rendered = render_g07_acceptance_lane_text();
        assert!(
            rendered.contains("g07_acceptance_lane: signal.runtime.g07-integrated-acceptance-lane")
        );
        assert!(
            rendered.contains("acceptance_task: effigy acceptance:g07-integrated-acceptance-lane")
        );
        assert!(rendered.contains("- effigy acceptance:multichannel-boundary"));
        assert!(rendered.contains("- effigy acceptance:preview-transform-boundary"));
        assert!(rendered.contains("- effigy acceptance:complex-io-boundary"));
        assert!(rendered.contains("- effigy acceptance:lv2-boundary"));
        assert!(rendered.contains("title: Linux Plugin And Backend Continuity"));
        assert!(rendered.contains("id: cross-family-export-proof"));
        assert!(rendered.contains("id: required-lane-task"));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-g07-acceptance-lane --format=json"
        ));
    }

    #[test]
    fn g07_acceptance_lane_json_reports_required_and_advisory_policy() {
        let rendered = render_g07_acceptance_lane_json();
        assert!(rendered.contains("\"lane\":\"signal.runtime.g07-integrated-acceptance-lane\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/050-multichannel-linux-time-stretch-and-control-surface-acceptance-contract.md\""
        ));
        assert!(rendered
            .contains("\"acceptance_task\":\"effigy acceptance:g07-integrated-acceptance-lane\""));
        assert!(rendered.contains("\"required_task_count\":15"));
        assert!(rendered.contains("\"advisory_task_count\":2"));
        assert!(rendered.contains("\"id\":\"routing-and-multichannel-coherence\""));
        assert!(rendered.contains("\"id\":\"linux-plugin-and-backend-continuity\""));
        assert!(rendered.contains("\"id\":\"external-control-and-advanced-hardware\""));
        assert!(rendered.contains("\"id\":\"stretch-analysis-artifact-and-preview\""));
        assert!(rendered.contains("\"id\":\"cross-family-export-proof\""));
        assert!(rendered.contains(
            "\"command\":\"cargo test -p signal-supervisor-tools export_json_carries_cross_family_g07_acceptance_evidence\""
        ));
        assert!(
            rendered.contains("\"command\":\"effigy acceptance:g07-integrated-acceptance-lane\"")
        );
    }

    #[test]
    fn device_workflow_acceptance_lane_text_reports_required_and_deferred_policy() {
        let rendered = render_device_workflow_acceptance_lane_text();
        assert!(rendered.contains(
            "device_workflow_acceptance_lane: signal.runtime.device-workflow-acceptance-lane"
        ));
        assert!(
            rendered.contains("acceptance_task: effigy acceptance:device-workflow-acceptance-lane")
        );
        assert!(rendered.contains(
            "contract_path: docs/contracts/066-cross-backend-device-protocol-and-live-workflow-acceptance-contract.md"
        ));
        assert!(rendered.contains("- effigy acceptance:external-midi-boundary"));
        assert!(rendered.contains("- effigy acceptance:advanced-hardware-boundary"));
        assert!(rendered.contains("title: Live Endpoint Ownership And Protocol Continuity"));
        assert!(rendered.contains("title: Control-Surface And Advanced Hardware Workflow"));
        assert!(rendered.contains("title: Cross-Backend Host-Edge Coherence"));
        assert!(rendered.contains("id: cross-family-export-proof"));
        assert!(rendered.contains("id: required-lane-task"));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-device-workflow-acceptance-lane --format=json"
        ));
    }

    #[test]
    fn device_workflow_acceptance_lane_json_reports_required_and_deferred_policy() {
        let rendered = render_device_workflow_acceptance_lane_json();
        assert!(rendered.contains("\"lane\":\"signal.runtime.device-workflow-acceptance-lane\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/066-cross-backend-device-protocol-and-live-workflow-acceptance-contract.md\""
        ));
        assert!(rendered
            .contains("\"acceptance_task\":\"effigy acceptance:device-workflow-acceptance-lane\""));
        assert!(rendered.contains("\"required_task_count\":4"));
        assert!(rendered.contains("\"advisory_task_count\":0"));
        assert!(rendered.contains("\"id\":\"live-endpoint-ownership-and-protocol-continuity\""));
        assert!(rendered.contains("\"id\":\"control-surface-and-advanced-hardware-workflow\""));
        assert!(rendered.contains("\"id\":\"cross-backend-host-edge-coherence\""));
        assert!(rendered.contains("\"id\":\"cross-family-export-proof\""));
        assert!(rendered.contains(
            "\"command\":\"cargo test -p signal-supervisor-tools export_json_carries_cross_family_device_workflow_acceptance_evidence\""
        ));
        assert!(rendered.contains("\"id\":\"required-lane-task\""));
        assert!(
            rendered.contains("\"command\":\"effigy acceptance:device-workflow-acceptance-lane\"")
        );
    }

    #[test]
    fn linux_live_acceptance_lane_text_reports_required_and_deferred_policy() {
        let rendered = render_linux_live_acceptance_lane_text();
        assert!(rendered
            .contains("linux_live_acceptance_lane: signal.runtime.linux-live-acceptance-lane"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:linux-live-acceptance-lane"));
        assert!(rendered.contains(
            "contract_path: docs/contracts/067-live-linux-backend-acceptance-and-failure-injection-contract.md"
        ));
        assert!(rendered.contains("- effigy acceptance:linux-live-ownership-boundary"));
        assert!(rendered.contains("- effigy acceptance:jack-coordination-boundary"));
        assert!(rendered.contains("- effigy acceptance:pipewire-alsa-parity-boundary"));
        assert!(rendered.contains("- effigy acceptance:linux-backend-clock-topology-boundary"));
        assert!(rendered.contains("title: Live Ownership And Guarded Continuity"));
        assert!(rendered.contains("title: Backend-Native Coordination And Parity"));
        assert!(rendered.contains("title: Cross-Backend Host-Edge Coherence"));
        assert!(rendered.contains("id: cross-family-export-proof"));
        assert!(rendered.contains(
            "cargo test -p signal-supervisor-tools export_json_carries_cross_family_linux_live_acceptance_evidence"
        ));
        assert!(rendered.contains("id: required-lane-task"));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-linux-live-acceptance-lane --format=json"
        ));
    }

    #[test]
    fn linux_live_acceptance_lane_json_reports_required_and_deferred_policy() {
        let rendered = render_linux_live_acceptance_lane_json();
        assert!(rendered.contains("\"lane\":\"signal.runtime.linux-live-acceptance-lane\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/067-live-linux-backend-acceptance-and-failure-injection-contract.md\""
        ));
        assert!(rendered
            .contains("\"acceptance_task\":\"effigy acceptance:linux-live-acceptance-lane\""));
        assert!(rendered.contains("\"required_task_count\":4"));
        assert!(rendered.contains("\"advisory_task_count\":0"));
        assert!(rendered.contains("\"id\":\"live-ownership-and-guarded-continuity\""));
        assert!(rendered.contains("\"id\":\"backend-native-coordination-and-parity\""));
        assert!(rendered.contains("\"id\":\"cross-backend-host-edge-coherence\""));
        assert!(rendered.contains("\"id\":\"cross-family-export-proof\""));
        assert!(rendered.contains(
            "\"command\":\"cargo test -p signal-supervisor-tools export_json_carries_cross_family_linux_live_acceptance_evidence\""
        ));
        assert!(rendered.contains("\"id\":\"required-lane-task\""));
        assert!(rendered.contains("\"command\":\"effigy acceptance:linux-live-acceptance-lane\""));
    }

    #[test]
    fn immersive_acceptance_lane_text_reports_required_and_deferred_policy() {
        let rendered = render_immersive_acceptance_lane_text();
        assert!(rendered
            .contains("immersive_acceptance_lane: signal.runtime.immersive-acceptance-lane"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:immersive-acceptance-lane"));
        assert!(rendered.contains(
            "contract_path: docs/contracts/068-immersive-render-and-monitoring-acceptance-contract.md"
        ));
        assert!(rendered.contains("- effigy acceptance:spatial-boundary"));
        assert!(rendered.contains("title: Room-Policy And Render Continuity"));
        assert!(rendered.contains("title: Deployment Fold-Down And Monitoring Coherence"));
        assert!(rendered.contains("title: Cross-Surface Immersive Coherence"));
        assert!(rendered.contains("id: cross-family-export-proof"));
        assert!(rendered.contains(
            "cargo test -p signal-supervisor-tools export_json_carries_cross_family_immersive_acceptance_evidence"
        ));
        assert!(rendered.contains("id: required-lane-task"));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-immersive-acceptance-lane --format=json"
        ));
    }

    #[test]
    fn immersive_acceptance_lane_json_reports_required_and_deferred_policy() {
        let rendered = render_immersive_acceptance_lane_json();
        assert!(rendered.contains("\"lane\":\"signal.runtime.immersive-acceptance-lane\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/068-immersive-render-and-monitoring-acceptance-contract.md\""
        ));
        assert!(rendered
            .contains("\"acceptance_task\":\"effigy acceptance:immersive-acceptance-lane\""));
        assert!(rendered.contains("\"required_task_count\":1"));
        assert!(rendered.contains("\"advisory_task_count\":0"));
        assert!(rendered.contains("\"id\":\"room-policy-and-render-continuity\""));
        assert!(rendered.contains("\"id\":\"deployment-fold-down-and-monitoring-coherence\""));
        assert!(rendered.contains("\"id\":\"cross-surface-immersive-coherence\""));
        assert!(rendered.contains("\"id\":\"cross-family-export-proof\""));
        assert!(rendered.contains(
            "\"command\":\"cargo test -p signal-supervisor-tools export_json_carries_cross_family_immersive_acceptance_evidence\""
        ));
        assert!(rendered.contains("\"id\":\"required-lane-task\""));
        assert!(rendered.contains("\"command\":\"effigy acceptance:immersive-acceptance-lane\""));
    }

    #[test]
    fn control_preview_workflow_acceptance_lane_text_reports_required_and_deferred_policy() {
        let rendered = render_control_preview_workflow_acceptance_lane_text();
        assert!(rendered.contains(
            "control_preview_workflow_acceptance_lane: signal.runtime.control-preview-workflow-acceptance-lane"
        ));
        assert!(rendered.contains(
            "acceptance_task: effigy acceptance:control-preview-workflow-acceptance-lane"
        ));
        assert!(rendered.contains(
            "contract_path: docs/contracts/069-control-surface-and-preview-workflow-acceptance-contract.md"
        ));
        assert!(rendered.contains("- effigy acceptance:advanced-hardware-boundary"));
        assert!(rendered.contains("- effigy acceptance:preview-transform-boundary"));
        assert!(rendered.contains("title: Control-Surface Workflow Coherence"));
        assert!(rendered.contains("title: Preview Workflow Coherence"));
        assert!(rendered.contains("title: Cross-Surface Workflow Coherence"));
        assert!(rendered.contains("id: cross-family-export-proof"));
        assert!(rendered.contains(
            "cargo test -p signal-supervisor-tools export_json_carries_cross_family_control_preview_workflow_acceptance_evidence"
        ));
        assert!(rendered.contains("id: lane-descriptor-proof"));
        assert!(rendered.contains("id: required-lane-task"));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-control-preview-workflow-acceptance-lane --format=json"
        ));
    }

    #[test]
    fn control_preview_workflow_acceptance_lane_json_reports_required_and_deferred_policy() {
        let rendered = render_control_preview_workflow_acceptance_lane_json();
        assert!(rendered
            .contains("\"lane\":\"signal.runtime.control-preview-workflow-acceptance-lane\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/069-control-surface-and-preview-workflow-acceptance-contract.md\""
        ));
        assert!(rendered.contains(
            "\"acceptance_task\":\"effigy acceptance:control-preview-workflow-acceptance-lane\""
        ));
        assert!(rendered.contains("\"required_task_count\":2"));
        assert!(rendered.contains("\"advisory_task_count\":0"));
        assert!(rendered.contains("\"id\":\"control-surface-workflow-coherence\""));
        assert!(rendered.contains("\"id\":\"preview-workflow-coherence\""));
        assert!(rendered.contains("\"id\":\"cross-surface-workflow-coherence\""));
        assert!(rendered.contains("\"id\":\"cross-family-export-proof\""));
        assert!(rendered.contains(
            "\"command\":\"cargo test -p signal-supervisor-tools export_json_carries_cross_family_control_preview_workflow_acceptance_evidence\""
        ));
        assert!(rendered.contains("\"id\":\"lane-descriptor-proof\""));
        assert!(rendered.contains(
            "\"command\":\"cargo test -p signal-supervisor-tools control_preview_workflow_acceptance_lane_json_reports_required_and_deferred_policy\""
        ));
        assert!(rendered.contains("\"id\":\"required-lane-task\""));
        assert!(rendered.contains(
            "\"command\":\"effigy acceptance:control-preview-workflow-acceptance-lane\""
        ));
    }

    #[test]
    fn integrated_live_workflow_acceptance_lane_text_reports_required_and_deferred_policy() {
        let rendered = render_integrated_live_workflow_acceptance_lane_text();
        assert!(rendered.contains(
            "integrated_live_workflow_acceptance_lane: signal.runtime.integrated-live-ownership-and-workflow-acceptance-lane"
        ));
        assert!(rendered.contains(
            "acceptance_task: effigy acceptance:integrated-live-ownership-and-workflow-acceptance-lane"
        ));
        assert!(rendered.contains(
            "contract_path: docs/contracts/070-integrated-live-ownership-and-workflow-acceptance-contract.md"
        ));
        assert!(rendered.contains("- effigy acceptance:linux-live-acceptance-lane"));
        assert!(rendered.contains("- effigy acceptance:device-workflow-acceptance-lane"));
        assert!(rendered.contains("- effigy acceptance:immersive-acceptance-lane"));
        assert!(rendered.contains("- effigy acceptance:control-preview-workflow-acceptance-lane"));
        assert!(rendered.contains("title: Linux Live And Device Workflow Continuity"));
        assert!(rendered.contains("title: Immersive And Preview Workflow Continuity"));
        assert!(rendered.contains("title: Cross-Surface Integrated Coherence"));
        assert!(rendered.contains("title: Shared Grouped Integrated Acceptance Export"));
        assert!(rendered.contains("id: cross-family-export-proof"));
        assert!(rendered.contains(
            "cargo test -p signal-supervisor-tools export_json_carries_cross_family_integrated_live_workflow_acceptance_evidence"
        ));
        assert!(rendered.contains("id: lane-descriptor-proof"));
        assert!(rendered.contains("id: required-lane-task"));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-integrated-live-workflow-acceptance-lane --format=json"
        ));
    }

    #[test]
    fn integrated_live_workflow_acceptance_lane_json_reports_required_and_deferred_policy() {
        let rendered = render_integrated_live_workflow_acceptance_lane_json();
        assert!(rendered.contains(
            "\"lane\":\"signal.runtime.integrated-live-ownership-and-workflow-acceptance-lane\""
        ));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/070-integrated-live-ownership-and-workflow-acceptance-contract.md\""
        ));
        assert!(rendered.contains(
            "\"acceptance_task\":\"effigy acceptance:integrated-live-ownership-and-workflow-acceptance-lane\""
        ));
        assert!(rendered.contains("\"required_task_count\":4"));
        assert!(rendered.contains("\"advisory_task_count\":0"));
        assert!(rendered.contains("\"id\":\"linux-live-and-device-workflow-continuity\""));
        assert!(rendered.contains("\"id\":\"immersive-and-preview-workflow-continuity\""));
        assert!(rendered.contains("\"id\":\"cross-surface-integrated-coherence\""));
        assert!(rendered.contains("\"id\":\"shared-grouped-integrated-acceptance-export\""));
        assert!(rendered.contains("\"id\":\"cross-family-export-proof\""));
        assert!(rendered.contains(
            "\"command\":\"cargo test -p signal-supervisor-tools export_json_carries_cross_family_integrated_live_workflow_acceptance_evidence\""
        ));
        assert!(rendered.contains("\"id\":\"lane-descriptor-proof\""));
        assert!(rendered.contains(
            "\"command\":\"cargo test -p signal-supervisor-tools integrated_live_workflow_acceptance_lane_json_reports_required_and_deferred_policy\""
        ));
        assert!(rendered.contains("\"id\":\"required-lane-task\""));
        assert!(rendered.contains(
            "\"command\":\"effigy acceptance:integrated-live-ownership-and-workflow-acceptance-lane\""
        ));
    }

    #[test]
    fn export_json_carries_cross_family_device_workflow_acceptance_evidence() {
        let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        let recorder = RuntimeEventRecorder::default();
        let report = RuntimeSupervisorReport::capture(&runtime, &recorder);
        let observation = report
            .observation
            .clone()
            .with_host_external_io(&sample_g07_acceptance_host_io())
            .with_external_midi_snapshot(sample_g07_external_midi_snapshot());
        let report = RuntimeSupervisorReport {
            observation,
            ..report
        };

        let export = render_supervisor_export_json(
            HostProfile::Local,
            Scenario::Mixed,
            "{}".into(),
            &report.profiling_receipt(),
            &report.soak_receipt(),
            &report,
        );

        assert!(export.contains("\"external_midi_snapshot\":{"));
        assert!(export.contains("\"live_ownership\":{"));
        assert!(export.contains("\"backend_parity\":\""));
        assert!(export.contains("\"attach_continuity\":\""));
        assert!(export.contains("\"supports_widened_expression\":true"));
        assert!(export.contains("\"control_surface_snapshot\":{"));
        assert!(export.contains("\"graph_state\":\"Guarded\""));
        assert!(export.contains("\"advanced_hardware_snapshot\":{"));
    }

    #[test]
    fn export_json_carries_cross_family_linux_live_acceptance_evidence() {
        let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        let recorder = RuntimeEventRecorder::default();
        let report = RuntimeSupervisorReport::capture(&runtime, &recorder);
        let observation = report
            .observation
            .clone()
            .with_host_external_io(&sample_g07_acceptance_host_io())
            .with_external_midi_snapshot(sample_control_preview_workflow_external_midi_snapshot());
        let report = RuntimeSupervisorReport {
            observation,
            ..report
        };

        let export = render_supervisor_export_json(
            HostProfile::Local,
            Scenario::Mixed,
            "{}".into(),
            &report.profiling_receipt(),
            &report.soak_receipt(),
            &report,
        );

        assert!(export.contains("\"linux_backend_session_snapshot\":{"));
        assert!(export.contains("\"jack_coordination_snapshot\":{"));
        assert!(export.contains("\"pipewire_alsa_parity_snapshot\":{"));
        assert!(export.contains("\"transport_posture\":\"Unavailable\""));
        assert!(export.contains("\"session_role\":\"Unavailable\""));
        assert!(export.contains("\"clock_domain\":\"SameClock\""));
        assert!(export.contains("\"linux_clocking_parity\":\"Portable\""));
    }

    #[test]
    fn export_json_carries_cross_family_immersive_acceptance_evidence() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 128));
        runtime
            .handshake(HandshakeRequest {
                client_version: "immersive-acceptance-export".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(96_000),
            })
            .expect("immersive acceptance export handshake should succeed");
        runtime
            .configure(RuntimeConfigRequest::new(48_000, 128))
            .expect("immersive acceptance export configure should succeed");
        runtime
            .apply_graph_projection(signal_runtime::GraphProjection {
                graph_id: "graph:supervisor:immersive-acceptance".into(),
                node_count: 2,
                nodes: vec![
                    signal_runtime::GraphNodeProjection {
                        node_id: "spatial-stereo".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 12,
                        stages: vec![GraphStageSpec::StereoBalance { balance: -0.2 }],
                    },
                    signal_runtime::GraphNodeProjection {
                        node_id: "spatial-surround".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 20,
                        stages: vec![GraphStageSpec::StereoBalance { balance: 0.35 }],
                    },
                ],
            })
            .expect("immersive acceptance graph should apply");
        runtime
            .apply_graph_contract_projection(signal_runtime::GraphContractProjection {
                graph_id: "graph:supervisor:immersive-acceptance".into(),
                contract_count: 2,
                nodes: vec![
                    signal_runtime::GraphNodeContractProjection {
                        node_id: "spatial-stereo".into(),
                        buffer_contract: signal_runtime::GraphNodeBufferContractProjection {
                            input: signal_runtime::GraphNodeBusEndpointProjection {
                                bus_id: "main:in".into(),
                                channels: ChannelLayout::Stereo,
                            },
                            output: signal_runtime::GraphNodeBusEndpointProjection {
                                bus_id: "bus:spatial:stereo".into(),
                                channels: ChannelLayout::Stereo,
                            },
                            ..signal_runtime::GraphNodeBufferContractProjection::default()
                        },
                        topology: signal_runtime::GraphNodeTopologyProjection {
                            role: Some(GraphNodeTopologyRole::TrackLane),
                            track_lane_id: Some("track:stereo".into()),
                            bus_group_id: Some("bus:spatial:stereo".into()),
                            console_group_id: None,
                            send_return_id: None,
                        },
                    },
                    signal_runtime::GraphNodeContractProjection {
                        node_id: "spatial-surround".into(),
                        buffer_contract: signal_runtime::GraphNodeBufferContractProjection {
                            input: signal_runtime::GraphNodeBusEndpointProjection {
                                bus_id: "main:surround-in".into(),
                                channels: ChannelLayout::Count(ChannelCount(6)),
                            },
                            output: signal_runtime::GraphNodeBusEndpointProjection {
                                bus_id: "bus:spatial:surround".into(),
                                channels: ChannelLayout::Count(ChannelCount(6)),
                            },
                            ..signal_runtime::GraphNodeBufferContractProjection::default()
                        },
                        topology: signal_runtime::GraphNodeTopologyProjection {
                            role: Some(GraphNodeTopologyRole::TrackLane),
                            track_lane_id: Some("track:surround".into()),
                            bus_group_id: Some("bus:spatial:surround".into()),
                            console_group_id: None,
                            send_return_id: None,
                        },
                    },
                ],
            })
            .expect("immersive acceptance graph contract should apply");
        runtime
            .apply_plugin_backed_node_bindings(signal_runtime::PluginBackedNodeBindingProjection {
                graph_id: "graph:supervisor:immersive-acceptance".into(),
                bindings: vec![
                    signal_runtime::PluginBackedNodeBinding {
                        node_id: "spatial-stereo".into(),
                        sandbox_id: "sandbox:spatial-stereo".into(),
                    },
                    signal_runtime::PluginBackedNodeBinding {
                        node_id: "spatial-surround".into(),
                        sandbox_id: "sandbox:spatial-surround".into(),
                    },
                ],
            })
            .expect("immersive acceptance bindings should apply");
        runtime.record_plugin_sandbox_lifecycle(
            "sandbox:spatial-stereo",
            PluginSandboxLifecycleStage::InstancePrepared,
            Some(1),
        );
        runtime.record_plugin_sandbox_lifecycle(
            "sandbox:spatial-surround",
            PluginSandboxLifecycleStage::InstancePrepared,
            Some(1),
        );

        let recorder = RuntimeEventRecorder::default();
        let report = RuntimeSupervisorReport::capture(&runtime, &recorder);

        let export = render_supervisor_export_json(
            HostProfile::Local,
            Scenario::Mixed,
            "{}".into(),
            &report.profiling_receipt(),
            &report.soak_receipt(),
            &report,
        );

        assert!(export.contains("\"execution_topology_summary\":{"));
        assert!(export.contains("\"plugin_chain_snapshot\":{"));
        assert!(export.contains("\"immersive_spatial_node_count\":"));
        assert!(export.contains("\"fallback_monitoring_scene_spatial_node_count\":"));
        assert!(export.contains("\"renderer_capability_spatial_node_count\":"));
        assert!(export.contains("\"immersive_export_spatial_node_count\":"));
        assert!(export.contains("\"immersive_room_policy\":{"));
        assert!(export.contains("\"deployment_monitoring\":{"));
        assert!(export.contains("\"renderer_export\":{"));
    }

    #[test]
    fn export_json_carries_cross_family_control_preview_workflow_acceptance_evidence() {
        let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        let recorder = RuntimeEventRecorder::default();
        let report = RuntimeSupervisorReport::capture(&runtime, &recorder);
        let observation = report
            .observation
            .clone()
            .with_host_external_io(&sample_g07_acceptance_host_io());
        let report = RuntimeSupervisorReport {
            observation,
            ..report
        };

        let export = render_supervisor_export_json(
            HostProfile::Local,
            Scenario::Mixed,
            "{}".into(),
            &report.profiling_receipt(),
            &report.soak_receipt(),
            &report,
        );

        assert!(export.contains("\"control_surface_snapshot\":{"));
        assert!(export.contains("\"advanced_hardware_snapshot\":{"));
        assert!(export.contains("\"preview_transform_snapshot\":{"));
        assert!(export.contains("\"preview_device_policy\":{"));
        assert!(export.contains("\"routing_posture\":\""));
        assert!(export.contains("\"low_latency_device_policy_outcome\":\""));
        assert!(export.contains("\"preview_workflow\":{"));
        assert!(export.contains("\"queue_posture\":\""));
        assert!(export.contains("\"audition_continuity_outcome\":\""));
        assert!(export.contains("\"transform_scheduling_outcome\":\""));
    }

    #[test]
    fn export_json_carries_cross_family_integrated_live_workflow_acceptance_evidence() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 128));
        runtime
            .handshake(HandshakeRequest {
                client_version: "integrated-live-workflow-acceptance-export".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(96_000),
            })
            .expect("integrated live workflow acceptance handshake should succeed");
        runtime
            .configure(RuntimeConfigRequest::new(48_000, 128))
            .expect("integrated live workflow acceptance configure should succeed");
        runtime
            .apply_graph_projection(signal_runtime::GraphProjection {
                graph_id: "graph:supervisor:integrated-live-workflow".into(),
                node_count: 2,
                nodes: vec![
                    signal_runtime::GraphNodeProjection {
                        node_id: "spatial-stereo".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 12,
                        stages: vec![GraphStageSpec::StereoBalance { balance: -0.2 }],
                    },
                    signal_runtime::GraphNodeProjection {
                        node_id: "spatial-surround".into(),
                        execution_class: GraphNodeExecutionClass::PluginBacked,
                        latency_samples: 20,
                        stages: vec![GraphStageSpec::StereoBalance { balance: 0.35 }],
                    },
                ],
            })
            .expect("integrated live workflow graph should apply");
        runtime
            .apply_graph_contract_projection(signal_runtime::GraphContractProjection {
                graph_id: "graph:supervisor:integrated-live-workflow".into(),
                contract_count: 2,
                nodes: vec![
                    signal_runtime::GraphNodeContractProjection {
                        node_id: "spatial-stereo".into(),
                        buffer_contract: signal_runtime::GraphNodeBufferContractProjection {
                            input: signal_runtime::GraphNodeBusEndpointProjection {
                                bus_id: "main:in".into(),
                                channels: ChannelLayout::Stereo,
                            },
                            output: signal_runtime::GraphNodeBusEndpointProjection {
                                bus_id: "bus:spatial:stereo".into(),
                                channels: ChannelLayout::Stereo,
                            },
                            ..signal_runtime::GraphNodeBufferContractProjection::default()
                        },
                        topology: signal_runtime::GraphNodeTopologyProjection {
                            role: Some(GraphNodeTopologyRole::TrackLane),
                            track_lane_id: Some("track:stereo".into()),
                            bus_group_id: Some("bus:spatial:stereo".into()),
                            console_group_id: None,
                            send_return_id: None,
                        },
                    },
                    signal_runtime::GraphNodeContractProjection {
                        node_id: "spatial-surround".into(),
                        buffer_contract: signal_runtime::GraphNodeBufferContractProjection {
                            input: signal_runtime::GraphNodeBusEndpointProjection {
                                bus_id: "main:surround-in".into(),
                                channels: ChannelLayout::Count(ChannelCount(6)),
                            },
                            output: signal_runtime::GraphNodeBusEndpointProjection {
                                bus_id: "bus:spatial:surround".into(),
                                channels: ChannelLayout::Count(ChannelCount(6)),
                            },
                            ..signal_runtime::GraphNodeBufferContractProjection::default()
                        },
                        topology: signal_runtime::GraphNodeTopologyProjection {
                            role: Some(GraphNodeTopologyRole::TrackLane),
                            track_lane_id: Some("track:surround".into()),
                            bus_group_id: Some("bus:spatial:surround".into()),
                            console_group_id: None,
                            send_return_id: None,
                        },
                    },
                ],
            })
            .expect("integrated live workflow graph contract should apply");
        runtime
            .apply_plugin_backed_node_bindings(signal_runtime::PluginBackedNodeBindingProjection {
                graph_id: "graph:supervisor:integrated-live-workflow".into(),
                bindings: vec![
                    signal_runtime::PluginBackedNodeBinding {
                        node_id: "spatial-stereo".into(),
                        sandbox_id: "sandbox:integrated-live-stereo".into(),
                    },
                    signal_runtime::PluginBackedNodeBinding {
                        node_id: "spatial-surround".into(),
                        sandbox_id: "sandbox:integrated-live-surround".into(),
                    },
                ],
            })
            .expect("integrated live workflow bindings should apply");
        runtime.record_plugin_sandbox_lifecycle(
            "sandbox:integrated-live-stereo",
            PluginSandboxLifecycleStage::InstancePrepared,
            Some(1),
        );
        runtime.record_plugin_sandbox_lifecycle(
            "sandbox:integrated-live-surround",
            PluginSandboxLifecycleStage::InstancePrepared,
            Some(1),
        );

        let recorder = RuntimeEventRecorder::default();
        let report = RuntimeSupervisorReport::capture(&runtime, &recorder);
        let observation = report
            .observation
            .clone()
            .with_host_external_io(&sample_g07_acceptance_host_io())
            .with_external_midi_snapshot(sample_control_preview_workflow_external_midi_snapshot());
        let report = RuntimeSupervisorReport {
            observation,
            ..report
        };

        let export = render_supervisor_export_json(
            HostProfile::Local,
            Scenario::Mixed,
            "{}".into(),
            &report.profiling_receipt(),
            &report.soak_receipt(),
            &report,
        );

        assert!(export.contains("\"linux_backend_session_snapshot\":{"));
        assert!(export.contains("\"jack_coordination_snapshot\":{"));
        assert!(export.contains("\"pipewire_alsa_parity_snapshot\":{"));
        assert!(export.contains("\"external_midi_snapshot\":{"));
        assert!(export.contains("\"live_ownership\":{"));
        assert!(export.contains("\"control_surface_snapshot\":{"));
        assert!(export.contains("\"advanced_hardware_snapshot\":{"));
        assert!(export.contains("\"preview_transform_snapshot\":{"));
        assert!(export.contains("\"preview_workflow\":{"));
        assert!(export.contains("\"execution_topology_summary\":{"));
        assert!(export.contains("\"plugin_chain_snapshot\":{"));
        assert!(export.contains("\"immersive_room_policy\":{"));
        assert!(export.contains("\"deployment_monitoring\":{"));
        assert!(export.contains("\"renderer_export\":{"));
    }

    #[test]
    fn export_json_carries_cross_family_g07_acceptance_evidence() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
        runtime
            .handshake(HandshakeRequest {
                client_version: "g07-integrated-acceptance-export".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(96_000),
            })
            .expect("g07 integrated acceptance export handshake should succeed");
        runtime
            .configure(RuntimeConfigRequest::new(48_000, 256))
            .expect("g07 integrated acceptance export configure should succeed");
        runtime
            .start()
            .expect("g07 integrated acceptance export start should succeed");

        runtime.record_plugin_format_platform_coverage(vec![
            RuntimePluginFormatPlatformCoverageRecord {
                format: PluginFormat::Clap,
                supported_platforms: vec![
                    RuntimePluginHostPlatform::MacOs,
                    RuntimePluginHostPlatform::Linux,
                    RuntimePluginHostPlatform::Windows,
                ],
                unsupported_platforms: Vec::new(),
                linux_parity_band: RuntimePluginParityBand::Portable,
                linux_preferred_sandbox_outcome: Some(
                    RuntimePluginIsolationOutcome::IsolatedSandbox,
                ),
                linux_strict_sandbox_default: true,
                summary:
                    "platforms=MacOs/Linux/Windows linux=Portable linux_policy=IsolatedSandbox unsupported=none"
                        .into(),
            },
            RuntimePluginFormatPlatformCoverageRecord {
                format: PluginFormat::Vst3,
                supported_platforms: vec![
                    RuntimePluginHostPlatform::MacOs,
                    RuntimePluginHostPlatform::Linux,
                    RuntimePluginHostPlatform::Windows,
                ],
                unsupported_platforms: Vec::new(),
                linux_parity_band: RuntimePluginParityBand::Portable,
                linux_preferred_sandbox_outcome: Some(
                    RuntimePluginIsolationOutcome::IsolatedSandbox,
                ),
                linux_strict_sandbox_default: true,
                summary:
                    "platforms=MacOs/Linux/Windows linux=Portable linux_policy=IsolatedSandbox unsupported=none"
                        .into(),
            },
        ]);
        let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
            roots: vec!["~/.clap".into(), "~/.vst3".into()],
            formats: vec![PluginFormat::Clap, PluginFormat::Vst3],
        });
        runtime.record_plugin_scan_results(
            scan_handle,
            vec![
                sample_discovered_type_record(),
                sample_backend_breadth_record(),
            ],
        );

        let preview_path = integrated_acceptance_media_fixture_path("g07-preview-ready");
        write_g07_acceptance_transient_wav(&preview_path);
        runtime
            .reconcile_media_assets(vec![RuntimeMediaAssetRegistration {
                asset_id: "asset:sha256:g07-preview-ready".into(),
                content_hash: "g07-preview-ready".into(),
                source_path: preview_path.display().to_string(),
                file_name: "g07-preview-ready.wav".into(),
                byte_size: fs::metadata(&preview_path)
                    .expect("g07 acceptance media fixture should exist")
                    .len(),
                sample_rate_hz: 48_000,
                channel_count: 1,
                duration_samples: 48_000,
                waveform_bin_count: 32,
            }])
            .expect("g07 integrated acceptance media asset should reconcile");
        runtime
            .reconcile_warp_clips(vec![signal_runtime::RuntimeWarpClipRegistration {
                clip_id: "clip:g07-preview-ready".into(),
                media_asset_id: Some("asset:sha256:g07-preview-ready".into()),
                mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
                source_tempo_bpm: Some(120.0),
                anchor_timeline_samples: 0,
                start_samples: 0,
                duration_samples: 48_000,
            }])
            .expect("g07 integrated acceptance warp clip should reconcile");
        runtime
            .reconcile_clip_processing_clips(vec![
                signal_runtime::RuntimeClipProcessingRegistration {
                    clip_id: "clip:g07-preview-ready".into(),
                    media_asset_id: Some("asset:sha256:g07-preview-ready".into()),
                    warp_mode: signal_runtime::RuntimeWarpMode::ElastiqueDraft,
                    start_samples: 0,
                    duration_samples: 48_000,
                    fade_in: signal_runtime::RuntimeClipFadeEnvelope::default(),
                    fade_out: signal_runtime::RuntimeClipFadeEnvelope::default(),
                    clip_gain: signal_runtime::RuntimeClipGainEnvelope::default(),
                },
            ])
            .expect("g07 integrated acceptance clip processing clip should reconcile");
        runtime
            .apply_transport_projection(signal_runtime::TransportProjection {
                playing: false,
                timeline_position_samples: 0,
                tempo_bpm: 180.0,
                loop_state: None,
            })
            .expect("g07 integrated acceptance transport projection should apply");
        runtime
            .start_media_preview("asset:sha256:g07-preview-ready")
            .expect("g07 integrated acceptance media preview should start");

        let recorder = RuntimeEventRecorder::default();
        let mut report = RuntimeSupervisorReport::capture(&runtime, &recorder);
        report
            .observation
            .execution_topology_summary
            .secondary_input_count = 1;
        report
            .observation
            .execution_topology_summary
            .required_secondary_input_count = 1;
        report
            .observation
            .execution_topology_summary
            .bus_connection_count = 1;
        report
            .observation
            .execution_topology_summary
            .auxiliary_path_count = 1;
        report
            .observation
            .execution_topology_summary
            .spatial_node_count = 1;
        report
            .observation
            .execution_topology_summary
            .active_spatial_node_count = 1;
        report
            .observation
            .execution_topology_summary
            .surround_bed_spatial_node_count = 1;
        report
            .observation
            .execution_topology_summary
            .expanded_fallback_spatial_node_count = 1;
        report.observation = report
            .observation
            .clone()
            .with_host_external_io(&sample_g07_acceptance_host_io())
            .with_external_midi_snapshot(sample_g07_external_midi_snapshot());

        let export = render_supervisor_export_json(
            HostProfile::Local,
            Scenario::Mixed,
            "{}".into(),
            &report.profiling_receipt(),
            &report.soak_receipt(),
            &report,
        );

        assert!(export.contains("\"plugin_discovery_snapshot\":{"));
        assert!(export.contains("\"plugin_type_id\":\"plugin:clap:export-consumer\""));
        assert!(export.contains("\"plugin_type_id\":\"plugin:vst3:export-instrument\""));
        assert!(export.contains("\"default_multichannel_io\":{"));
        assert!(export.contains("\"execution_topology_summary\":{"));
        assert!(export.contains("\"secondary_input_count\":1"));
        assert!(export.contains("\"bus_connection_count\":1"));
        assert!(export.contains("\"spatial_node_count\":1"));
        assert!(export.contains("\"surround_bed_spatial_node_count\":1"));
        assert!(export.contains("\"external_io_snapshot\":{"));
        assert!(export.contains("\"linux_backend_identity\":\"Alsa\""));
        assert!(export.contains("\"linux_backend_portability\":\"Portable\""));
        assert!(export.contains("\"linux_clocking_parity\":\"Portable\""));
        assert!(export.contains("\"linux_duplex_parity\":\"Aligned\""));
        assert!(export.contains("\"linux_endpoint_topology_parity\":\"Portable\""));
        assert!(export.contains("\"external_midi_snapshot\":{"));
        assert!(export.contains("\"provider_name\":\"signal-host-local\""));
        assert!(export.contains("\"control_surface_snapshot\":{"));
        assert!(export.contains("\"graph_state\":\"Guarded\""));
        assert!(export.contains("\"supports_widened_expression\":true"));
        assert!(export.contains("\"advanced_hardware_snapshot\":{"));
        assert!(export.contains("\"scripting_safe_posture\":\"Guarded\""));
        assert!(export.contains("\"feedback_channel_posture\":\"Guarded\""));
        assert!(export.contains("\"stretch_engine_snapshot\":{"));
        assert!(export.contains("\"marker_analysis_snapshot\":{"));
        assert!(export.contains("\"transform_artifact_snapshot\":{"));
        assert!(export.contains("\"preview_transform_snapshot\":{"));
        assert!(export.contains("\"tempo_assist_ready_clip_count\":1"));
        assert!(export.contains("\"reusable_clip_count\":1"));
        assert!(export.contains("\"active_audition_clip_count\":1"));

        let _ = fs::remove_file(&preview_path);
        if let Some(path) = runtime
            .get_media_pipeline_snapshot()
            .assets
            .iter()
            .find(|asset| asset.asset_id == "asset:sha256:g07-preview-ready")
            .and_then(|asset| asset.cache_path.as_deref())
        {
            let _ = fs::remove_file(path);
        }
    }

    #[test]
    fn g06_soak_lane_text_reports_required_and_deferred_policy() {
        let rendered = render_g06_soak_lane_text();
        assert!(rendered.contains("g06_soak_lane: signal.g06.long-session-soak-lane"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:g06-soak-lane"));
        assert!(rendered.contains("id: required-local-soak-export"));
        assert!(rendered.contains("status: required"));
        assert!(rendered.contains("id: deferred-server-soak-export"));
        assert!(rendered.contains("status: deferred"));
        assert!(rendered.contains("id: g06-soak-lane-task"));
    }

    #[test]
    fn g06_soak_lane_json_reports_required_and_deferred_policy() {
        let rendered = render_g06_soak_lane_json();
        assert!(rendered.contains("\"lane\":\"signal.g06.long-session-soak-lane\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/031-long-session-soak-promotion-gate-and-loophole-readiness-contract.md\""
        ));
        assert!(rendered.contains("\"acceptance_task\":\"effigy acceptance:g06-soak-lane\""));
        assert!(rendered.contains("\"id\":\"required-local-soak-export\""));
        assert!(rendered.contains("\"status\":\"required\""));
        assert!(rendered.contains("\"id\":\"deferred-server-soak-export\""));
        assert!(rendered.contains("\"status\":\"deferred\""));
        assert!(rendered.contains("\"id\":\"g06-soak-lane-proof\""));
    }

    #[test]
    fn host_edge_boundary_text_reports_stable_and_unstable_edges() {
        let rendered = render_host_edge_boundary_text();
        assert!(rendered.contains("host_edge_boundary: signal.host.edge.boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:host-edge-consumer"));
        assert!(rendered.contains("surface: RuntimeSupervisorApi implemented by both hosts"));
        assert!(rendered.contains("surface: supervisor_report() -> RuntimeSupervisorReport"));
        assert!(rendered.contains("tier: consumer-facing-but-unstable"));
        assert!(rendered.contains("surface: boot_* fault, recovery, watchdog, and soak helpers"));
    }

    #[test]
    fn host_edge_boundary_json_reports_stable_and_unstable_edges() {
        let rendered = render_host_edge_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.host.edge.boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/009-shared-host-convenience-api-and-consumer-edge-contract.md\""
        ));
        assert!(rendered.contains("\"acceptance_task\":\"effigy acceptance:host-edge-consumer\""));
        assert!(rendered.contains("\"id\":\"shared-runtime-supervisor-api\""));
        assert!(rendered.contains("\"id\":\"shared-supervisor-report\""));
        assert!(rendered.contains("\"id\":\"host-summary-dtos\""));
        assert!(rendered.contains("\"tier\":\"scenario-only\""));
    }

    #[test]
    fn release_boundary_text_reports_packaging_baseline() {
        let rendered = render_release_boundary_text();
        assert!(rendered.contains("release_boundary: signal.release.boundary"));
        assert!(rendered.contains("release_version: 0.1.0"));
        assert!(rendered.contains("version_source: workspace.package.version"));
        assert!(rendered.contains("changelog_path: CHANGELOG.md"));
        assert!(rendered.contains("conformance_task: effigy acceptance:conformance"));
        assert!(rendered
            .contains("cargo run -p signal-supervisor-tools -- --describe-export --format=json"));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-packaging-manifest --format=json"
        ));
        assert!(rendered.contains(
            "publication packaging beyond the repo-owned manifest descriptor and receipt inventory"
        ));
    }

    #[test]
    fn release_boundary_json_reports_packaging_baseline() {
        let rendered = render_release_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.release.boundary\""));
        assert!(rendered.contains("\"release_version\":\"0.1.0\""));
        assert!(rendered.contains("\"version_source\":\"workspace.package.version\""));
        assert!(rendered.contains("\"changelog_path\":\"CHANGELOG.md\""));
        assert!(rendered.contains("\"conformance_task\":\"effigy acceptance:conformance\""));
        assert!(rendered.contains("\"id\":\"workspace-changelog\""));
        assert!(rendered.contains("\"id\":\"consumer-conformance\""));
        assert!(rendered.contains("\"id\":\"supervisor-export-description\""));
        assert!(rendered.contains("\"id\":\"publication-packaging-manifest\""));
    }

    #[test]
    fn packaging_manifest_text_reports_release_bundle_and_receipts() {
        let rendered = render_packaging_manifest_text();
        assert!(rendered.contains("packaging_manifest: signal.release.packaging-manifest"));
        assert!(rendered.contains("release_version: 0.1.0"));
        assert!(rendered.contains(
            "contract_path: docs/contracts/010-publication-grade-packaging-manifest-and-release-receipt-contract.md"
        ));
        assert!(rendered.contains("acceptance_task: effigy acceptance:release-packaging-consumer"));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-host-edge-boundary --format=json"
        ));
        assert!(rendered.contains("id: manifest-generation-receipt"));
        assert!(rendered.contains("id: validation-receipt"));
        assert!(rendered.contains("crates.io publication and registry upload automation"));
    }

    #[test]
    fn packaging_manifest_json_reports_release_bundle_and_receipts() {
        let rendered = render_packaging_manifest_json();
        assert!(rendered.contains("\"manifest\":\"signal.release.packaging-manifest\""));
        assert!(rendered.contains("\"release_version\":\"0.1.0\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/010-publication-grade-packaging-manifest-and-release-receipt-contract.md\""
        ));
        assert!(rendered
            .contains("\"acceptance_task\":\"effigy acceptance:release-packaging-consumer\""));
        assert!(rendered.contains("\"id\":\"release-boundary-descriptor\""));
        assert!(rendered.contains("\"id\":\"manifest-generation-receipt\""));
        assert!(rendered.contains("\"id\":\"validation-receipt\""));
        assert!(rendered.contains("\"id\":\"release-boundary-baseline\""));
    }

    #[test]
    fn downstream_automation_text_reports_mandatory_and_optional_fixtures() {
        let rendered = render_downstream_automation_text();
        assert!(rendered.contains("downstream_automation_boundary: signal.downstream.automation"));
        assert!(rendered.contains("mandatory_release_task: effigy acceptance:downstream-release"));
        assert!(rendered.contains("optional_depth_task: effigy acceptance:downstream-depth"));
        assert!(rendered.contains("id: release-packaging-consumer"));
        assert!(rendered.contains("id: local-mixed-watchdog-export"));
        assert!(rendered.contains(
            "signal.supervisor.export JSON with profiling_receipt, soak_receipt, and supervisor_report"
        ));
    }

    #[test]
    fn downstream_automation_json_reports_mandatory_and_optional_fixtures() {
        let rendered = render_downstream_automation_json();
        assert!(rendered.contains("\"boundary\":\"signal.downstream.automation\""));
        assert!(rendered
            .contains("\"mandatory_release_task\":\"effigy acceptance:downstream-release\""));
        assert!(rendered.contains("\"optional_depth_task\":\"effigy acceptance:downstream-depth\""));
        assert!(rendered.contains("\"combined_task\":\"effigy acceptance:downstream-automation\""));
        assert!(rendered.contains("\"id\":\"downstream-automation-descriptor\""));
        assert!(rendered.contains("\"id\":\"local-soak-export\""));
        assert!(rendered.contains("\"id\":\"analysis-acceptance\""));
    }

    #[test]
    fn downstream_fail_gates_text_reports_required_and_deferred_policy() {
        let rendered = render_downstream_fail_gates_text();
        assert!(rendered.contains("downstream_fail_gates: signal.downstream.fail-gates"));
        assert!(rendered.contains("fail_gate_task: effigy acceptance:downstream-gate"));
        assert!(rendered.contains("id: mandatory-release-gate"));
        assert!(rendered.contains("blocks_release: true"));
        assert!(rendered.contains("id: optional-depth-lane"));
        assert!(rendered.contains("blocks_release: false"));
        assert!(rendered.contains("id: server-soak-export"));
    }

    #[test]
    fn downstream_fail_gates_json_reports_required_and_deferred_policy() {
        let rendered = render_downstream_fail_gates_json();
        assert!(rendered.contains("\"boundary\":\"signal.downstream.fail-gates\""));
        assert!(rendered.contains("\"fail_gate_task\":\"effigy acceptance:downstream-gate\""));
        assert!(rendered.contains("\"id\":\"mandatory-release-gate\""));
        assert!(rendered.contains("\"blocks_release\":true"));
        assert!(rendered.contains("\"id\":\"optional-depth-lane\""));
        assert!(rendered.contains("\"blocks_release\":false"));
        assert!(rendered.contains("\"id\":\"server-soak-export\""));
        assert!(rendered.contains("\"status\":\"deferred\""));
    }

    #[test]
    fn generation_closeout_text_reports_combined_boundary_and_next_queue() {
        let rendered = render_generation_closeout_text();
        assert!(rendered.contains("generation_closeout: signal.generation.closeout"));
        assert!(rendered.contains("generation: g08"));
        assert!(rendered.contains(
            "contract_path: docs/contracts/071-generation-closeout-and-downstream-workflow-readiness-gate-contract.md"
        ));
        assert!(rendered.contains(
            "roadmap_path: docs/roadmaps/g08/020-generation-closeout-and-downstream-workflow-readiness-gate.md"
        ));
        assert!(rendered.contains("closeout_task: effigy acceptance:g08-closeout"));
        assert!(rendered.contains("promotion_decision: close-g08-and-handoff-to-post-g08-backlog"));
        assert!(rendered.contains("closeout_gate_status: complete"));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-integrated-live-workflow-acceptance-lane --format=json"
        ));
        assert!(rendered.contains(
            "next_queue_path: docs/roadmaps/backlog/post-g08-repeated-run-environment-matrices-and-downstream-workflow-depth.md"
        ));
        assert!(rendered.contains("next_queue_status: backlog"));
        assert!(rendered.contains("id: linux-live-and-guarded-ownership-surface"));
        assert!(rendered.contains("status: sufficient-for-closeout"));
        assert!(rendered.contains("g08 is closed."));
    }

    #[test]
    fn generation_closeout_json_reports_combined_boundary_and_next_queue() {
        let rendered = render_generation_closeout_json();
        assert!(rendered.contains("\"closeout\":\"signal.generation.closeout\""));
        assert!(rendered.contains("\"generation\":\"g08\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/071-generation-closeout-and-downstream-workflow-readiness-gate-contract.md\""
        ));
        assert!(rendered.contains(
            "\"roadmap_path\":\"docs/roadmaps/g08/020-generation-closeout-and-downstream-workflow-readiness-gate.md\""
        ));
        assert!(rendered.contains("\"closeout_task\":\"effigy acceptance:g08-closeout\""));
        assert!(rendered
            .contains("\"promotion_decision\":\"close-g08-and-handoff-to-post-g08-backlog\""));
        assert!(rendered.contains("\"closeout_gate_status\":\"complete\""));
        assert!(rendered.contains(
            "\"g08_integrated_acceptance_lane_command\":\"cargo run -p signal-supervisor-tools -- --describe-integrated-live-workflow-acceptance-lane --format=json\""
        ));
        assert!(rendered.contains(
            "\"next_queue_path\":\"docs/roadmaps/backlog/post-g08-repeated-run-environment-matrices-and-downstream-workflow-depth.md\""
        ));
        assert!(rendered.contains("\"next_queue_status\":\"backlog\""));
        assert!(rendered.contains("\"id\":\"integrated-acceptance-base\""));
        assert!(rendered.contains("\"id\":\"closeout-descriptor-proof\""));
        assert!(rendered.contains("\"id\":\"generation-closeout-description\""));
        assert!(rendered.contains("\"id\":\"linux-live-and-guarded-ownership-surface\""));
        assert!(rendered.contains("\"status\":\"sufficient-for-closeout\""));
        assert!(rendered.contains(
            "\"broader repeated-run and environment-specific acceptance depth remain outside the bounded g08 closeout fast path and are now explicit post-g08 backlog work instead of implied follow-up\""
        ));
    }

    #[test]
    fn export_json_is_versioned() {
        let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let report = RuntimeSupervisorReport::capture(&runtime, &Default::default());
        let profiling = report.profiling_receipt();
        let soak = report.soak_receipt();
        let export = render_supervisor_export_json(
            HostProfile::Local,
            Scenario::Default,
            "{}".into(),
            &profiling,
            &soak,
            &report,
        );
        assert!(export.contains("\"schema\":\"signal.supervisor.export\""));
        assert!(export.contains("\"schema_version\":1"));
        assert!(export.contains("\"profiling_receipt\":{"));
        assert!(export.contains("\"soak_receipt\":{"));
    }

    #[test]
    fn export_json_carries_last_deferred_service_receipt() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        runtime
            .set_safe_mode(SafeModeRequest { enabled: true })
            .expect("enable safe mode");
        let purge_receipt = runtime
            .purge_offline_render_artifacts(RuntimeOfflineRenderPurgeRequest {
                request_id: "purge:export-proof".into(),
                artifact_root_path: Some("/tmp/nonexistent-artifacts".into()),
                report_path: Some("/tmp/nonexistent-report.json".into()),
            })
            .expect("safe mode should defer purge export proof");
        assert!(!purge_receipt.purged_report);
        assert!(!purge_receipt.purged_artifact_root);

        let report = RuntimeSupervisorReport::capture(&runtime, &Default::default());
        let profiling = report.profiling_receipt();
        let soak = report.soak_receipt();
        let export = render_supervisor_export_json(
            HostProfile::Local,
            Scenario::Default,
            "{}".into(),
            &profiling,
            &soak,
            &report,
        );

        assert!(export.contains("\"last_deferred_service\":{"));
        assert!(export.contains("\"work_class\":\"OfflineRenderPurge\""));
        assert!(export.contains("\"decision\":\"Defer\""));
        assert!(export.contains("\"reason\":\"SafeMode\""));
    }

    #[test]
    fn export_json_carries_runtime_owned_plugin_discovery_catalog() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
            roots: vec!["~/Library/Audio/Plug-Ins/CLAP".into()],
            formats: vec![PluginFormat::Clap],
        });
        runtime.record_plugin_scan_results(
            scan_handle,
            vec![
                sample_discovered_type_record(),
                sample_backend_breadth_record(),
            ],
        );
        runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
            sandbox_id: "export-consumer-sandbox".into(),
            plugin_format: PluginFormat::Clap,
            plugin_type_id: None,
        });
        runtime.record_plugin_sandbox_lifecycle(
            "export-consumer-sandbox",
            PluginSandboxLifecycleStage::SandboxEnsured,
            None,
        );

        let report = RuntimeSupervisorReport::capture(&runtime, &Default::default());
        let profiling = report.profiling_receipt();
        let soak = report.soak_receipt();
        let export = render_supervisor_export_json(
            HostProfile::Local,
            Scenario::Default,
            "{}".into(),
            &profiling,
            &soak,
            &report,
        );

        assert!(export.contains("\"host_summary\":{}"));
        assert!(export.contains("\"supervisor_report\":{"));
        assert!(export.contains("\"plugin_discovery_snapshot\":{"));
        assert!(export.contains("\"discovered_type_count\":2"));
        assert!(export.contains("\"discovered_format_count\":2"));
        assert!(export.contains("\"plugin_type_id\":\"plugin:clap:export-consumer\""));
        assert!(export.contains("\"plugin_type_id\":\"plugin:vst3:export-instrument\""));
        assert!(export.contains("\"format\":\"Clap\""));
        assert!(export.contains("\"multi_format_catalog\":true"));
        assert!(export.contains("\"requires_main_thread_for_state_count\":1"));
        assert!(export.contains("\"format_coverage\":["));
        assert!(export.contains("\"supports_snapshot\":true"));
        assert!(export.contains("\"supports_activate\":true"));
    }

    #[test]
    fn export_json_carries_runtime_owned_plugin_discovery_capability_coverage() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
            roots: vec!["~/Library/Audio/Plug-Ins".into()],
            formats: vec![PluginFormat::Clap, PluginFormat::Vst3],
        });
        runtime.record_plugin_scan_results(
            scan_handle,
            vec![
                sample_discovered_type_record(),
                sample_backend_breadth_record(),
            ],
        );

        let report = RuntimeSupervisorReport::capture(&runtime, &Default::default());
        let export = render_supervisor_export_json(
            HostProfile::Local,
            Scenario::Default,
            "{}".into(),
            &report.profiling_receipt(),
            &report.soak_receipt(),
            &report,
        );

        assert!(export.contains("\"discovered_format_count\":2"));
        assert!(export.contains("\"multi_format_catalog\":true"));
        assert!(export.contains("\"requires_main_thread_for_state_count\":1"));
        assert!(export.contains("\"max_parameter_count\":24"));
        assert!(export.contains("\"format\":\"Vst3\""));
        assert!(export.contains("\"instrument_count\":1"));
    }

    #[test]
    fn export_json_carries_runtime_recovery_sequence() {
        let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let mut recorder = RuntimeEventRecorder::default();
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::RecoveryCycle {
                sandbox_id: "sandbox-1".into(),
                intent: RecoveryRestartIntent::WatchdogRecovery,
                stop_reason: StopReason::DegradedModeRecovery,
                processing_epoch: Some(4),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxLifecycle {
                sandbox_id: "sandbox-1".into(),
                stage: PluginSandboxLifecycleStage::TransportAttached,
                processing_epoch: Some(4),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-1".into(),
                lease_id: "lease-4".into(),
                region_id: "region-4".into(),
                stage: PluginSandboxTransportStage::Attached,
                processing_epoch: Some(4),
                detail: None,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::HeartbeatCycle {
                sandbox_id: "sandbox-1".into(),
                stage: HeartbeatCycleStage::Responded,
                processing_epoch: Some(4),
                block_sequence: Some(9),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::BlockDispatch {
                sandbox_id: "sandbox-1".into(),
                lease_id: "lease-4".into(),
                processing_epoch: 4,
                block_sequence: 9,
                frame_count: 512,
                stage: BlockDispatchStage::Completed,
                completion_state: Some(CompletionState::Completed),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::LeaseRollover {
                sandbox_id: "sandbox-1".into(),
                previous_lease_id: "lease-3".into(),
                lease_id: "lease-4".into(),
                processing_epoch: 4,
                first_block_sequence: 9,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::BrokerInvalidation {
                sandbox_id: "sandbox-1".into(),
                lease_id: "lease-4".into(),
                processing_epoch: 4,
                block_sequence: Some(9),
                stage: BrokerInvalidationStage::CompletionRegionInvalidated,
                reason: "watchdog recovery teardown".into(),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::CompletionSlotTransition {
                sandbox_id: "sandbox-1".into(),
                lease_id: "lease-4".into(),
                processing_epoch: 4,
                block_sequence: 9,
                stage: CompletionSlotStage::TimedOut,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::CompletionSlotTransition {
                sandbox_id: "sandbox-1".into(),
                lease_id: "lease-4".into(),
                processing_epoch: 4,
                block_sequence: 9,
                stage: CompletionSlotStage::FallbackApplied,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::BrokerFailure {
                sandbox_id: "sandbox-1".into(),
                lease_id: Some("lease-4".into()),
                processing_epoch: Some(4),
                block_sequence: Some(9),
                stage: BrokerFailureStage::PayloadRead,
                detail: "failed to attach shared-memory region: stale mapping".into(),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-1".into(),
                lease_id: "lease-4".into(),
                region_id: "region-4".into(),
                stage: PluginSandboxTransportStage::DetachRequested,
                processing_epoch: Some(4),
                detail: None,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-1".into(),
                lease_id: "lease-4".into(),
                region_id: "region-4".into(),
                stage: PluginSandboxTransportStage::Detached,
                processing_epoch: Some(4),
                detail: None,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-1".into(),
                lease_id: "lease-4".into(),
                region_id: "region-4".into(),
                stage: PluginSandboxTransportStage::DetachFault,
                processing_epoch: Some(4),
                detail: Some("broker detach fault: stale region mapping".into()),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::SandboxOperationFailure {
                sandbox_id: "sandbox-1".into(),
                lease_id: Some("lease-4".into()),
                processing_epoch: Some(4),
                operation: "processBlock".into(),
                error_kind: "resourceUnavailable".into(),
                stage: SandboxOperationFailureStage::ProcessAttach,
                detail: "failed to attach shared-memory region: stale mapping".into(),
            },
        );
        let report = RuntimeSupervisorReport::capture(&runtime, &recorder);
        let profiling = report.profiling_receipt();
        let soak = report.soak_receipt();
        let export = render_supervisor_export_json(
            HostProfile::Local,
            Scenario::Soak,
            "{}".into(),
            &profiling,
            &soak,
            &report,
        );
        assert!(export.contains("\"recovery_events\":1"));
        assert!(export.contains("\"recovery_sequence\":[{"));
        assert!(export.contains("\"intent\":\"WatchdogRecovery\""));
        assert!(export.contains("\"last_recovery_intent\":\"WatchdogRecovery\""));
        assert!(export.contains("\"lifecycle_events\":1"));
        assert!(export.contains("\"lifecycle_sequence\":[{"));
        assert!(export.contains("\"stage\":\"TransportAttached\""));
        assert!(export.contains("\"transport_events\":4"));
        assert!(export.contains("\"transport_sequence\":[{"));
        assert!(export.contains("\"region_id\":\"region-4\""));
        assert!(export.contains("\"heartbeat_events\":1"));
        assert!(export.contains("\"heartbeat_sequence\":[{"));
        assert!(export.contains("\"block_sequence\":9"));
        assert!(export.contains("\"block_dispatch_events\":1"));
        assert!(export.contains("\"block_dispatch_sequence\":[{"));
        assert!(export.contains("\"completion_state\":\"Completed\""));
        assert!(export.contains("\"lease_rollover_events\":1"));
        assert!(export.contains("\"lease_rollover_sequence\":[{"));
        assert!(export.contains("\"previous_lease_id\":\"lease-3\""));
        assert!(export.contains("\"invalidation_events\":1"));
        assert!(export.contains("\"invalidation_sequence\":[{"));
        assert!(export.contains("\"stage\":\"CompletionRegionInvalidated\""));
        assert!(export.contains("\"completion_slot_events\":2"));
        assert!(export.contains("\"completion_slot_sequence\":[{"));
        assert!(export.contains("\"stage\":\"FallbackApplied\""));
        assert!(export.contains("\"transport_fault_events\":8"));
        assert!(export.contains("\"last_transport_fault\":{"));
        assert!(export.contains("\"transport_fault_sequence\":[{"));
        assert!(export.contains("\"source\":\"HostBroker\""));
        assert!(export.contains("\"source\":\"SandboxOperation\""));
        assert!(export.contains("\"source\":\"RuntimeDispatch\""));
        assert!(export.contains("\"phase\":\"Dispatch\""));
        assert!(export.contains("\"phase\":\"Teardown\""));
        assert!(export.contains("\"resource\":\"SharedMemoryPayload\""));
        assert!(export.contains("\"resource\":\"SharedMemoryLease\""));
        assert!(export.contains("\"resource\":\"CompletionSlot\""));
        assert!(export.contains("\"operation\":\"block_payload.read\""));
        assert!(export.contains("\"operation\":\"transport.detach_request\""));
        assert!(export.contains("\"operation\":\"transport.detached\""));
        assert!(export.contains("\"operation\":\"transport.detach_fault\""));
        assert!(export.contains("\"operation\":\"completion_region.invalidate\""));
        assert!(export.contains("\"operation\":\"completion_slot.timeout\""));
        assert!(export.contains("\"operation\":\"completion_slot.fallback_apply\""));
        assert!(export.contains("\"operation\":\"processBlock\""));
        assert!(export.contains("\"stage\":\"TransportDetachRequested\""));
        assert!(export.contains("\"stage\":\"TransportDetached\""));
        assert!(export.contains("\"stage\":\"DetachFault\""));
        assert!(export.contains("\"stage\":\"CompletionRegionInvalidated\""));
        assert!(export.contains("\"stage\":\"CompletionSlotTimedOut\""));
        assert!(export.contains("\"stage\":\"FallbackApplied\""));
        assert!(export.contains("\"transport_fault_summary\":{"));
        assert!(export.contains("\"boundary_mode\":\"FaultAdjacentOnly\""));
        assert!(export.contains("\"host_broker_events\":4"));
        assert!(export.contains("\"sandbox_operation_events\":1"));
        assert!(export.contains("\"runtime_dispatch_events\":3"));
        assert!(export.contains("\"dispatch_events\":"));
        assert!(export.contains("\"teardown_events\":"));
        assert!(export.contains("\"transport_concurrency_snapshot\":{"));
        assert!(export.contains("\"steady_session_limit\":1"));
        assert!(export.contains("\"recovery_session_limit\":2"));
        assert!(export.contains("\"current_attached_sessions\":0"));
        assert!(export.contains("\"current_lingering_sessions\":0"));
        assert!(export.contains("\"peak_lingering_sessions\":0"));
        assert!(export.contains("\"current_detach_requested_sessions\":0"));
        assert!(export.contains("\"current_detach_faulted_sessions\":0"));
        assert!(export.contains("\"transport_session_summary\":{"));
        assert!(export.contains("\"boundary_mode\":\"HealthyPathVisible\""));
        assert!(export.contains("\"current_state\":\"DetachFaulted\""));
        assert!(export.contains("\"currently_attached\":false"));
        assert!(export.contains("\"heartbeat_freshness\":\"Fresh\""));
        assert!(export.contains("\"dispatch_state\":\"Completed\""));
        assert!(export.contains("\"current_attached_session_count\":0"));
        assert!(export.contains("\"max_concurrent_attached_sessions\":1"));
        assert!(export.contains("\"attach_events\":1"));
        assert!(export.contains("\"detach_requested_events\":1"));
        assert!(export.contains("\"detached_events\":1"));
        assert!(export.contains("\"detach_fault_events\":1"));
        assert!(export.contains("\"heartbeat_responded_events\":1"));
        assert!(export.contains("\"dispatch_completed_events\":1"));
        assert!(export.contains("\"active_sandbox_id\":null"));
        assert!(export.contains("\"active_lease_id\":null"));
        assert!(export.contains("\"active_region_id\":null"));
        assert!(export.contains("\"active_block_sequence\":"));
        assert!(export.contains("\"active_sessions\":[]"));
        assert!(export.contains("\"last_region_id\":\"region-4\""));
        assert!(export.contains("\"broker_failure_events\":1"));
        assert!(export.contains("\"broker_failure_sequence\":[{"));
        assert!(export.contains("\"stage\":\"PayloadRead\""));
        assert!(export.contains("\"sandbox_operation_failure_events\":1"));
        assert!(export.contains("\"sandbox_operation_failure_sequence\":[{"));
        assert!(export.contains("\"stage\":\"ProcessAttach\""));
    }

    #[test]
    fn export_json_carries_cross_family_integrated_acceptance_evidence() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        runtime
            .handshake(HandshakeRequest {
                client_version: "integrated-acceptance-export".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(96_000),
            })
            .expect("integrated acceptance export handshake should succeed");
        runtime
            .configure(RuntimeConfigRequest::new(48_000, 512))
            .expect("integrated acceptance export configure should succeed");
        runtime
            .start()
            .expect("integrated acceptance export start should succeed");

        runtime.record_watchdog_restart(WatchdogRestartRecord {
            sandbox_id: "integrated-acceptance-sandbox".into(),
            trigger: RuntimeWatchdogTrigger::HeartbeatMisses,
            processing_epoch: 1,
        });
        runtime.record_watchdog_restart(WatchdogRestartRecord {
            sandbox_id: "integrated-acceptance-sandbox".into(),
            trigger: RuntimeWatchdogTrigger::DeadlineMisses,
            processing_epoch: 2,
        });

        runtime.record_plugin_format_platform_coverage(vec![
            RuntimePluginFormatPlatformCoverageRecord {
                format: PluginFormat::Clap,
                supported_platforms: vec![
                    RuntimePluginHostPlatform::MacOs,
                    RuntimePluginHostPlatform::Linux,
                    RuntimePluginHostPlatform::Windows,
                ],
                unsupported_platforms: Vec::new(),
                linux_parity_band: RuntimePluginParityBand::Portable,
                linux_preferred_sandbox_outcome: Some(RuntimePluginIsolationOutcome::IsolatedSandbox),
                linux_strict_sandbox_default: true,
                summary:
                    "platforms=MacOs/Linux/Windows linux=Portable linux_policy=IsolatedSandbox unsupported=none"
                        .into(),
            },
            RuntimePluginFormatPlatformCoverageRecord {
                format: PluginFormat::Vst3,
                supported_platforms: vec![
                    RuntimePluginHostPlatform::MacOs,
                    RuntimePluginHostPlatform::Linux,
                    RuntimePluginHostPlatform::Windows,
                ],
                unsupported_platforms: Vec::new(),
                linux_parity_band: RuntimePluginParityBand::Portable,
                linux_preferred_sandbox_outcome: Some(RuntimePluginIsolationOutcome::IsolatedSandbox),
                linux_strict_sandbox_default: true,
                summary:
                    "platforms=MacOs/Linux/Windows linux=Portable linux_policy=IsolatedSandbox unsupported=none"
                        .into(),
            },
            RuntimePluginFormatPlatformCoverageRecord {
                format: PluginFormat::Au,
                supported_platforms: vec![RuntimePluginHostPlatform::MacOs],
                unsupported_platforms: vec![
                    RuntimePluginHostPlatform::Linux,
                    RuntimePluginHostPlatform::Windows,
                ],
                linux_parity_band: RuntimePluginParityBand::Unsupported,
                linux_preferred_sandbox_outcome: None,
                linux_strict_sandbox_default: false,
                summary: "platforms=MacOs linux=Unsupported unsupported=Linux/Windows".into(),
            },
        ]);
        let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
            roots: vec![
                "~/.clap".into(),
                "~/.vst3".into(),
                "~/Library/Audio/Plug-Ins/Components".into(),
            ],
            formats: vec![PluginFormat::Clap, PluginFormat::Vst3, PluginFormat::Au],
        });
        runtime.record_plugin_scan_results(
            scan_handle,
            vec![
                sample_discovered_type_record(),
                sample_backend_breadth_record(),
                sample_au_breadth_record(),
            ],
        );
        runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
            sandbox_id: "integrated-acceptance-vst3".into(),
            plugin_format: PluginFormat::Vst3,
            plugin_type_id: Some("plugin:vst3:export-instrument".into()),
        });
        runtime.record_plugin_sandbox_lifecycle(
            "integrated-acceptance-vst3",
            PluginSandboxLifecycleStage::InstancePrepared,
            Some(2),
        );

        runtime
            .set_safe_mode(SafeModeRequest { enabled: true })
            .expect("integrated acceptance export safe mode should enable");
        let deferred = runtime
            .render_offline_queue(vec![RuntimeOfflineRenderRequest {
                request_id: "render:integrated-acceptance".into(),
                timeline_start_samples: 0,
                duration_samples: 64,
                export_sample_rate_hz: 48_000,
                include_main_mix: true,
                artifact_root_path: None,
                stem_targets: Vec::new(),
                freeze_artifacts: Vec::new(),
            }])
            .expect("integrated acceptance export queue should defer in safe mode");
        assert_eq!(deferred.orchestration.deferred_work_item_count, 1);

        let ready_path = integrated_acceptance_media_fixture_path("ready");
        let missing_path = integrated_acceptance_media_fixture_path("missing");
        write_integrated_acceptance_test_wav(&ready_path);
        runtime
            .reconcile_media_assets(vec![
                RuntimeMediaAssetRegistration {
                    asset_id: "asset:sha256:integrated-ready".into(),
                    content_hash: "integrated-ready".into(),
                    source_path: ready_path.display().to_string(),
                    file_name: "integrated-ready.wav".into(),
                    byte_size: fs::metadata(&ready_path)
                        .expect("integrated acceptance media fixture should exist")
                        .len(),
                    sample_rate_hz: 48_000,
                    channel_count: 1,
                    duration_samples: 128,
                    waveform_bin_count: 16,
                },
                RuntimeMediaAssetRegistration {
                    asset_id: "asset:sha256:integrated-missing".into(),
                    content_hash: "integrated-missing".into(),
                    source_path: missing_path.display().to_string(),
                    file_name: "integrated-missing.wav".into(),
                    byte_size: 0,
                    sample_rate_hz: 48_000,
                    channel_count: 1,
                    duration_samples: 128,
                    waveform_bin_count: 16,
                },
            ])
            .expect("integrated acceptance media assets should reconcile");
        runtime
            .start_media_preview("asset:sha256:integrated-ready")
            .expect("integrated acceptance media preview should start");

        let recorder = RuntimeEventRecorder::default();
        let mut report = RuntimeSupervisorReport::capture(&runtime, &recorder);
        report.observation = report
            .observation
            .clone()
            .with_host_external_io(&sample_integrated_acceptance_host_io());
        let export = render_supervisor_export_json(
            HostProfile::Local,
            Scenario::Mixed,
            "{}".into(),
            &report.profiling_receipt(),
            &report.soak_receipt(),
            &report,
        );

        assert!(export.contains("\"fault_status\":{"));
        assert!(export.contains("\"primary_fault_cause\":\"WatchdogRestart\""));
        assert!(export.contains("\"interruption_summary\":{"));
        assert!(export.contains("\"watchdog_restart_count\":2"));
        assert!(export.contains("\"fault_diagnostic_receipt\":{"));
        assert!(export.contains("\"primary_family\":\"DeferredWorkPressure\""));
        assert!(export.contains("\"last_deferred_service\":{"));
        assert!(export.contains("\"decision\":\"Defer\""));
        assert!(export.contains("\"plugin_discovery_snapshot\":{"));
        assert!(export.contains("\"plugin_type_id\":\"plugin:clap:export-consumer\""));
        assert!(export.contains("\"plugin_type_id\":\"plugin:vst3:export-instrument\""));
        assert!(export.contains("\"plugin_type_id\":\"plugin:au:export-au\""));
        assert!(export.contains("\"parity_coverage\":[{"));
        assert!(export.contains("\"supported_platforms\":[\"MacOs\"]"));
        assert!(export.contains("\"unsupported_platforms\":[\"Linux\",\"Windows\"]"));
        assert!(export.contains("\"device_supervision_snapshot\":{"));
        assert!(export.contains("\"external_io_snapshot\":{"));
        assert!(export.contains("\"monitoring_state\":\"Guarded\""));
        assert!(export.contains("\"drift_state\":\"CrossClockManaged\""));
        assert!(export.contains("\"duplex_mismatch_state\":\"CrossClockDiverged\""));
        assert!(export.contains("\"endpoint_topology\":\"Duplex\""));
        assert!(export.contains("\"media_pipeline_snapshot\":{"));
        assert!(export.contains("\"media_service_snapshot\":{"));
        assert!(export.contains("\"preview_state\":\"Previewing\""));
        assert!(export.contains("\"invalidated_asset_count\":1"));
        assert!(export.contains("\"media_library_snapshot\":{"));
        assert!(export.contains("\"ready_descriptor_count\":1"));
        assert!(export.contains("\"loudness_ready_descriptor_count\":1"));
        assert!(export.contains("\"character_ready_descriptor_count\":1"));

        let _ = fs::remove_file(&ready_path);
        if let Some(path) = runtime
            .get_media_pipeline_snapshot()
            .assets
            .iter()
            .find(|asset| asset.asset_id == "asset:sha256:integrated-ready")
            .and_then(|asset| asset.cache_path.as_deref())
        {
            let _ = fs::remove_file(path);
        }
    }

    #[test]
    fn export_json_serializes_per_session_transport_liveness() {
        let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let mut recorder = RuntimeEventRecorder::default();
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-a".into(),
                region_id: "region-a".into(),
                stage: PluginSandboxTransportStage::Attached,
                processing_epoch: Some(2),
                detail: None,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-b".into(),
                lease_id: "lease-b".into(),
                region_id: "region-b".into(),
                stage: PluginSandboxTransportStage::Attached,
                processing_epoch: Some(3),
                detail: None,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-a".into(),
                region_id: "region-a".into(),
                stage: PluginSandboxTransportStage::DetachRequested,
                processing_epoch: Some(4),
                detail: None,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::HeartbeatCycle {
                sandbox_id: "sandbox-a".into(),
                stage: HeartbeatCycleStage::Missed,
                processing_epoch: Some(4),
                block_sequence: Some(11),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::HeartbeatCycle {
                sandbox_id: "sandbox-b".into(),
                stage: HeartbeatCycleStage::Responded,
                processing_epoch: Some(5),
                block_sequence: Some(12),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::BlockDispatch {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-a".into(),
                processing_epoch: 4,
                block_sequence: 11,
                frame_count: 512,
                stage: BlockDispatchStage::TimedOut,
                completion_state: Some(CompletionState::TimedOut),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::BlockDispatch {
                sandbox_id: "sandbox-b".into(),
                lease_id: "lease-b".into(),
                processing_epoch: 5,
                block_sequence: 12,
                frame_count: 512,
                stage: BlockDispatchStage::Completed,
                completion_state: Some(CompletionState::Completed),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::CompletionSlotTransition {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-a".into(),
                processing_epoch: 4,
                block_sequence: 11,
                stage: CompletionSlotStage::TimedOut,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::BrokerFailure {
                sandbox_id: "sandbox-b".into(),
                lease_id: Some("lease-b".into()),
                processing_epoch: Some(5),
                block_sequence: Some(12),
                stage: BrokerFailureStage::PayloadRead,
                detail: "stale shared-memory mapping".into(),
            },
        );

        let report = RuntimeSupervisorReport::capture(&runtime, &recorder);
        assert_eq!(
            report.observation.transport_session_summary.active_sessions[0].state,
            TransportSessionState::DetachRequested
        );
        assert_eq!(
            report.observation.transport_session_summary.active_sessions[0].heartbeat_freshness,
            TransportHeartbeatFreshness::Missed
        );
        assert_eq!(
            report.observation.transport_session_summary.active_sessions[0].dispatch_state,
            TransportDispatchState::TimedOut
        );
        assert_eq!(
            report.observation.transport_session_summary.active_sessions[1].heartbeat_freshness,
            TransportHeartbeatFreshness::Fresh
        );
        assert!(
            report.observation.transport_session_summary.active_sessions[0].transport_fault_count
                >= 1
        );
        assert!(
            report.observation.transport_session_summary.active_sessions[1].transport_fault_count
                >= 1
        );

        let profiling = report.profiling_receipt();
        let soak = report.soak_receipt();
        let export = render_supervisor_export_json(
            HostProfile::Local,
            Scenario::Mixed,
            "{}".into(),
            &profiling,
            &soak,
            &report,
        );
        assert!(export.contains("\"active_sessions\":[{"));
        assert!(export.contains("\"sandbox_id\":\"sandbox-a\""));
        assert!(export.contains("\"state\":\"DetachRequested\""));
        assert!(export.contains("\"currently_attached\":true"));
        assert!(export.contains("\"heartbeat_freshness\":\"Missed\""));
        assert!(export.contains("\"dispatch_state\":\"TimedOut\""));
        assert!(export.contains("\"peak_attached_sessions\":"));
        assert!(export.contains("\"active_block_sequence\":11"));
        assert!(export.contains("\"transport_fault_count\":1"));
        assert!(export.contains("\"last_transport_fault_source\":\"RuntimeDispatch\""));
        assert!(export.contains("\"last_transport_fault_stage\":\"CompletionSlotTimedOut\""));
        assert!(export.contains("\"last_transport_fault_phase\":\"Dispatch\""));
        assert!(export.contains("\"last_transport_fault_processing_epoch\":4"));
        assert!(export.contains("\"last_transport_fault_block_sequence\":11"));
        assert!(export.contains("\"sandbox_id\":\"sandbox-b\""));
        assert!(export.contains("\"heartbeat_freshness\":\"Fresh\""));
        assert!(export.contains("\"dispatch_state\":\"Completed\""));
        assert!(export.contains("\"active_block_sequence\":12"));
        assert!(export.contains("\"last_transport_fault_source\":\"HostBroker\""));
        assert!(export.contains("\"last_transport_fault_stage\":\"PayloadRead\""));
        assert!(export.contains("\"last_transport_fault_processing_epoch\":5"));
        assert!(export.contains("\"last_transport_fault_block_sequence\":12"));
    }

    #[test]
    fn local_summary_json_excludes_payload_by_default() {
        let summary = sample_local_summary();
        let rendered =
            super::render_local_summary_json(&summary, ExportDebugOptions { payload: false });
        assert!(!rendered.contains("\"payload\":{"));
        assert!(rendered.contains("\"sections\":[\"execution\",\"transport\",\"faults\"]"));
        assert!(rendered.contains("\"debug_sections_supported\":[\"payload\"]"));
        assert!(rendered.contains("\"debug_sections_enabled\":[]"));
        assert!(rendered.contains("\"last_recovery_intent\":\"WatchdogRecovery\""));
        assert!(rendered.contains("\"last_stop_reason\":\"DegradedModeRecovery\""));
    }

    #[test]
    fn local_summary_json_includes_payload_when_requested() {
        let summary = sample_local_summary();
        let rendered =
            super::render_local_summary_json(&summary, ExportDebugOptions { payload: true });
        assert!(rendered.contains("\"payload\""));
        assert!(rendered.contains("\"generated_event_bytes\""));
        assert!(
            rendered.contains("\"sections\":[\"execution\",\"transport\",\"faults\",\"payload\"]")
        );
        assert!(rendered.contains("\"debug_sections_supported\":[\"payload\"]"));
        assert!(rendered.contains("\"debug_sections_enabled\":[\"payload\"]"));
    }

    #[test]
    fn local_summary_text_reports_section_list() {
        let summary = sample_local_summary();
        let default_rendered =
            super::render_local_summary(&summary, ExportDebugOptions { payload: false });
        let payload_rendered =
            super::render_local_summary(&summary, ExportDebugOptions { payload: true });
        assert!(default_rendered.contains("sections: execution,transport,faults"));
        assert!(default_rendered.contains("debug_sections_supported: payload"));
        assert!(default_rendered.contains("debug_sections_enabled: none"));
        assert!(default_rendered.contains("last_recovery_intent=Some(WatchdogRecovery)"));
        assert!(default_rendered.contains("last_stop_reason=Some(DegradedModeRecovery)"));
        assert!(payload_rendered.contains("sections: execution,transport,faults,payload"));
        assert!(payload_rendered.contains("debug_sections_supported: payload"));
        assert!(payload_rendered.contains("debug_sections_enabled: payload"));
    }
}
