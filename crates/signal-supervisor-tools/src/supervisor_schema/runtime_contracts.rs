use crate::HostSummaryDebugSection;

pub(crate) const EXPORT_SCHEMA: &str = "signal.supervisor.export";
pub(crate) const EXPORT_SCHEMA_VERSION: u32 = 1;
pub(crate) const DEFAULT_HOST_SUMMARY_SECTIONS: &[&str] = &["execution", "transport", "faults"];
pub(crate) const SUPPORTED_DEBUG_SECTIONS: &[HostSummaryDebugSection] =
    &[HostSummaryDebugSection::Payload];
pub(crate) const INTERRUPTION_BOUNDARY: &str = "signal.runtime.interruption-boundary";
pub(crate) const INTERRUPTION_CONTRACT_PATH: &str =
    "docs/contracts/012-runtime-interruption-taxonomy-and-resumability-contract.md";
pub(crate) const INTERRUPTION_ACCEPTANCE_TASK: &str = "effigy acceptance:interruption-boundary";
pub(crate) const FAULT_DIAGNOSTIC_BOUNDARY: &str = "signal.runtime.fault-diagnostic-boundary";
pub(crate) const FAULT_DIAGNOSTIC_CONTRACT_PATH: &str =
    "docs/contracts/016-runtime-fault-cause-attribution-and-diagnostic-receipt-contract.md";
pub(crate) const FAULT_DIAGNOSTIC_ACCEPTANCE_TASK: &str =
    "effigy acceptance:fault-diagnostic-boundary";
pub(crate) const CRITICAL_PATH_BOUNDARY: &str = "signal.runtime.critical-path-boundary";
pub(crate) const CRITICAL_PATH_CONTRACT_PATH: &str =
    "docs/contracts/018-graph-critical-path-hot-node-and-worker-lane-instrumentation-contract.md";
pub(crate) const CRITICAL_PATH_ACCEPTANCE_TASK: &str = "effigy acceptance:critical-path-boundary";
pub(crate) const BLOCK_TIMING_BOUNDARY: &str = "signal.runtime.block-timing-boundary";
pub(crate) const BLOCK_TIMING_CONTRACT_PATH: &str =
    "docs/contracts/017-per-block-execution-timing-and-pressure-snapshot-contract.md";
pub(crate) const BLOCK_TIMING_ACCEPTANCE_TASK: &str = "effigy acceptance:block-timing-boundary";
pub(crate) const DEFERRED_WORK_POLICY_BOUNDARY: &str =
    "signal.runtime.deferred-work-policy-boundary";
pub(crate) const DEFERRED_WORK_POLICY_CONTRACT_PATH: &str =
    "docs/contracts/019-deferred-work-scheduler-priority-backpressure-and-cancellation-contract.md";
pub(crate) const DEFERRED_WORK_POLICY_ACCEPTANCE_TASK: &str =
    "effigy acceptance:deferred-work-policy-boundary";
pub(crate) const RECORDING_CONTINUITY_BOUNDARY: &str =
    "signal.runtime.recording-continuity-boundary";
pub(crate) const RECORDING_CONTINUITY_CONTRACT_PATH: &str =
    "docs/contracts/013-recording-continuity-midi-capture-and-checkpoint-contract.md";
pub(crate) const RECORDING_CONTINUITY_ACCEPTANCE_TASK: &str =
    "effigy acceptance:recording-continuity";
pub(crate) const OFFLINE_RENDER_CONTINUITY_BOUNDARY: &str =
    "signal.runtime.offline-render-continuity-boundary";
pub(crate) const OFFLINE_RENDER_CONTINUITY_CONTRACT_PATH: &str =
    "docs/contracts/015-offline-render-recovery-and-resumability-contract.md";
pub(crate) const OFFLINE_RENDER_CONTINUITY_ACCEPTANCE_TASK: &str =
    "effigy acceptance:offline-render-continuity";
pub(crate) const PLUGIN_CONTINUITY_BOUNDARY: &str = "signal.runtime.plugin-continuity-boundary";
pub(crate) const PLUGIN_CONTINUITY_CONTRACT_PATH: &str =
    "docs/contracts/014-plugin-isolation-policy-transport-rebind-and-shared-sandbox-continuity-contract.md";
pub(crate) const PLUGIN_CONTINUITY_ACCEPTANCE_TASK: &str = "effigy acceptance:plugin-continuity";
pub(crate) const VST3_BOUNDARY: &str = "signal.runtime.vst3-boundary";
pub(crate) const VST3_CONTRACT_PATH: &str =
    "docs/contracts/020-vst3-adapter-baseline-and-runtime-owned-lifecycle-contract.md";
