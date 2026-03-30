use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeObservationReport {
    pub readiness: RuntimeReadiness,
    pub effective_config: EffectiveRuntimeConfig,
    pub control_snapshot: RuntimeControlSnapshot,
    pub scheduler_snapshot: RuntimeSchedulerSnapshot,
    pub diagnostics_snapshot: RuntimeDiagnosticsSnapshot,
    pub metering_snapshot: RuntimeMeteringSnapshot,
    pub supervision_snapshot: RuntimeSupervisionSnapshot,
    pub fault_status: RuntimeFaultStatusSnapshot,
    pub fault_diagnostic_receipt: RuntimeFaultDiagnosticReceipt,
    pub interruption_summary: RuntimeInterruptionSummary,
    pub device_supervision_snapshot: RuntimeDeviceSupervisionSnapshot,
    pub external_io_snapshot: RuntimeExternalIoSnapshot,
    pub linux_backend_session_snapshot: RuntimeLinuxBackendSessionSnapshot,
    pub pipewire_alsa_parity_snapshot: RuntimePipeWireAlsaParitySnapshot,
    pub jack_coordination_snapshot: RuntimeJackCoordinationSnapshot,
    pub external_midi_snapshot: RuntimeExternalMidiEndpointGraphSnapshot,
    pub control_surface_snapshot: RuntimeControlSurfaceSnapshot,
    pub advanced_hardware_snapshot: RuntimeAdvancedHardwareSnapshot,
    pub timeline_snapshot: RuntimeTimelineSnapshot,
    pub tempo_map_snapshot: RuntimeTempoMapSnapshot,
    pub warp_pipeline_snapshot: RuntimeWarpPipelineSnapshot,
    pub clip_processing_pipeline_snapshot: RuntimeClipProcessingPipelineSnapshot,
    pub stretch_engine_snapshot: RuntimeStretchEngineSnapshot,
    pub marker_analysis_snapshot: RuntimeMarkerAnalysisSnapshot,
    pub transform_artifact_snapshot: RuntimeTransformArtifactSnapshot,
    pub preview_transform_snapshot: RuntimePreviewTransformServiceSnapshot,
    pub recording_capture_snapshot: RuntimeRecordingCaptureSnapshot,
    pub media_pipeline_snapshot: RuntimeMediaPipelineSnapshot,
    pub media_service_snapshot: RuntimeMediaServiceSnapshot,
    pub media_library_snapshot: RuntimeMediaLibraryServiceSnapshot,
    pub offline_render_session_snapshot: RuntimeOfflineRenderSessionSnapshot,
    pub automation_snapshot: RuntimeAutomationSnapshot,
    pub plugin_event_snapshot: RuntimePluginEventSnapshot,
    pub engine_block_snapshot: RuntimeEngineBlockSnapshot,
    pub transport_concurrency_snapshot: RuntimeTransportConcurrencySnapshot,
    pub plugin_discovery_snapshot: RuntimePluginDiscoverySnapshot,
    pub plugin_lifecycle_snapshot: RuntimePluginLifecycleSnapshot,
    pub lv2_extension_snapshot: RuntimeLv2ExtensionSnapshot,
    pub plugin_pin_matrix_snapshot: RuntimePluginPinMatrixSnapshot,
    pub plugin_chain_snapshot: RuntimePluginChainSnapshot,
    pub scheduler_summary: RuntimeSchedulerExportSummary,
    pub block_summary: RuntimeBlockExecutionSummary,
    pub degradation_summary: RuntimeDegradationSummary,
    pub execution_topology_summary: RuntimeExecutionTopologySummary,
    pub transport_fault_summary: TransportFaultSummary,
    pub transport_session_summary: TransportSessionSummary,
    pub last_deferred_service_receipt: Option<RuntimeDeferredServiceReceipt>,
    pub observation: RuntimeObservationDiagnostics,
}

