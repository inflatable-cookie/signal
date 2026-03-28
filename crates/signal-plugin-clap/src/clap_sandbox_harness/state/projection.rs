use signal_ipc::{PluginInstanceStatePayload, PluginProcessConfigurationPayload};
use signal_plugin::{PluginLifecycleState, PluginReadiness};

use super::ClapSandboxLifecycleHarness;
use crate::clap_sandbox_harness::failure::{lifecycle_state_string, plugin_fault_payload};

impl ClapSandboxLifecycleHarness {
    pub(crate) fn current_instance_id(&self) -> Option<String> {
        self.active_instance
            .as_ref()
            .map(|instance| instance.instance_id.0.clone())
    }

    pub(crate) fn current_lifecycle_state(&self) -> Option<PluginLifecycleState> {
        if self.active {
            Some(PluginLifecycleState::Active)
        } else if self.prepared_epoch.is_some() {
            Some(PluginLifecycleState::Prepared)
        } else if self.active_instance.is_some() {
            Some(PluginLifecycleState::InstanceCreated)
        } else if self.loaded_plugin.is_some() {
            Some(PluginLifecycleState::TypeLoaded)
        } else if self.sandbox_id.is_some() {
            Some(PluginLifecycleState::Discovered)
        } else {
            None
        }
    }

    pub(crate) fn current_readiness(&self) -> PluginReadiness {
        match self.last_fault.clone() {
            Some(fault) => PluginReadiness::from_fault(fault),
            None if self.active => PluginReadiness::Ready,
            None if self.sandbox_id.is_some() => PluginReadiness::Starting,
            None => PluginReadiness::Stopped,
        }
    }

    pub(crate) fn process_configuration_payload(
        &self,
    ) -> Option<PluginProcessConfigurationPayload> {
        Some(PluginProcessConfigurationPayload {
            sample_rate_hz: self.prepared_sample_rate_hz?,
            max_block_frames: self.prepared_max_block_frames?,
            io_layout: crate::io_layout_payload(self.active_io_layout?),
        })
    }

    pub(crate) fn instance_state_payload(
        &self,
        instance_id: &str,
        lifecycle_state: PluginLifecycleState,
    ) -> Option<PluginInstanceStatePayload> {
        let plugin_type_id = self.loaded_plugin.as_ref()?.plugin_type_id.0.clone();
        let readiness = self.current_readiness();
        let (readiness_state, degraded_reasons) = match readiness {
            PluginReadiness::Starting => ("Starting".to_string(), Vec::new()),
            PluginReadiness::Ready => ("Ready".to_string(), Vec::new()),
            PluginReadiness::Stopped => ("Stopped".to_string(), Vec::new()),
            PluginReadiness::Degraded { reasons } => (
                "Degraded".to_string(),
                reasons
                    .into_iter()
                    .map(|reason| reason.0.to_string())
                    .collect(),
            ),
            PluginReadiness::Failed { .. } => ("Failed".to_string(), Vec::new()),
        };

        Some(PluginInstanceStatePayload {
            plugin_type_id,
            instance_id: instance_id.to_string(),
            lifecycle_state: lifecycle_state_string(lifecycle_state).into(),
            readiness_state,
            degraded_reasons,
            active: self.active,
            processing: self.process_configuration_payload(),
            last_fault: self.last_fault.as_ref().map(plugin_fault_payload),
        })
    }
}