pub(crate) const VST3_ACCEPTANCE_TASK: &str = "effigy acceptance:vst3-boundary";
pub(crate) const AU_BOUNDARY: &str = "signal.runtime.au-boundary";
pub(crate) const AU_CONTRACT_PATH: &str =
    "docs/contracts/021-au-adapter-baseline-and-runtime-owned-lifecycle-contract.md";
pub(crate) const AU_ACCEPTANCE_TASK: &str = "effigy acceptance:au-boundary";
pub(crate) const LV2_BOUNDARY: &str = "signal.runtime.lv2-boundary";
pub(crate) const LV2_CONTRACT_PATH: &str =
    "docs/contracts/055-lv2-worker-urid-patch-and-extension-negotiation-contract.md";
pub(crate) const LV2_ACCEPTANCE_TASK: &str = "effigy acceptance:lv2-boundary";
pub(crate) const CROSS_ADAPTER_PARITY_BOUNDARY: &str =
    "signal.runtime.cross-adapter-parity-boundary";
pub(crate) const CROSS_ADAPTER_PARITY_CONTRACT_PATH: &str =
    "docs/contracts/022-backend-capability-parity-linux-plugin-support-and-cross-adapter-conformance-contract.md";
pub(crate) const CROSS_ADAPTER_PARITY_ACCEPTANCE_TASK: &str =
    "effigy acceptance:cross-adapter-parity-boundary";
pub(crate) const LINUX_PLUGIN_PARITY_BOUNDARY: &str = "signal.runtime.linux-plugin-parity-boundary";
pub(crate) const LINUX_PLUGIN_PARITY_CONTRACT_PATH: &str =
    "docs/contracts/039-linux-cross-adapter-plugin-parity-and-sandbox-policy-contract.md";
pub(crate) const LINUX_PLUGIN_PARITY_ACCEPTANCE_TASK: &str =
    "effigy acceptance:linux-plugin-parity-boundary";
pub(crate) const LINUX_AUDIO_BACKEND_BOUNDARY: &str = "signal.runtime.linux-audio-backend-boundary";
pub(crate) const LINUX_AUDIO_BACKEND_CONTRACT_PATH: &str =
    "docs/contracts/040-linux-audio-backend-portability-across-alsa-jack-and-pipewire-contract.md";
pub(crate) const LINUX_AUDIO_BACKEND_ACCEPTANCE_TASK: &str =
    "effigy acceptance:linux-audio-backend-boundary";
pub(crate) const LINUX_LIVE_OWNERSHIP_BOUNDARY: &str =
    "signal.runtime.linux-live-ownership-boundary";
pub(crate) const LINUX_LIVE_OWNERSHIP_CONTRACT_PATH: &str =
    "docs/contracts/052-live-linux-audio-backend-ownership-and-session-lifecycle-contract.md";
pub(crate) const LINUX_LIVE_OWNERSHIP_ACCEPTANCE_TASK: &str =
    "effigy acceptance:linux-live-ownership-boundary";
pub(crate) const JACK_COORDINATION_BOUNDARY: &str = "signal.runtime.jack-coordination-boundary";
pub(crate) const JACK_COORDINATION_CONTRACT_PATH: &str =
    "docs/contracts/053-jack-transport-graph-and-backend-native-coordination-contract.md";
pub(crate) const JACK_COORDINATION_ACCEPTANCE_TASK: &str =
    "effigy acceptance:jack-coordination-boundary";
pub(crate) const PIPEWIRE_ALSA_PARITY_BOUNDARY: &str =
    "signal.runtime.pipewire-alsa-parity-boundary";
pub(crate) const PIPEWIRE_ALSA_PARITY_CONTRACT_PATH: &str =
    "docs/contracts/054-pipewire-and-alsa-session-role-device-claim-and-stream-policy-parity-contract.md";
pub(crate) const PIPEWIRE_ALSA_PARITY_ACCEPTANCE_TASK: &str =
    "effigy acceptance:pipewire-alsa-parity-boundary";
pub(crate) const LINUX_BACKEND_CLOCK_TOPOLOGY_BOUNDARY: &str =
    "signal.runtime.linux-backend-clock-topology-boundary";
