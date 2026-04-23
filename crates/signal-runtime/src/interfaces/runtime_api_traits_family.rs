use super::*;

/// Specification for a plugin sandbox to be provisioned by the runtime.
///
/// Pass to `ensure_plugin_sandbox()`.  The `sandbox_id` must be stable across
/// reconfigures; `plugin_type_id` is `None` when the format has not yet been
/// resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginSandboxSpec {
    /// Stable identifier for the sandbox, unique within the runtime.
    pub sandbox_id: String,
    /// Plugin format for this sandbox.
    pub plugin_format: PluginFormat,
    /// Plugin type identifier, if the format has been resolved.
    pub plugin_type_id: Option<String>,
}

/// Opaque handle returned by `ensure_plugin_sandbox()`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SandboxHandle(pub u64);

/// Requests a specific backend policy tier for the runtime.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BackendPolicyOverride {
    /// The backend policy tier to activate.
    pub tier: BackendPolicyTier,
}

/// Lifecycle control: handshake, configure, start, stop, restart, safe mode.
pub trait RuntimeLifecycleApi {
    /// Performs the initial handshake with the runtime, establishing the client version.
    fn handshake(&mut self, request: HandshakeRequest) -> Result<HandshakeResponse, RuntimeError>;
    /// Applies a new runtime configuration.
    fn configure(&mut self, request: RuntimeConfigRequest) -> Result<(), RuntimeError>;
    /// Starts audio processing.
    fn start(&mut self) -> Result<(), RuntimeError>;
    /// Stops audio processing with the given reason.
    fn stop(&mut self, reason: StopReason) -> Result<(), RuntimeError>;
    /// Restarts the runtime with the given restart request.
    fn restart(&mut self, request: RestartRequest) -> Result<(), RuntimeError>;
    /// Enables or disables safe mode.
    fn set_safe_mode(&mut self, request: SafeModeRequest) -> Result<(), RuntimeError>;
}

/// Graph, schedule, transport, parameter, automation, and hardware projection.
pub trait RuntimeProjectionApi {
    /// Sets the realtime pressure level signalled to the prework service.
    fn set_prework_service_pressure(
        &mut self,
        pressure: RuntimePreworkServicePressure,
    ) -> Result<(), RuntimeError>;
    /// Sets the prework forecast operating mode.
    fn set_prework_forecast_mode(
        &mut self,
        mode: RuntimePreworkForecastMode,
    ) -> Result<(), RuntimeError>;
    /// Sets the prework forecast profile selection.
    fn set_prework_forecast_profile(
        &mut self,
        selection: RuntimePreworkForecastProfileSelection,
    ) -> Result<(), RuntimeError>;
    /// Sets a raw prework forecast policy override.
    fn set_prework_forecast_policy(
        &mut self,
        policy: RuntimePreworkForecastPolicy,
    ) -> Result<(), RuntimeError>;
    /// Services the prework lane for the given processing epoch and cycle count.
    fn service_prework_lane(
        &mut self,
        processing_epoch: u64,
        cycles: usize,
    ) -> Result<usize, RuntimeError>;
    /// Applies plugin-backed node bindings to the active graph.
    fn apply_plugin_backed_node_bindings(
        &mut self,
        projection: PluginBackedNodeBindingProjection,
    ) -> Result<ProjectionReceipt, RuntimeError>;
    /// Applies a plugin placement policy to the runtime.
    fn apply_plugin_placement_policy(
        &mut self,
        policy: RuntimePluginPlacementPolicy,
    ) -> Result<(), RuntimeError>;
    /// Applies a graph contract projection to the runtime.
    fn apply_graph_contract_projection(
        &mut self,
        projection: GraphContractProjection,
    ) -> Result<ProjectionReceipt, RuntimeError>;
    /// Applies a full graph projection to the runtime.
    fn apply_graph_projection(
        &mut self,
        projection: GraphProjection,
    ) -> Result<ProjectionReceipt, RuntimeError>;
    /// Applies a schedule projection to the runtime.
    fn apply_schedule_projection(
        &mut self,
        projection: ScheduleProjection,
    ) -> Result<ProjectionReceipt, RuntimeError>;
    /// Applies an automation projection to the runtime.
    fn apply_automation_projection(
        &mut self,
        projection: RuntimeAutomationProjection,
    ) -> Result<ProjectionReceipt, RuntimeError>;
    /// Applies a tempo map projection to the runtime.
    fn apply_tempo_map_projection(
        &mut self,
        projection: RuntimeTempoMapProjection,
    ) -> Result<ProjectionReceipt, RuntimeError>;
    /// Applies a transport state projection to the runtime.
    fn apply_transport_projection(
        &mut self,
        projection: TransportProjection,
    ) -> Result<(), RuntimeError>;
    /// Applies a parameter batch to the runtime.
    fn apply_parameter_batch(&mut self, batch: ParameterBatch) -> Result<(), RuntimeError>;
    /// Applies a hardware configuration request to the runtime.
    fn apply_hardware_config(&mut self, request: HardwareConfigRequest)
        -> Result<(), RuntimeError>;
}