impl RuntimeObservationReport {
    pub fn capture(runtime: &impl RuntimeObservationApi, recorder: &RuntimeEventRecorder) -> Self {
        let observation = recorder.diagnostics();
        let readiness = runtime.get_readiness();
        let effective_config = runtime.get_effective_config();
        let control_snapshot = runtime.get_control_snapshot();
        let scheduler_snapshot = runtime.get_scheduler_snapshot();
        let diagnostics_snapshot = runtime.get_diagnostics_snapshot();
        let metering_snapshot = runtime.get_metering_snapshot();
        let supervision_snapshot = runtime.get_supervision_snapshot();
        let timeline_snapshot = runtime.get_timeline_snapshot();
        let tempo_map_snapshot = runtime.get_tempo_map_snapshot();
        let warp_pipeline_snapshot = runtime.get_warp_pipeline_snapshot();
        let clip_processing_pipeline_snapshot = runtime.get_clip_processing_pipeline_snapshot();
        let stretch_engine_snapshot = runtime.get_stretch_engine_snapshot();
        let marker_analysis_snapshot = runtime.get_marker_analysis_snapshot();
        let transform_artifact_snapshot = runtime.get_transform_artifact_snapshot();
        let preview_transform_snapshot = runtime.get_preview_transform_snapshot();
        let recording_capture_snapshot = runtime.get_recording_capture_snapshot();
        let media_pipeline_snapshot = runtime.get_media_pipeline_snapshot();
        let media_service_snapshot = runtime.get_media_service_snapshot();
        let media_library_snapshot = runtime.get_media_library_service_snapshot();
        let offline_render_session_snapshot = runtime.get_offline_render_session_snapshot();
        let automation_snapshot = runtime.get_automation_snapshot();
        let plugin_event_snapshot = runtime.get_plugin_event_snapshot();
        let engine_block_snapshot = runtime.get_engine_block_snapshot();
        let execution_topology_summary = runtime.get_execution_topology_summary();
        let transport_concurrency_snapshot = runtime.get_transport_concurrency_snapshot();
        let plugin_discovery_snapshot = runtime.get_plugin_discovery_snapshot();
        let plugin_lifecycle_snapshot = runtime.get_plugin_lifecycle_snapshot();
        let plugin_chain_snapshot = runtime.get_plugin_chain_snapshot();
        let lv2_extension_snapshot = RuntimeLv2ExtensionSnapshot::capture(
            &plugin_discovery_snapshot,
            &plugin_lifecycle_snapshot,
        );
        let plugin_pin_matrix_snapshot = RuntimePluginPinMatrixSnapshot::capture(
            &plugin_discovery_snapshot,
            &plugin_lifecycle_snapshot,
            &plugin_chain_snapshot,
        );
        let last_deferred_service_receipt = runtime.get_last_deferred_service_receipt();
        let scheduler_summary =
            RuntimeSchedulerExportSummary::from_snapshot(&engine_block_snapshot);
        let block_summary = RuntimeBlockExecutionSummary::from_snapshot(&engine_block_snapshot);
        let fault_status = RuntimeFaultStatusSnapshot::capture(RuntimeFaultStatusCaptureInput {
            readiness: readiness.clone(),
            control_snapshot: &control_snapshot,
            diagnostics_snapshot: &diagnostics_snapshot,
            supervision_snapshot: &supervision_snapshot,
            engine_block_snapshot: &engine_block_snapshot,
            transport_concurrency_snapshot: &transport_concurrency_snapshot,
            plugin_lifecycle_snapshot: &plugin_lifecycle_snapshot,
            device_loss_active: false,
            device_loss_count: 0,
        });
        let degradation_summary = RuntimeDegradationSummary::capture(
            &readiness,
            diagnostics_snapshot,
            &supervision_snapshot,
            &engine_block_snapshot,
            &transport_concurrency_snapshot,
            &observation,
        );
        let interruption_summary = RuntimeInterruptionSummary::capture(
            &fault_status,
            last_deferred_service_receipt.as_ref(),
        );
        let fault_diagnostic_receipt = RuntimeFaultDiagnosticReceipt::capture(
            &fault_status,
            &interruption_summary,
            &degradation_summary,
            &engine_block_snapshot,
            last_deferred_service_receipt.as_ref(),
            None,
        );
        let device_supervision_snapshot = RuntimeDeviceSupervisionSnapshot::capture(
            &effective_config,
            &supervision_snapshot,
            &fault_status,
            &interruption_summary,
            None,
        );
        let external_io_snapshot = RuntimeHostIoSummary::unavailable_external_io_snapshot(
            &effective_config,
            &device_supervision_snapshot,
        );
        let linux_backend_session_snapshot = RuntimeLinuxBackendSessionSnapshot::unavailable();
        let pipewire_alsa_parity_snapshot = RuntimePipeWireAlsaParitySnapshot::unavailable();
        let jack_coordination_snapshot = RuntimeJackCoordinationSnapshot::unavailable();
        let external_midi_snapshot = RuntimeExternalMidiEndpointGraphSnapshot::unavailable();
        let control_surface_snapshot =
            RuntimeControlSurfaceSnapshot::from_external_midi_snapshot(&external_midi_snapshot);
        let advanced_hardware_snapshot =
            RuntimeAdvancedHardwareSnapshot::from_control_surface_snapshot(
                &control_surface_snapshot,
            );
        Self {
            readiness: readiness.clone(),
            effective_config,
            control_snapshot,
            scheduler_snapshot,
            diagnostics_snapshot,
            metering_snapshot,
            supervision_snapshot: supervision_snapshot.clone(),
            fault_status,
            fault_diagnostic_receipt,
            interruption_summary,
            device_supervision_snapshot,
            external_io_snapshot,
            linux_backend_session_snapshot,
            pipewire_alsa_parity_snapshot,
            jack_coordination_snapshot,
            external_midi_snapshot,
            control_surface_snapshot,
            advanced_hardware_snapshot,
            timeline_snapshot,
            tempo_map_snapshot,
            warp_pipeline_snapshot,
            clip_processing_pipeline_snapshot,
            stretch_engine_snapshot,
            marker_analysis_snapshot,
            transform_artifact_snapshot,
            preview_transform_snapshot,
            recording_capture_snapshot,
            media_pipeline_snapshot,
            media_service_snapshot,
            media_library_snapshot,
            offline_render_session_snapshot,
            automation_snapshot,
            plugin_event_snapshot,
            engine_block_snapshot,
            transport_concurrency_snapshot,
            plugin_discovery_snapshot,
            plugin_lifecycle_snapshot,
            lv2_extension_snapshot,
            plugin_pin_matrix_snapshot,
            plugin_chain_snapshot,
            scheduler_summary,
            block_summary,
            degradation_summary,
            execution_topology_summary,
            transport_fault_summary: TransportFaultSummary::from_records(
                &observation.transport_fault_events,
            ),
            transport_session_summary: TransportSessionSummary::from_diagnostics(&observation),
            last_deferred_service_receipt,
            observation,
        }
    }