pub(crate) const LINUX_BACKEND_CLOCK_TOPOLOGY_CONTRACT_PATH: &str =
    "docs/contracts/041-linux-backend-clocking-duplex-and-endpoint-topology-parity-contract.md";
pub(crate) const LINUX_BACKEND_CLOCK_TOPOLOGY_ACCEPTANCE_TASK: &str =
    "effigy acceptance:linux-backend-clock-topology-boundary";
pub(crate) const EXTERNAL_MIDI_BOUNDARY: &str = "signal.runtime.external-midi-boundary";
pub(crate) const EXTERNAL_MIDI_CONTRACT_PATH: &str =
    "docs/contracts/065-live-external-midi-device-ownership-and-backend-parity-contract.md";
pub(crate) const EXTERNAL_MIDI_ACCEPTANCE_TASK: &str = "effigy acceptance:external-midi-boundary";
pub(crate) const GENERIC_EVENT_BOUNDARY: &str = "signal.runtime.generic-event-boundary";
pub(crate) const GENERIC_EVENT_CONTRACT_PATH: &str =
    "docs/contracts/023-generic-midi-note-expression-and-plugin-event-model-contract.md";
pub(crate) const GENERIC_EVENT_ACCEPTANCE_TASK: &str = "effigy acceptance:generic-event-boundary";
pub(crate) const CONTROLLER_EXPRESSION_BOUNDARY: &str =
    "signal.runtime.controller-expression-boundary";
pub(crate) const CONTROLLER_EXPRESSION_CONTRACT_PATH: &str =
    "docs/contracts/043-midi-2-0-mpe-and-richer-controller-expression-contract.md";
pub(crate) const CONTROLLER_EXPRESSION_ACCEPTANCE_TASK: &str =
    "effigy acceptance:controller-expression-boundary";
pub(crate) const CONTROL_SURFACE_BOUNDARY: &str = "signal.runtime.control-surface-boundary";
pub(crate) const CONTROL_SURFACE_CONTRACT_PATH: &str =
    "docs/contracts/044-control-surface-transport-mapping-and-feedback-contract.md";
pub(crate) const CONTROL_SURFACE_ACCEPTANCE_TASK: &str =
    "effigy acceptance:control-surface-boundary";
pub(crate) const ADVANCED_HARDWARE_BOUNDARY: &str = "signal.runtime.advanced-hardware-boundary";
pub(crate) const ADVANCED_HARDWARE_CONTRACT_PATH: &str =
    "docs/contracts/061-control-surface-scene-mapping-feedback-pages-and-safe-action-graph-contract.md";
pub(crate) const ADVANCED_HARDWARE_ACCEPTANCE_TASK: &str =
    "effigy acceptance:advanced-hardware-boundary";
pub(crate) const RECALL_PORTABILITY_BOUNDARY: &str = "signal.runtime.recall-portability-boundary";
pub(crate) const RECALL_PORTABILITY_CONTRACT_PATH: &str =
    "docs/contracts/024-plugin-preset-state-interchange-portable-recall-and-ara-context-contract.md";
pub(crate) const RECALL_PORTABILITY_ACCEPTANCE_TASK: &str =
    "effigy acceptance:recall-portability-boundary";
pub(crate) const DEVICE_SUPERVISION_BOUNDARY: &str = "signal.runtime.device-supervision-boundary";
pub(crate) const DEVICE_SUPERVISION_CONTRACT_PATH: &str =
    "docs/contracts/025-device-supervision-restart-state-machine-and-fault-boundary-contract.md";
pub(crate) const DEVICE_SUPERVISION_ACCEPTANCE_TASK: &str =
    "effigy acceptance:device-supervision-boundary";
pub(crate) const CLOCK_TOPOLOGY_BOUNDARY: &str = "signal.runtime.clock-topology-boundary";
pub(crate) const CLOCK_TOPOLOGY_CONTRACT_PATH: &str =
    "docs/contracts/026-clock-domain-drift-duplex-mismatch-and-endpoint-topology-contract.md";
pub(crate) const CLOCK_TOPOLOGY_ACCEPTANCE_TASK: &str = "effigy acceptance:clock-topology-boundary";
pub(crate) const EXTERNAL_IO_BOUNDARY: &str = "signal.runtime.external-io-boundary";
pub(crate) const EXTERNAL_IO_CONTRACT_PATH: &str =
    "docs/contracts/027-external-io-monitoring-tap-point-and-loopback-measurement-contract.md";
