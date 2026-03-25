use super::super::*;

impl SignalRuntime {
    pub(crate) fn set_prework_forecast_policy_internal(
        &mut self,
        policy: Option<RuntimePreworkForecastPolicy>,
    ) {
        self.prework_forecast_policy = policy.clone();
        self.engine.snapshot.prework_forecast_policy_configured = policy.is_some();
        self.engine
            .snapshot
            .prework_forecast_policy_target_window_blocks =
            policy.as_ref().map(|policy| policy.target_window_blocks);
    }

    pub(crate) fn set_prework_forecast_requested_mode_internal(
        &mut self,
        mode: RuntimePreworkForecastMode,
    ) {
        self.prework_forecast_requested_mode = mode;
        self.engine.snapshot.prework_forecast_requested_mode = mode;
    }

    pub(crate) fn set_prework_forecast_mode_internal(&mut self, mode: RuntimePreworkForecastMode) {
        self.prework_forecast_mode = mode;
        self.engine.snapshot.prework_forecast_mode = mode;
    }

    pub(crate) fn set_prework_forecast_profile_internal(
        &mut self,
        selection: Option<RuntimePreworkForecastProfileSelection>,
        source: Option<RuntimePreworkForecastProfileSource>,
    ) {
        self.prework_forecast_profile = selection;
        self.prework_forecast_profile_source = source;
        self.engine.snapshot.prework_forecast_profile =
            selection.map(|selection| selection.profile);
        self.engine.snapshot.prework_forecast_profile_source = source;
        self.engine
            .snapshot
            .prework_forecast_profile_target_window_override =
            selection.and_then(|selection| selection.target_window_blocks_override);
    }

    pub(crate) fn default_prework_forecast_profile_selection_for_runtime_profile(
        profile: RuntimeProfile,
    ) -> RuntimePreworkForecastProfileSelection {
        RuntimePreworkForecastProfileSelection {
            profile: match profile {
                RuntimeProfile::Local => RuntimePreworkForecastProfile::Local,
                RuntimeProfile::Server => RuntimePreworkForecastProfile::Server,
            },
            target_window_blocks_override: None,
        }
    }

    pub(crate) fn prework_forecast_policy_for_profile(
        selection: RuntimePreworkForecastProfileSelection,
    ) -> RuntimePreworkForecastPolicy {
        let mut policy = match selection.profile {
            RuntimePreworkForecastProfile::Local => RuntimePreworkForecastPolicy {
                target_window_blocks: 2,
                prepare_budget_per_cycle: 2,
                buffer_seed_offset: 0,
                transport_playing: true,
                transport_tempo_bpm: 126.0,
                transport_loop_length_blocks: 16,
                parameter_target: "engine.local.drive".into(),
                parameter_cycle_length: 8,
            },
            RuntimePreworkForecastProfile::Server => RuntimePreworkForecastPolicy {
                target_window_blocks: 2,
                prepare_budget_per_cycle: 2,
                buffer_seed_offset: 17,
                transport_playing: true,
                transport_tempo_bpm: 122.0,
                transport_loop_length_blocks: 24,
                parameter_target: "engine.server.balance".into(),
                parameter_cycle_length: 6,
            },
        };
        if let Some(target_window_blocks) = selection.target_window_blocks_override {
            policy.target_window_blocks = target_window_blocks;
        }
        policy
    }

    pub(crate) fn reconcile_prework_forecast_mode_state(&mut self) -> Result<(), RuntimeError> {
        match self.prework_forecast_requested_mode {
            RuntimePreworkForecastMode::RuntimeRoleDefault => {
                let selection =
                    Self::default_prework_forecast_profile_selection_for_runtime_profile(
                        self.config.profile,
                    );
                let policy = Self::prework_forecast_policy_for_profile(selection);
                self.set_prework_forecast_profile_internal(
                    Some(selection),
                    Some(RuntimePreworkForecastProfileSource::RuntimeRoleDefault),
                );
                self.set_prework_forecast_policy_internal(Some(policy));
            }
            RuntimePreworkForecastMode::ExplicitProfile => {
                let selection = self.prework_forecast_profile.ok_or_else(|| {
                    RuntimeError::new(
                        RuntimeErrorKind::InvalidState,
                        "explicit forecast mode requires a stored forecast profile selection",
                    )
                })?;
                let policy = Self::prework_forecast_policy_for_profile(selection);
                self.set_prework_forecast_profile_internal(
                    Some(selection),
                    Some(RuntimePreworkForecastProfileSource::ExplicitSelection),
                );
                self.set_prework_forecast_policy_internal(Some(policy));
            }
            RuntimePreworkForecastMode::RawPolicyOverride => {
                let policy = self.prework_forecast_policy.clone().ok_or_else(|| {
                    RuntimeError::new(
                        RuntimeErrorKind::InvalidState,
                        "raw forecast override mode requires a stored forecast policy",
                    )
                })?;
                self.set_prework_forecast_profile_internal(
                    None,
                    Some(RuntimePreworkForecastProfileSource::RawPolicyOverride),
                );
                self.set_prework_forecast_policy_internal(Some(policy));
            }
            RuntimePreworkForecastMode::Disabled => {}
        }

        let effective_mode = if self.anticipative_enabled {
            self.prework_forecast_requested_mode
        } else {
            RuntimePreworkForecastMode::Disabled
        };
        self.set_prework_forecast_mode_internal(effective_mode);
        Ok(())
    }

    pub(crate) fn set_prework_forecast_mode_state(
        &mut self,
        mode: RuntimePreworkForecastMode,
    ) -> Result<(), RuntimeError> {
        self.require_configured()?;
        let previous_requested_mode = self.prework_forecast_requested_mode;
        let previous_effective_mode = self.prework_forecast_mode;
        let previous_profile = self.prework_forecast_profile;
        let previous_profile_source = self.prework_forecast_profile_source;
        let previous_policy = self.prework_forecast_policy.clone();
        if mode == RuntimePreworkForecastMode::Disabled
            && self.prework_forecast_mode != RuntimePreworkForecastMode::Disabled
        {
            self.engine
                .invalidate_prework_cache(RuntimePreworkInvalidationReason::PlanningDisabled);
        }
        self.set_prework_forecast_requested_mode_internal(mode);
        match mode {
            RuntimePreworkForecastMode::Disabled => {
                self.reconcile_prework_forecast_mode_state()?;
                self.invalidate_prework_for_forecast_plan_change_if_needed(
                    previous_requested_mode,
                    previous_effective_mode,
                    previous_profile,
                    previous_profile_source,
                    previous_policy,
                )?;
                self.refresh_prework_service_policy_and_state(None);
                Ok(())
            }
            RuntimePreworkForecastMode::RuntimeRoleDefault => {
                self.reconcile_prework_forecast_mode_state()?;
                self.invalidate_prework_for_forecast_plan_change_if_needed(
                    previous_requested_mode,
                    previous_effective_mode,
                    previous_profile,
                    previous_profile_source,
                    previous_policy,
                )?;
                self.refresh_prework_service_policy_and_state(None);
                Ok(())
            }
            RuntimePreworkForecastMode::ExplicitProfile => Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "explicit-profile forecast mode requires a profile selection",
            )),
            RuntimePreworkForecastMode::RawPolicyOverride => Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "raw-policy forecast mode requires an explicit forecast policy",
            )),
        }
    }
}
