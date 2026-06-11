use super::*;

/// Complete observation snapshot of all runtime subsystems without the event
/// stream.
///
/// Constructed by `capture()` using a `RuntimeObservationApi` + a
/// `RuntimeEventRecorder`.  Consumed directly or wrapped in a
/// [`RuntimeSupervisorReport`] for diagnostics.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeObservationReport {
    /// Current readiness state of the runtime.
    pub readiness: RuntimeReadiness,
    /// Effective runtime configuration (sample rate, block size, flags).
    pub effective_config: EffectiveRuntimeConfig,
    /// Control-plane snapshot (running state, session counts, etc.).
    pub control_snapshot: RuntimeControlSnapshot,
    /// Scheduler snapshot (phase/lane structure, topology).
    pub scheduler_snapshot: RuntimeSchedulerSnapshot,
    /// Runtime diagnostics snapshot (CPU load, xrun count, etc.).
    pub diagnostics_snapshot: RuntimeDiagnosticsSnapshot,
    /// Metering snapshot with per-bus peak/RMS data.
    pub metering_snapshot: RuntimeMeteringSnapshot,
    /// Supervision snapshot (watchdog state, safe mode, fault gates).
    pub supervision_snapshot: RuntimeSupervisionSnapshot,
    /// Fault status snapshot derived from readiness and supervision state.
    pub fault_status: RuntimeFaultStatusSnapshot,
    /// Fault diagnostic receipt with classified interruption root cause.
    pub fault_diagnostic_receipt: RuntimeFaultDiagnosticReceipt,
    /// Interruption summary derived from fault status.
    pub interruption_summary: RuntimeInterruptionSummary,
    /// Device supervision snapshot including hardware health.
    pub device_supervision_snapshot: RuntimeDeviceSupervisionSnapshot,
    /// External I/O snapshot (hardware backend state, clocking, latency).
    pub external_io_snapshot: RuntimeExternalIoSnapshot,
    /// Timeline playback/transport snapshot.
    pub timeline_snapshot: RuntimeTimelineSnapshot,
    /// Tempo map snapshot with resolved project tempo.
    pub tempo_map_snapshot: RuntimeTempoMapSnapshot,
    /// Warp pipeline snapshot with per-clip warp states.
    pub warp_pipeline_snapshot: RuntimeWarpPipelineSnapshot,
    /// Clip processing pipeline snapshot.
    pub clip_processing_pipeline_snapshot: RuntimeClipProcessingPipelineSnapshot,
    /// Stretch engine snapshot.
    pub stretch_engine_snapshot: RuntimeStretchEngineSnapshot,
    /// Marker analysis snapshot.
    pub marker_analysis_snapshot: RuntimeMarkerAnalysisSnapshot,
    /// Transform artifact snapshot.
    pub transform_artifact_snapshot: RuntimeTransformArtifactSnapshot,
    /// Preview transform service snapshot.
    pub preview_transform_snapshot: RuntimePreviewTransformServiceSnapshot,
    /// Recording capture snapshot.
    pub recording_capture_snapshot: RuntimeRecordingCaptureSnapshot,
    /// Media pipeline snapshot (asset ingestion and conforming state).
    pub media_pipeline_snapshot: RuntimeMediaPipelineSnapshot,
    /// Media service snapshot (indexing and preview state).
    pub media_service_snapshot: RuntimeMediaServiceSnapshot,
    /// Media library service snapshot (analysis descriptor state).
    pub media_library_snapshot: RuntimeMediaLibraryServiceSnapshot,
    /// Offline render session queue snapshot.
    pub offline_render_session_snapshot: RuntimeOfflineRenderSessionSnapshot,
    /// Automation snapshot.
    pub automation_snapshot: RuntimeAutomationSnapshot,
    /// Engine block snapshot with per-block timing and scheduler state.
    pub engine_block_snapshot: RuntimeEngineBlockSnapshot,
    /// Transport concurrency snapshot (session and lease counts).
    pub transport_concurrency_snapshot: RuntimeTransportConcurrencySnapshot,
    /// Plugin discovery snapshot.
    pub plugin_discovery_snapshot: RuntimePluginDiscoverySnapshot,
    /// Plugin lifecycle snapshot.
    pub plugin_lifecycle_snapshot: RuntimePluginLifecycleSnapshot,
    /// LV2 extension snapshot.
    pub lv2_extension_snapshot: RuntimeLv2ExtensionSnapshot,
    /// Plugin pin matrix snapshot.
    pub plugin_pin_matrix_snapshot: RuntimePluginPinMatrixSnapshot,
    /// Plugin chain snapshot.
    pub plugin_chain_snapshot: RuntimePluginChainSnapshot,
    /// Scheduler export summary derived from the engine block snapshot.
    pub scheduler_summary: RuntimeSchedulerExportSummary,
    /// Block execution summary derived from the engine block snapshot.
    pub block_summary: RuntimeBlockExecutionSummary,
    /// Degradation summary capturing active fault conditions.
    pub degradation_summary: RuntimeDegradationSummary,
    /// Execution topology summary (track lanes, bus groups, etc.).
    pub execution_topology_summary: RuntimeExecutionTopologySummary,
    /// Transport fault summary derived from observed transport fault events.
    pub transport_fault_summary: TransportFaultSummary,
    /// Transport session summary derived from observed events.
    pub transport_session_summary: TransportSessionSummary,
    /// Most recent deferred background service receipt, if any.
    pub last_deferred_service_receipt: Option<RuntimeDeferredServiceReceipt>,
    /// Categorised event diagnostics from the attached event recorder.
    pub observation: RuntimeObservationDiagnostics,
}

impl RuntimeObservationReport {
    /// Captures a full observation report by polling all runtime subsystems and the event recorder.
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

    /// Replaces the device supervision snapshot using hardware I/O context from the host.
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

    /// Replaces the external I/O snapshot using the host I/O summary.
    pub fn with_host_external_io(mut self, host_io: &RuntimeHostIoSummary) -> Self {
        self.external_io_snapshot = host_io.build_external_io_snapshot();
        self
    }
}
