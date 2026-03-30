use super::*;

impl RuntimeLifecycleApi for SignalRuntime {
    fn handshake(&mut self, request: HandshakeRequest) -> Result<HandshakeResponse, RuntimeError> {
        if request.client_version.is_empty() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "client_version must not be empty",
            ));
        }
        if matches!(request.max_sample_rate_hint, Some(0)) {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "max_sample_rate_hint must be positive when provided",
            ));
        }

        self.control.handshaken = true;
        self.control.handshake_count = self.control.handshake_count.saturating_add(1);
        self.control.last_client_version = Some(request.client_version.clone());

        Ok(HandshakeResponse {
            runtime_version: env!("CARGO_PKG_VERSION").to_string(),
            protocol_version: 1,
            supports_anticipative: true,
            supports_dynamic_reconfigure: true,
            max_channels: 2048,
            max_sample_rate: request.max_sample_rate_hint.unwrap_or(192_000),
        })
    }

    fn configure(&mut self, request: RuntimeConfigRequest) -> Result<(), RuntimeError> {
        self.configure_runtime_state(request)
    }

    fn start(&mut self) -> Result<(), RuntimeError> {
        self.start_runtime_state()
    }

    fn stop(&mut self, reason: StopReason) -> Result<(), RuntimeError> {
        self.stop_runtime_state(reason)
    }

    fn restart(&mut self, request: RestartRequest) -> Result<(), RuntimeError> {
        self.require_handshake()?;
        if request.reconfigure.is_none() {
            self.require_configured()?;
        }
        if self.control.running {
            self.stop(StopReason::DeviceReconfigure)?;
        }
        if let Some(config) = request.reconfigure {
            self.configure(config)?;
        }
        self.control.restart_count = self.control.restart_count.saturating_add(1);
        self.start()
    }

    fn set_safe_mode(&mut self, request: SafeModeRequest) -> Result<(), RuntimeError> {
        self.set_safe_mode_state(request)
    }
}

impl RuntimeProjectionApi for SignalRuntime {
    fn set_prework_service_pressure(
        &mut self,
        pressure: RuntimePreworkServicePressure,
    ) -> Result<(), RuntimeError> {
        self.require_configured()?;
        self.engine.set_prework_service_pressure(pressure);
        self.refresh_prework_service_policy_and_state(None);
        Ok(())
    }

    fn set_prework_forecast_mode(
        &mut self,
        mode: RuntimePreworkForecastMode,
    ) -> Result<(), RuntimeError> {
        SignalRuntime::set_prework_forecast_mode(self, mode)
    }

    fn set_prework_forecast_profile(
        &mut self,
        selection: RuntimePreworkForecastProfileSelection,
    ) -> Result<(), RuntimeError> {
        SignalRuntime::set_prework_forecast_profile(self, selection)
    }

    fn set_prework_forecast_policy(
        &mut self,
        policy: RuntimePreworkForecastPolicy,
    ) -> Result<(), RuntimeError> {
        SignalRuntime::set_prework_forecast_policy(self, policy)
    }

    fn service_prework_lane(
        &mut self,
        processing_epoch: u64,
        cycles: usize,
    ) -> Result<usize, RuntimeError> {
        SignalRuntime::service_prework_lane(self, processing_epoch, cycles)
    }

    fn apply_plugin_backed_node_bindings(
        &mut self,
        projection: PluginBackedNodeBindingProjection,
    ) -> Result<ProjectionReceipt, RuntimeError> {
        self.apply_plugin_backed_node_bindings_projection(projection)
    }

    fn apply_plugin_placement_policy(
        &mut self,
        policy: RuntimePluginPlacementPolicy,
    ) -> Result<(), RuntimeError> {
        self.require_configured()?;
        self.plugin_placement_policy = policy;
        Ok(())
    }

    fn apply_graph_contract_projection(
        &mut self,
        projection: GraphContractProjection,
    ) -> Result<ProjectionReceipt, RuntimeError> {
        self.apply_graph_contract_projection_state(projection)
    }

    fn apply_graph_projection(
        &mut self,
        projection: GraphProjection,
    ) -> Result<ProjectionReceipt, RuntimeError> {
        self.apply_graph_projection_state(projection)
    }

    fn apply_schedule_projection(
        &mut self,
        projection: ScheduleProjection,
    ) -> Result<ProjectionReceipt, RuntimeError> {
        self.apply_schedule_projection_state(projection)
    }

    fn apply_automation_projection(
        &mut self,
        projection: RuntimeAutomationProjection,
    ) -> Result<ProjectionReceipt, RuntimeError> {
        Self::validate_automation_projection_request(&projection)?;
        self.projection_epoch = self.projection_epoch.saturating_add(1);
        self.automation.apply_projection(projection);
        Ok(ProjectionReceipt {
            accepted_epoch: self.projection_epoch,
            applied_at_block_boundary: true,
        })
    }

    fn apply_tempo_map_projection(
        &mut self,
        projection: RuntimeTempoMapProjection,
    ) -> Result<ProjectionReceipt, RuntimeError> {
        Self::validate_tempo_map_projection_request(&projection)?;
        self.projection_epoch = self.projection_epoch.saturating_add(1);
        self.tempo_map.apply_projection(projection);
        Ok(ProjectionReceipt {
            accepted_epoch: self.projection_epoch,
            applied_at_block_boundary: true,
        })
    }

    fn apply_transport_projection(
        &mut self,
        projection: TransportProjection,
    ) -> Result<(), RuntimeError> {
        self.apply_transport_projection_state(projection)
    }

    fn apply_parameter_batch(&mut self, batch: ParameterBatch) -> Result<(), RuntimeError> {
        self.apply_parameter_batch_state(batch)
    }

    fn apply_hardware_config(
        &mut self,
        request: HardwareConfigRequest,
    ) -> Result<(), RuntimeError> {
        self.apply_hardware_config_state(request)
    }
}