pub(crate) const EXTERNAL_IO_ACCEPTANCE_TASK: &str = "effigy acceptance:external-io-boundary";
pub(crate) const MEDIA_SERVICE_BOUNDARY: &str = "signal.runtime.media-service-boundary";
pub(crate) const MEDIA_SERVICE_CONTRACT_PATH: &str =
    "docs/contracts/028-media-indexing-waveform-analysis-and-preview-service-contract.md";
pub(crate) const MEDIA_SERVICE_ACCEPTANCE_TASK: &str = "effigy acceptance:media-service-boundary";
pub(crate) const ANALYSIS_METADATA_BOUNDARY: &str = "signal.runtime.analysis-metadata-boundary";
pub(crate) const ANALYSIS_METADATA_CONTRACT_PATH: &str =
    "docs/contracts/029-analysis-metadata-extraction-and-library-service-contract.md";
pub(crate) const ANALYSIS_METADATA_ACCEPTANCE_TASK: &str =
    "effigy acceptance:analysis-metadata-boundary";
pub(crate) const MULTICHANNEL_BOUNDARY: &str = "signal.runtime.multichannel-boundary";
pub(crate) const MULTICHANNEL_CONTRACT_PATH: &str =
    "docs/contracts/032-canonical-multichannel-layout-and-channel-role-contract.md";
pub(crate) const MULTICHANNEL_ACCEPTANCE_TASK: &str = "effigy acceptance:multichannel-boundary";
pub(crate) const MULTI_BUS_BOUNDARY: &str = "signal.runtime.multi-bus-boundary";
pub(crate) const MULTI_BUS_CONTRACT_PATH: &str =
    "docs/contracts/034-multi-bus-graph-execution-and-auxiliary-topology-contract.md";
pub(crate) const MULTI_BUS_ACCEPTANCE_TASK: &str = "effigy acceptance:multi-bus-boundary";
pub(crate) const SIDECHAIN_BOUNDARY: &str = "signal.runtime.sidechain-boundary";
pub(crate) const SIDECHAIN_CONTRACT_PATH: &str =
    "docs/contracts/033-sidechain-routing-and-secondary-input-execution-contract.md";
pub(crate) const SIDECHAIN_ACCEPTANCE_TASK: &str = "effigy acceptance:sidechain-boundary";
pub(crate) const COMPLEX_IO_BOUNDARY: &str = "signal.runtime.complex-io-boundary";
pub(crate) const COMPLEX_IO_CONTRACT_PATH: &str =
    "docs/contracts/056-complex-plugin-pin-matrix-and-dynamic-bus-negotiation-contract.md";
pub(crate) const COMPLEX_IO_ACCEPTANCE_TASK: &str = "effigy acceptance:complex-io-boundary";
pub(crate) const SPATIAL_BOUNDARY: &str = "signal.runtime.spatial-boundary";
pub(crate) const SPATIAL_CONTRACT_PATH: &str =
    "docs/contracts/059-renderer-capability-negotiation-and-immersive-export-contract.md";
pub(crate) const SPATIAL_ACCEPTANCE_TASK: &str = "effigy acceptance:spatial-boundary";
pub(crate) const STRETCH_BOUNDARY: &str = "signal.runtime.stretch-boundary";
pub(crate) const STRETCH_CONTRACT_PATH: &str =
    "docs/contracts/046-sample-domain-time-stretch-engine-contract.md";
pub(crate) const STRETCH_ACCEPTANCE_TASK: &str = "effigy acceptance:stretch-boundary";
pub(crate) const MARKER_ANALYSIS_BOUNDARY: &str = "signal.runtime.marker-analysis-boundary";
pub(crate) const MARKER_ANALYSIS_CONTRACT_PATH: &str =
    "docs/contracts/047-warp-marker-transient-anchor-and-tempo-assist-analysis-contract.md";
pub(crate) const MARKER_ANALYSIS_ACCEPTANCE_TASK: &str =
    "effigy acceptance:marker-analysis-boundary";
pub(crate) const TRANSFORM_ARTIFACT_BOUNDARY: &str = "signal.runtime.transform-artifact-boundary";
pub(crate) const TRANSFORM_ARTIFACT_CONTRACT_PATH: &str =
    "docs/contracts/064-asset-session-transform-persistence-retention-and-cache-placement-policy-contract.md";
pub(crate) const TRANSFORM_ARTIFACT_ACCEPTANCE_TASK: &str =
    "effigy acceptance:transform-artifact-boundary";
