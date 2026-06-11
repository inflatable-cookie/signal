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

/// Graph plan and hardware projection.
pub trait RuntimeProjectionApi {
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
    /// Applies a hardware configuration request to the runtime.
    fn apply_hardware_config(&mut self, request: HardwareConfigRequest)
        -> Result<(), RuntimeError>;
}

/// Read-only observation surface for all runtime subsystems.
///
/// All methods take `&self` — callers that only observe never need a mutable
/// borrow.  Use `subscribe()` to register a `RuntimeEventSink` for push
/// notifications.
pub trait RuntimeObservationApi {
    /// Registers an event sink to receive push notifications from the runtime.
    fn subscribe(&mut self, sink: Box<dyn RuntimeEventSink>) -> SubscriptionHandle;
    /// Returns the current overall readiness state of the runtime.
    fn get_readiness(&self) -> RuntimeReadiness;
    /// Returns the currently active runtime configuration values.
    fn get_effective_config(&self) -> EffectiveRuntimeConfig;
    /// Returns the current runtime control-plane snapshot.
    fn get_control_snapshot(&self) -> RuntimeControlSnapshot;
    /// Returns the current scalar diagnostics snapshot.
    fn get_diagnostics_snapshot(&self) -> RuntimeDiagnosticsSnapshot;
    /// Returns the current supervision snapshot.
    fn get_supervision_snapshot(&self) -> RuntimeSupervisionSnapshot;
    /// Returns the current recording capture snapshot.
    fn get_recording_capture_snapshot(&self) -> RuntimeRecordingCaptureSnapshot;
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
    /// Returns the current execution topology summary.
    fn get_execution_topology_summary(&self) -> RuntimeExecutionTopologySummary;
    /// Returns the declared latency of the applied graph plan, in samples.
    fn get_graph_latency_samples(&self) -> u32;
    /// Returns the current plugin discovery snapshot.
    fn get_plugin_discovery_snapshot(&self) -> RuntimePluginDiscoverySnapshot;
    /// Returns the current plugin lifecycle snapshot.
    fn get_plugin_lifecycle_snapshot(&self) -> RuntimePluginLifecycleSnapshot;
    /// Returns the current plugin chain snapshot.
    fn get_plugin_chain_snapshot(&self) -> RuntimePluginChainSnapshot;
}

/// Supervisor-level operations: plugin sandbox management, recording capture,
/// and media asset reconciliation.
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
    /// Tears down the plugin sandbox with the given ID.
    fn teardown_plugin_sandbox(&mut self, sandbox_id: &str) -> Result<(), RuntimeError>;
    /// Restarts the plugin sandbox with the given ID.
    fn restart_plugin_sandbox(&mut self, sandbox_id: &str) -> Result<(), RuntimeError>;
    /// Applies a backend policy override to the runtime.
    fn set_backend_policy(&mut self, request: BackendPolicyOverride) -> Result<(), RuntimeError>;
}
