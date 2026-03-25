use super::*;

impl SignalRuntime {
    pub(crate) fn refresh_runtime_state(&mut self) {
        match self.readiness {
            RuntimeReadiness::Failed { .. } | RuntimeReadiness::Stopped => {}
            RuntimeReadiness::Starting => {}
            RuntimeReadiness::Ready | RuntimeReadiness::Degraded { .. } => {
                self.readiness = if self.safe_mode_enabled {
                    let mut reasons = vec![DegradedReason("safe-mode-enabled")];
                    if self.supervision.xrun_overload_active {
                        reasons.push(DegradedReason("xrun-overload-recovery-active"));
                    }
                    if self.supervision.watchdog_restart_count
                        >= self.supervision.policy.safe_mode_restart_threshold
                    {
                        reasons.push(DegradedReason("watchdog-restart-threshold-exceeded"));
                    }
                    RuntimeReadiness::Degraded { reasons }
                } else {
                    RuntimeReadiness::Ready
                };
            }
        }
    }

    pub(crate) fn configure_runtime_state(
        &mut self,
        request: RuntimeConfigRequest,
    ) -> Result<(), RuntimeError> {
        self.require_handshake()?;
        if request.block_size == 0 || request.sample_rate.0 == 0 {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "sample_rate and block_size must be non-zero",
            ));
        }

        self.config.sample_rate = request.sample_rate;
        self.config.graph.block_size = request.block_size;
        self.anticipative_enabled = request.anticipative_enabled;
        self.engine
            .invalidate_prework_cache(RuntimePreworkInvalidationReason::RuntimeReconfigured);
        self.reconcile_prework_forecast_mode_state()?;
        self.engine.refresh_planning(self.anticipative_enabled);
        self.safe_mode_enabled = request.realtime_safe_mode;
        self.control.configured = true;
        self.control.running = false;
        self.engine
            .set_prework_service_pressure(RuntimePreworkServicePressure::Normal);
        self.control.configure_count = self.control.configure_count.saturating_add(1);
        self.control.last_reconfigure = Some(request);
        self.applied_parameter_batch = None;
        self.timeline.reset();
        self.automation.reset();
        self.plugin_events.reset();
        self.transport_concurrency.reset();
        self.recording_capture.interrupt_active_capture(
            RuntimeInterruptionClass::Restartable,
            "runtime reconfigured while capture active",
        );
        self.recording_capture.reset_for_runtime_reconfigure();
        self.mark_offline_render_sessions_restartable("runtime reconfigured while render active");
        self.media_pipeline = RuntimeMediaPipelineStateModel::default();
        self.tempo_map = RuntimeTempoMapStateModel::default();
        self.warp_pipeline = RuntimeWarpPipelineStateModel::default();
        self.readiness = RuntimeReadiness::Starting;
        self.refresh_runtime_state();
        self.refresh_prework_service_policy_and_state(None);
        self.refresh_scheduler_topology_summary();
        let _ = self.maybe_rebuild_prework_window_from_current_forecast_plan()?;
        self.refresh_prework_service_policy_and_state(None);
        self.refresh_scheduler_topology_summary();
        self.emit(RuntimeEvent::ReadinessChanged(self.readiness.clone()));
        self.emit(RuntimeEvent::EffectiveConfigChanged(
            self.get_effective_config(),
        ));
        Ok(())
    }

    pub(crate) fn start_runtime_state(&mut self) -> Result<(), RuntimeError> {
        self.require_handshake()?;
        self.require_configured()?;
        if self.control.running {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "runtime is already running",
            ));
        }

        self.readiness = RuntimeReadiness::Ready;
        self.control.running = true;
        self.control.start_count = self.control.start_count.saturating_add(1);
        self.refresh_runtime_state();
        let _ = self.maybe_rebuild_prework_window_from_current_forecast_plan()?;
        self.refresh_prework_service_policy_and_state(None);
        self.emit(RuntimeEvent::ReadinessChanged(self.readiness.clone()));
        Ok(())
    }

    pub(crate) fn stop_runtime_state(&mut self, reason: StopReason) -> Result<(), RuntimeError> {
        if !self.control.running {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "runtime is not running",
            ));
        }

        self.engine
            .invalidate_prework_cache(RuntimePreworkInvalidationReason::RuntimeStopped);
        self.readiness = RuntimeReadiness::Stopped;
        self.control.running = false;
        self.recording_capture.interrupt_active_capture(
            RuntimeInterruptionClass::Restartable,
            "runtime stopped while capture active",
        );
        self.mark_offline_render_sessions_restartable("runtime stopped while render active");
        self.engine
            .set_prework_service_pressure(RuntimePreworkServicePressure::Normal);
        self.control.stop_count = self.control.stop_count.saturating_add(1);
        self.control.last_stop_reason = Some(reason);
        self.refresh_prework_service_policy_and_state(None);
        self.emit(RuntimeEvent::ReadinessChanged(self.readiness.clone()));
        Ok(())
    }

    pub(crate) fn set_safe_mode_state(
        &mut self,
        request: SafeModeRequest,
    ) -> Result<(), RuntimeError> {
        self.safe_mode_enabled = request.enabled;
        if !request.enabled {
            self.supervision.clear_xrun_overload_recovery();
        }
        self.refresh_runtime_state();
        self.emit(RuntimeEvent::ReadinessChanged(self.readiness.clone()));
        self.emit(RuntimeEvent::EffectiveConfigChanged(
            self.get_effective_config(),
        ));
        Ok(())
    }
}