pub(crate) const PREVIEW_TRANSFORM_BOUNDARY: &str = "signal.runtime.preview-transform-boundary";
pub(crate) const PREVIEW_TRANSFORM_CONTRACT_PATH: &str =
    "docs/contracts/063-preview-browser-queue-media-audition-and-transform-scheduling-contract.md";
pub(crate) const PREVIEW_TRANSFORM_ACCEPTANCE_TASK: &str =
    "effigy acceptance:preview-transform-boundary";
pub(crate) const INTEGRATED_ACCEPTANCE_LANE: &str = "signal.runtime.integrated-acceptance-lane";
pub(crate) const INTEGRATED_ACCEPTANCE_CONTRACT_PATH: &str =
    "docs/contracts/030-fault-injection-harness-and-multi-backend-acceptance-contract.md";
pub(crate) const INTEGRATED_ACCEPTANCE_TASK: &str = "effigy acceptance:integrated-acceptance-lane";
pub(crate) const G07_ACCEPTANCE_LANE: &str = "signal.runtime.g07-integrated-acceptance-lane";
pub(crate) const G07_ACCEPTANCE_CONTRACT_PATH: &str =
    "docs/contracts/050-multichannel-linux-time-stretch-and-control-surface-acceptance-contract.md";
pub(crate) const G07_ACCEPTANCE_TASK: &str = "effigy acceptance:g07-integrated-acceptance-lane";
pub(crate) const DEVICE_WORKFLOW_ACCEPTANCE_LANE: &str =
    "signal.runtime.device-workflow-acceptance-lane";
pub(crate) const DEVICE_WORKFLOW_ACCEPTANCE_CONTRACT_PATH: &str =
    "docs/contracts/066-cross-backend-device-protocol-and-live-workflow-acceptance-contract.md";
pub(crate) const DEVICE_WORKFLOW_ACCEPTANCE_TASK: &str =
    "effigy acceptance:device-workflow-acceptance-lane";
pub(crate) const LINUX_LIVE_ACCEPTANCE_LANE: &str = "signal.runtime.linux-live-acceptance-lane";
pub(crate) const LINUX_LIVE_ACCEPTANCE_CONTRACT_PATH: &str =
    "docs/contracts/067-live-linux-backend-acceptance-and-failure-injection-contract.md";
pub(crate) const LINUX_LIVE_ACCEPTANCE_TASK: &str = "effigy acceptance:linux-live-acceptance-lane";
pub(crate) const IMMERSIVE_ACCEPTANCE_LANE: &str = "signal.runtime.immersive-acceptance-lane";
pub(crate) const IMMERSIVE_ACCEPTANCE_CONTRACT_PATH: &str =
    "docs/contracts/068-immersive-render-and-monitoring-acceptance-contract.md";
pub(crate) const IMMERSIVE_ACCEPTANCE_TASK: &str = "effigy acceptance:immersive-acceptance-lane";
pub(crate) const CONTROL_PREVIEW_WORKFLOW_ACCEPTANCE_LANE: &str =
    "signal.runtime.control-preview-workflow-acceptance-lane";
pub(crate) const CONTROL_PREVIEW_WORKFLOW_ACCEPTANCE_CONTRACT_PATH: &str =
    "docs/contracts/069-control-surface-and-preview-workflow-acceptance-contract.md";
pub(crate) const CONTROL_PREVIEW_WORKFLOW_ACCEPTANCE_TASK: &str =
    "effigy acceptance:control-preview-workflow-acceptance-lane";
pub(crate) const INTEGRATED_LIVE_WORKFLOW_ACCEPTANCE_LANE: &str =
    "signal.runtime.integrated-live-ownership-and-workflow-acceptance-lane";
pub(crate) const INTEGRATED_LIVE_WORKFLOW_ACCEPTANCE_CONTRACT_PATH: &str =
    "docs/contracts/070-integrated-live-ownership-and-workflow-acceptance-contract.md";
pub(crate) const INTEGRATED_LIVE_WORKFLOW_ACCEPTANCE_TASK: &str =
    "effigy acceptance:integrated-live-ownership-and-workflow-acceptance-lane";
pub(crate) const G06_SOAK_LANE: &str = "signal.g06.long-session-soak-lane";
pub(crate) const G06_SOAK_CONTRACT_PATH: &str =
    "docs/contracts/031-long-session-soak-promotion-gate-and-loophole-readiness-contract.md";
pub(crate) const G06_SOAK_ACCEPTANCE_TASK: &str = "effigy acceptance:g06-soak-lane";
