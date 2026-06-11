use super::*;

/// Observation snapshot of the runtime control surface without the event
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
    /// Control-plane snapshot (running state, lifecycle counts, etc.).
    pub control_snapshot: RuntimeControlSnapshot,
    /// Runtime diagnostics snapshot (CPU load, xrun count, etc.).
    pub diagnostics_snapshot: RuntimeDiagnosticsSnapshot,
    /// Supervision snapshot (watchdog state, safe mode, fault gates).
    pub supervision_snapshot: RuntimeSupervisionSnapshot,
    /// Fault status snapshot derived from readiness and supervision state.
    pub fault_status: RuntimeFaultStatusSnapshot,
    /// Interruption summary derived from fault status.
    pub interruption_summary: RuntimeInterruptionSummary,
    /// Device supervision snapshot including hardware health.
    pub device_supervision_snapshot: RuntimeDeviceSupervisionSnapshot,
    /// External I/O snapshot (hardware backend state, clocking, latency).
    pub external_io_snapshot: RuntimeExternalIoSnapshot,
    /// Tempo map snapshot with resolved project tempo.
    pub tempo_map_snapshot: RuntimeTempoMapSnapshot,
    /// Warp pipeline snapshot with per-clip warp states.
    pub warp_pipeline_snapshot: RuntimeWarpPipelineSnapshot,
    /// Clip processing pipeline snapshot.
    pub clip_processing_pipeline_snapshot: RuntimeClipProcessingPipelineSnapshot,
    /// Recording capture snapshot.
    pub recording_capture_snapshot: RuntimeRecordingCaptureSnapshot,
    /// Media pipeline snapshot (asset ingestion and conforming state).
    pub media_pipeline_snapshot: RuntimeMediaPipelineSnapshot,
    /// Media service snapshot (indexing and preview state).
    pub media_service_snapshot: RuntimeMediaServiceSnapshot,
    /// Media library service snapshot (analysis descriptor state).
    pub media_library_snapshot: RuntimeMediaLibraryServiceSnapshot,
    /// Plugin discovery snapshot.
    pub plugin_discovery_snapshot: RuntimePluginDiscoverySnapshot,
    /// Plugin lifecycle snapshot.
    pub plugin_lifecycle_snapshot: RuntimePluginLifecycleSnapshot,
    /// Plugin chain snapshot.
    pub plugin_chain_snapshot: RuntimePluginChainSnapshot,
    /// Execution topology summary (track lanes, bus groups, etc.).
    pub execution_topology_summary: RuntimeExecutionTopologySummary,
    /// Declared latency of the applied graph plan, in samples.
    pub graph_latency_samples: u32,
    /// Transport fault summary derived from observed transport fault events.
    pub transport_fault_summary: TransportFaultSummary,
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
        let diagnostics_snapshot = runtime.get_diagnostics_snapshot();
        let supervision_snapshot = runtime.get_supervision_snapshot();
        let tempo_map_snapshot = runtime.get_tempo_map_snapshot();
        let warp_pipeline_snapshot = runtime.get_warp_pipeline_snapshot();
        let clip_processing_pipeline_snapshot = runtime.get_clip_processing_pipeline_snapshot();
        let recording_capture_snapshot = runtime.get_recording_capture_snapshot();
        let media_pipeline_snapshot = runtime.get_media_pipeline_snapshot();
        let media_service_snapshot = runtime.get_media_service_snapshot();
        let media_library_snapshot = runtime.get_media_library_service_snapshot();
        let execution_topology_summary = runtime.get_execution_topology_summary();
        let graph_latency_samples = runtime.get_graph_latency_samples();
        let plugin_discovery_snapshot = runtime.get_plugin_discovery_snapshot();
        let plugin_lifecycle_snapshot = runtime.get_plugin_lifecycle_snapshot();
        let plugin_chain_snapshot = runtime.get_plugin_chain_snapshot();
        let fault_status = RuntimeFaultStatusSnapshot::capture(RuntimeFaultStatusCaptureInput {
            readiness: readiness.clone(),
            control_snapshot: &control_snapshot,
            diagnostics_snapshot: &diagnostics_snapshot,
            supervision_snapshot: &supervision_snapshot,
            plugin_lifecycle_snapshot: &plugin_lifecycle_snapshot,
            device_loss_active: false,
            device_loss_count: 0,
            missing_plugin_binding_count: plugin_chain_snapshot.missing_binding_stage_count,
        });
        let interruption_summary = RuntimeInterruptionSummary::capture(&fault_status);
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
            readiness,
            effective_config,
            control_snapshot,
            diagnostics_snapshot,
            supervision_snapshot,
            fault_status,
            interruption_summary,
            device_supervision_snapshot,
            external_io_snapshot,
            tempo_map_snapshot,
            warp_pipeline_snapshot,
            clip_processing_pipeline_snapshot,
            recording_capture_snapshot,
            media_pipeline_snapshot,
            media_service_snapshot,
            media_library_snapshot,
            plugin_discovery_snapshot,
            plugin_lifecycle_snapshot,
            plugin_chain_snapshot,
            execution_topology_summary,
            graph_latency_samples,
            transport_fault_summary: TransportFaultSummary::from_records(
                &observation.transport_fault_events,
            ),
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