    pub fn with_host_device_supervision(mut self, host_io: &RuntimeHostIoSummary) -> Self {
        self.device_supervision_snapshot = RuntimeDeviceSupervisionSnapshot::capture(
            &self.effective_config,
            &self.supervision_snapshot,
            &self.fault_status,
            &self.interruption_summary,
            Some(host_io),
        );
        self
    }

    pub fn with_host_external_io(mut self, host_io: &RuntimeHostIoSummary) -> Self {
        self.external_io_snapshot = host_io.build_external_io_snapshot();
        self
    }

    pub fn with_linux_backend_session_snapshot(mut self, host_io: &RuntimeHostIoSummary) -> Self {
        self.linux_backend_session_snapshot =
            RuntimeLinuxBackendSessionSnapshot::from_host_io(host_io);
        self
    }

    pub fn with_pipewire_alsa_parity_snapshot(mut self, host_io: &RuntimeHostIoSummary) -> Self {
        self.pipewire_alsa_parity_snapshot =
            RuntimePipeWireAlsaParitySnapshot::from_host_io_and_linux_session(
                host_io,
                &self.linux_backend_session_snapshot,
            );
        self
    }

    pub fn with_jack_coordination_snapshot(mut self, host_io: &RuntimeHostIoSummary) -> Self {
        self.jack_coordination_snapshot =
            RuntimeJackCoordinationSnapshot::from_host_io_and_transport_session(
                host_io,
                &self.transport_session_summary,
            );
        self
    }

    pub fn with_external_midi_snapshot(
        mut self,
        external_midi_snapshot: RuntimeExternalMidiEndpointGraphSnapshot,
    ) -> Self {
        let external_midi_snapshot = external_midi_snapshot.with_live_ownership_summary(
            &self.linux_backend_session_snapshot,
            &self.interruption_summary,
        );
        self.control_surface_snapshot =
            RuntimeControlSurfaceSnapshot::from_external_midi_snapshot(&external_midi_snapshot);
        self.advanced_hardware_snapshot =
            RuntimeAdvancedHardwareSnapshot::from_control_surface_snapshot(
                &self.control_surface_snapshot,
            );
        self.external_midi_snapshot = external_midi_snapshot;
        self
    }

    pub fn render_compact(&self) -> String {
        render_runtime_observation_report_compact(self)
    }
}

impl RuntimeObservationReport {
    pub fn render_json(&self) -> String {
        RuntimeSupervisorReport {
            observation: self.clone(),
            events: Vec::new(),
        }
        .render_json()
    }
}
