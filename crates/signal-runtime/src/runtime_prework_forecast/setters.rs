use super::super::*;

impl SignalRuntime {
    pub(crate) fn set_prework_forecast_mode(
        &mut self,
        mode: RuntimePreworkForecastMode,
    ) -> Result<(), RuntimeError> {
        self.set_prework_forecast_mode_state(mode)
    }

    pub(crate) fn set_prework_forecast_profile(
        &mut self,
        selection: RuntimePreworkForecastProfileSelection,
    ) -> Result<(), RuntimeError> {
        self.require_configured()?;
        let previous_requested_mode = self.prework_forecast_requested_mode;
        let previous_effective_mode = self.prework_forecast_mode;
        let previous_profile = self.prework_forecast_profile;
        let previous_profile_source = self.prework_forecast_profile_source;
        let previous_policy = self.prework_forecast_policy.clone();
        self.set_prework_forecast_profile_internal(
            Some(selection),
            Some(RuntimePreworkForecastProfileSource::ExplicitSelection),
        );
        self.set_prework_forecast_requested_mode_internal(
            RuntimePreworkForecastMode::ExplicitProfile,
        );
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

    pub(crate) fn set_prework_forecast_policy(
        &mut self,
        policy: RuntimePreworkForecastPolicy,
    ) -> Result<(), RuntimeError> {
        self.require_configured()?;
        let previous_requested_mode = self.prework_forecast_requested_mode;
        let previous_effective_mode = self.prework_forecast_mode;
        let previous_profile = self.prework_forecast_profile;
        let previous_profile_source = self.prework_forecast_profile_source;
        let previous_policy = self.prework_forecast_policy.clone();
        self.set_prework_forecast_profile_internal(
            None,
            Some(RuntimePreworkForecastProfileSource::RawPolicyOverride),
        );
        self.set_prework_forecast_policy_internal(Some(policy));
        self.set_prework_forecast_requested_mode_internal(
            RuntimePreworkForecastMode::RawPolicyOverride,
        );
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
}