/// Read-only observation surface for all runtime subsystems.
///
/// All methods take `&self` — callers that only observe never need a mutable
/// borrow.  Use `subscribe()` to register a [`RuntimeEventSink`] for push
/// notifications.
pub trait RuntimeObservationApi {
    /// Registers an event sink to receive push notifications from the runtime.
    fn subscribe(&mut self, sink: Box<dyn RuntimeEventSink>) -> SubscriptionHandle;
    /// Returns the current overall readiness state of the runtime.
    fn get_readiness(&self) -> RuntimeReadiness;
    /// Returns a multi-lane acceptance receipt for the current runtime state.
    fn get_acceptance_receipt(&self) -> RuntimeAcceptanceReceipt;
    /// Returns the currently active runtime configuration values.
    fn get_effective_config(&self) -> EffectiveRuntimeConfig;
    /// Returns the current runtime control-plane snapshot.
    fn get_control_snapshot(&self) -> RuntimeControlSnapshot;
    /// Returns the current graph scheduler snapshot.
    fn get_scheduler_snapshot(&self) -> RuntimeSchedulerSnapshot;
    /// Returns the current scheduler topology summary.
    fn get_scheduler_topology_summary(&self) -> RuntimeSchedulerTopologySummary;
    /// Returns the current scalar diagnostics snapshot.
    fn get_diagnostics_snapshot(&self) -> RuntimeDiagnosticsSnapshot;
    /// Returns the current metering snapshot.
    fn get_metering_snapshot(&self) -> RuntimeMeteringSnapshot;
    /// Returns the current supervision snapshot.
    fn get_supervision_snapshot(&self) -> RuntimeSupervisionSnapshot;
    /// Returns the current timeline snapshot.
    fn get_timeline_snapshot(&self) -> RuntimeTimelineSnapshot;
    /// Returns the current transport observation snapshot.
    fn get_transport_observation_snapshot(&self) -> RuntimeTransportObservationSnapshot;
    /// Returns the current recording capture snapshot.
    fn get_recording_capture_snapshot(&self) -> RuntimeRecordingCaptureSnapshot;
    /// Returns the current offline render session snapshot.
    fn get_offline_render_session_snapshot(&self) -> RuntimeOfflineRenderSessionSnapshot;
    /// Returns the current media pipeline snapshot.
    fn get_media_pipeline_snapshot(&self) -> RuntimeMediaPipelineSnapshot;
    /// Returns the current media service snapshot.
    fn get_media_service_snapshot(&self) -> RuntimeMediaServiceSnapshot;
    /// Returns the current media library service snapshot.
    fn get_media_library_service_snapshot(&self) -> RuntimeMediaLibraryServiceSnapshot;
    /// Returns the current tempo map snapshot.
    fn get_tempo_map_snapshot(&self) -> RuntimeTempoMapSnapshot;
    /// Returns the current warp pipeline snapshot.
    fn get_warp_pipeline_snapshot(&self) -> RuntimeWarpPipelineSnapshot;
    /// Returns the current clip processing pipeline snapshot.
    fn get_clip_processing_pipeline_snapshot(&self) -> RuntimeClipProcessingPipelineSnapshot;
    /// Returns the current stretch engine snapshot.
    fn get_stretch_engine_snapshot(&self) -> RuntimeStretchEngineSnapshot;
    /// Returns the current marker analysis snapshot.
    fn get_marker_analysis_snapshot(&self) -> RuntimeMarkerAnalysisSnapshot;
    /// Returns the current transform artifact snapshot.
    fn get_transform_artifact_snapshot(&self) -> RuntimeTransformArtifactSnapshot;
    /// Returns the current preview transform service snapshot.
    fn get_preview_transform_snapshot(&self) -> RuntimePreviewTransformServiceSnapshot;
    /// Returns the current automation snapshot.
    fn get_automation_snapshot(&self) -> RuntimeAutomationSnapshot;
    /// Returns the current plugin event snapshot.
    fn get_plugin_event_snapshot(&self) -> RuntimePluginEventSnapshot;
    /// Returns the current engine block snapshot.
    fn get_engine_block_snapshot(&self) -> RuntimeEngineBlockSnapshot;
    /// Returns the current execution topology summary.
    fn get_execution_topology_summary(&self) -> RuntimeExecutionTopologySummary;
    /// Returns the current transport concurrency snapshot.
    fn get_transport_concurrency_snapshot(&self) -> RuntimeTransportConcurrencySnapshot;
    /// Returns the current plugin discovery snapshot.
    fn get_plugin_discovery_snapshot(&self) -> RuntimePluginDiscoverySnapshot;
    /// Returns the current plugin lifecycle snapshot.
    fn get_plugin_lifecycle_snapshot(&self) -> RuntimePluginLifecycleSnapshot;
    /// Returns the current plugin chain snapshot.
    fn get_plugin_chain_snapshot(&self) -> RuntimePluginChainSnapshot;
    /// Returns the current plugin recall handoff snapshot.
    fn get_plugin_recall_handoff_snapshot(&self) -> RuntimePluginRecallHandoffSnapshot;
    /// Returns the most recent deferred service receipt, if any.
    fn get_last_deferred_service_receipt(&self) -> Option<RuntimeDeferredServiceReceipt>;
}

