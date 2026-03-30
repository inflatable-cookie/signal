use super::*;

impl core::fmt::Debug for SignalRuntime {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SignalRuntime")
            .field("config", &self.config)
            .field("readiness", &self.readiness)
            .field("safe_mode_enabled", &self.safe_mode_enabled)
            .field("anticipative_enabled", &self.anticipative_enabled)
            .field("active_output_device", &self.active_output_device)
            .field("applied_graph", &self.applied_graph)
            .field("applied_schedule", &self.applied_schedule)
            .field("applied_transport", &self.applied_transport)
            .field("latest_parameter_epoch", &self.latest_parameter_epoch)
            .field("projection_epoch", &self.projection_epoch)
            .field("control", &self.control)
            .field("timeline", &self.timeline)
            .field("automation", &self.automation)
            .field("engine", &self.engine)
            .field("diagnostics", &self.diagnostics)
            .field("supervision", &self.supervision)
            .finish()
    }
}

impl SignalRuntime {
    pub fn new(config: RuntimeConfig) -> Self {
        let mut runtime = Self {
            config,
            readiness: RuntimeReadiness::Stopped,
            safe_mode_enabled: false,
            anticipative_enabled: true,
            active_output_device: None,
            applied_graph: None,
            applied_schedule: None,
            applied_transport: None,
            applied_parameter_batch: None,
            prework_forecast_requested_mode: RuntimePreworkForecastMode::RuntimeRoleDefault,
            prework_forecast_mode: RuntimePreworkForecastMode::Disabled,
            prework_forecast_policy: None,
            prework_forecast_profile: None,
            prework_forecast_profile_source: None,
            latest_parameter_epoch: 0,
            projection_epoch: 0,
            control: RuntimeControlSnapshot::default(),
            timeline: RuntimeTimelineState::default(),
            automation: RuntimeAutomationState::default(),
            plugin_events: RuntimePluginEventState::default(),
            engine: RuntimeEngineState::default(),
            transport_concurrency: RuntimeTransportConcurrencyState::default(),
            plugin_discovery: RuntimePluginDiscoveryStateModel::default(),
            plugin_lifecycle: RuntimePluginLifecycleStateModel::default(),
            plugin_placement_policy: RuntimePluginPlacementPolicy::default(),
            recording_capture: RuntimeRecordingCaptureStateModel::default(),
            metering: RuntimeMeteringStateModel::default(),
            media_pipeline: RuntimeMediaPipelineStateModel::default(),
            tempo_map: RuntimeTempoMapStateModel::default(),
            warp_pipeline: RuntimeWarpPipelineStateModel::default(),
            clip_processing_pipeline: RuntimeClipProcessingPipelineStateModel::default(),
            diagnostics: RuntimeDiagnosticsSnapshot {
                cpu_load_percent: 0.0,
                xruns: 0,
                graph_latency_ms: 0.0,
                active_plugin_sandboxes: 0,
                backend_policy_tier: BackendPolicyTier::Tier0InHost,
                topology_compatible: false,
                topology_issue_count: 0,
                degraded_bound_plugin_sandboxes: 0,
                missing_bound_plugin_sandboxes: 0,
                last_output_peak: None,
                last_output_rms: None,
                momentary_loudness_lufs: None,
                short_term_loudness_lufs: None,
                integrated_loudness_lufs: None,
            },
            supervision: RuntimeSupervisionState::default(),
            last_deferred_service_receipt: RefCell::new(None),
            last_offline_render_session_snapshot: RefCell::new(None),
            last_offline_render_cancellation_receipt: RefCell::new(None),
            last_offline_render_purge_receipt: RefCell::new(None),
            offline_render_executions: HashMap::new(),
            next_subscription: 1,
            sinks: Vec::new(),
        };
        runtime.set_prework_forecast_requested_mode_internal(
            RuntimePreworkForecastMode::RuntimeRoleDefault,
        );
        runtime.set_prework_forecast_mode_internal(RuntimePreworkForecastMode::Disabled);
        runtime.recompute_prework_service_policy_snapshot();
        runtime
    }

    pub fn config(&self) -> RuntimeConfig {
        self.config
    }

    pub(crate) fn reconcile_prework_service_state(&mut self, processing_epoch: Option<u64>) {
        let state = if !self.engine.snapshot.prework_cache_enabled
            || self.prework_forecast_mode == RuntimePreworkForecastMode::Disabled
        {
            RuntimePreworkServiceState::Disabled
        } else if !self.control.running {
            RuntimePreworkServiceState::Paused
        } else if !self.engine.pending_prework_targets.is_empty()
            && (self.engine.snapshot.prework_service_plugin_gate_active
                || self.engine.snapshot.prework_service_transport_gate_active)
        {
            RuntimePreworkServiceState::Yielding
        } else if !self.engine.pending_prework_targets.is_empty() {
            RuntimePreworkServiceState::Pending
        } else {
            RuntimePreworkServiceState::Idle
        };
        self.engine
            .transition_prework_service_state(state, processing_epoch);
    }

    pub fn set_active_output_device(&mut self, device_id: impl Into<String>) {
        self.active_output_device = Some(device_id.into());
        self.emit(RuntimeEvent::HardwareDeviceChanged {
            device_id: self.active_output_device.clone(),
        });
    }

    pub fn set_active_plugin_sandboxes(&mut self, count: u32) {
        self.diagnostics.active_plugin_sandboxes = count;
        self.plugin_lifecycle.set_active_sandbox_count(count);
        self.refresh_prework_service_policy_and_state(None);
        self.emit(RuntimeEvent::PluginSandboxChanged {
            active_sandboxes: self.diagnostics.active_plugin_sandboxes,
        });
    }

    pub fn apply_plugin_node_render_batch(
        &mut self,
        batch: PluginNodeRenderBatch,
    ) -> Result<(), RuntimeError> {
        self.engine.apply_plugin_node_render_batch(batch)
    }

    pub fn set_backend_policy_tier(&mut self, tier: BackendPolicyTier) {
        self.diagnostics.backend_policy_tier = tier;
    }

    pub fn set_cpu_load_percent(&mut self, cpu_load_percent: f32) {
        self.diagnostics.cpu_load_percent = cpu_load_percent.max(0.0);
    }

    pub fn set_graph_latency_ms(&mut self, graph_latency_ms: f32) {
        self.diagnostics.graph_latency_ms = graph_latency_ms.max(0.0);
    }

    pub fn projection_epoch(&self) -> u64 {
        self.projection_epoch
    }

    pub fn reset_block_timeline(&mut self) {
        self.timeline.reset();
    }

    pub fn reset_automation_tracking(&mut self) {
        self.automation.reset();
    }

    pub fn reset_plugin_event_tracking(&mut self) {
        self.plugin_events.reset();
    }

    pub fn process_engine_block(
        &mut self,
        processing_epoch: u64,
        block_sequence: u64,
        buffer: AudioBuffer,
    ) -> Result<RuntimeEngineBlockResult, RuntimeError> {
        if !self.control.configured {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "runtime must be configured before processing engine blocks",
            ));
        }
        let block_start = Instant::now();
        let transport = self.applied_transport;
        let context = self.build_engine_execution_context(processing_epoch, block_sequence);
        let (parameter_batch, automation_metrics) =
            self.current_graph_parameter_batch(&context, buffer.frames().0);
        let pending_transition = self
            .timeline
            .consume_pending_transport_transition(block_sequence);
        let mut result = self
            .engine
            .process_block(context, transport, buffer, parameter_batch)?;
        self.apply_engine_transport_update(
            processing_epoch,
            block_sequence,
            pending_transition,
            &mut result,
        );
        self.finalize_engine_block_result(
            processing_epoch,
            block_sequence,
            automation_metrics,
            block_start,
            &mut result,
        )?;
        Ok(result)
    }
}