/// Supervisor-level operations: plugin sandbox management, recording capture,
/// media asset reconciliation, and offline rendering.
pub trait RuntimeSupervisorApi {
    /// Starts an asynchronous plugin scan with the given request parameters.
    fn start_plugin_scan(&mut self, request: PluginScanRequest)
        -> Result<ScanHandle, RuntimeError>;
    /// Ensures a plugin sandbox is provisioned for the given specification.
    fn ensure_plugin_sandbox(
        &mut self,
        request: PluginSandboxSpec,
    ) -> Result<SandboxHandle, RuntimeError>;
    /// Starts a recording capture session.
    fn start_recording_capture(
        &mut self,
        request: RuntimeRecordingCaptureStartRequest,
    ) -> Result<(), RuntimeError>;
    /// Finishes the active recording capture session and commits the result.
    fn finish_recording_capture(
        &mut self,
    ) -> Result<RuntimeRecordingCaptureCommitReceipt, RuntimeError>;
    /// Cancels the active recording capture session.
    fn cancel_recording_capture(&mut self) -> Result<(), RuntimeError>;
    /// Reconciles the set of registered media assets with the runtime.
    fn reconcile_media_assets(
        &mut self,
        assets: Vec<RuntimeMediaAssetRegistration>,
    ) -> Result<(), RuntimeError>;
    /// Starts a media preview for the given asset.
    fn start_media_preview(&mut self, asset_id: &str) -> Result<(), RuntimeError>;
    /// Stops the active media preview.
    fn stop_media_preview(&mut self) -> Result<(), RuntimeError>;
    /// Reconciles the set of registered warp clips with the runtime.
    fn reconcile_warp_clips(
        &mut self,
        clips: Vec<RuntimeWarpClipRegistration>,
    ) -> Result<(), RuntimeError>;
    /// Reconciles the set of registered clip processing clips with the runtime.
    fn reconcile_clip_processing_clips(
        &mut self,
        clips: Vec<RuntimeClipProcessingRegistration>,
    ) -> Result<(), RuntimeError>;
    /// Performs a synchronous offline render and returns the result.
    fn render_offline(
        &self,
        request: RuntimeOfflineRenderRequest,
    ) -> Result<RuntimeOfflineRenderResult, RuntimeError>;
    /// Performs a synchronous offline render with checkpoint support.
    fn render_offline_with_checkpoints(
        &self,
        request: RuntimeOfflineRenderRequest,
    ) -> Result<RuntimeOfflineRenderExecutionReceipt, RuntimeError>;
    /// Begins an asynchronous offline render execution.
    fn begin_offline_render_execution(
        &mut self,
        request: RuntimeOfflineRenderRequest,
    ) -> Result<RuntimeOfflineRenderExecutionProgressReceipt, RuntimeError>;
    /// Pauses an in-progress offline render execution.
    fn pause_offline_render_execution(
        &mut self,
        request_id: &str,
    ) -> Result<RuntimeOfflineRenderExecutionProgressReceipt, RuntimeError>;
    /// Resumes a paused offline render execution.
    fn resume_offline_render_execution(
        &mut self,
        request_id: &str,
    ) -> Result<RuntimeOfflineRenderExecutionProgressReceipt, RuntimeError>;
    /// Interrupts an in-progress offline render execution with the given reason.
    fn interrupt_offline_render_execution(
        &mut self,
        request_id: &str,
        reason: String,
    ) -> Result<RuntimeOfflineRenderExecutionProgressReceipt, RuntimeError>;
    /// Advances an offline render execution by one step.
    fn advance_offline_render_execution(
        &mut self,
        request_id: &str,
    ) -> Result<RuntimeOfflineRenderExecutionProgressReceipt, RuntimeError>;
    /// Cancels an in-progress offline render execution.
    fn cancel_offline_render_execution(
        &mut self,
        request_id: &str,
    ) -> Result<RuntimeOfflineRenderExecutionCancellationReceipt, RuntimeError>;
    /// Renders a queue of offline render requests in sequence.
    fn render_offline_queue(
        &self,
        requests: Vec<RuntimeOfflineRenderRequest>,
    ) -> Result<RuntimeOfflineRenderQueueResult, RuntimeError>;
    /// Purges offline render artifacts matching the given request.
    fn purge_offline_render_artifacts(
        &self,
        request: RuntimeOfflineRenderPurgeRequest,
    ) -> Result<RuntimeOfflineRenderPurgeReceipt, RuntimeError>;
    /// Tears down the plugin sandbox with the given ID.
    fn teardown_plugin_sandbox(&mut self, sandbox_id: &str) -> Result<(), RuntimeError>;
    /// Restarts the plugin sandbox with the given ID.
    fn restart_plugin_sandbox(&mut self, sandbox_id: &str) -> Result<(), RuntimeError>;
    /// Applies a backend policy override to the runtime.
    fn set_backend_policy(&mut self, request: BackendPolicyOverride) -> Result<(), RuntimeError>;
}
