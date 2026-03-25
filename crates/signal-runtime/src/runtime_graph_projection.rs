use crate::interfaces::{
    GraphContractProjection, GraphProjection, PluginBackedNodeBindingProjection, ProjectionReceipt,
    RuntimeError, RuntimeErrorKind, ScheduleProjection,
};

use super::SignalRuntime;

impl SignalRuntime {
    pub(crate) fn apply_plugin_backed_node_bindings_projection(
        &mut self,
        projection: PluginBackedNodeBindingProjection,
    ) -> Result<ProjectionReceipt, RuntimeError> {
        self.require_configured()?;
        self.engine
            .apply_plugin_backed_node_bindings(&projection, self.anticipative_enabled)?;
        self.refresh_prework_service_policy_and_state(None);
        let _ = self.maybe_rebuild_prework_window_from_current_forecast_plan()?;
        Ok(ProjectionReceipt {
            accepted_epoch: self.projection_epoch,
            applied_at_block_boundary: true,
        })
    }

    pub(crate) fn apply_graph_contract_projection_state(
        &mut self,
        projection: GraphContractProjection,
    ) -> Result<ProjectionReceipt, RuntimeError> {
        if projection.graph_id.is_empty() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "graph_id must not be empty",
            ));
        }

        self.require_configured()?;
        self.projection_epoch = self.projection_epoch.saturating_add(1);
        self.engine
            .apply_graph_contract_projection(&projection, self.anticipative_enabled)?;
        self.refresh_prework_service_policy_and_state(None);
        self.refresh_scheduler_topology_summary();
        let _ = self.maybe_rebuild_prework_window_from_current_forecast_plan()?;
        Ok(ProjectionReceipt {
            accepted_epoch: self.projection_epoch,
            applied_at_block_boundary: true,
        })
    }

    pub(crate) fn apply_graph_projection_state(
        &mut self,
        projection: GraphProjection,
    ) -> Result<ProjectionReceipt, RuntimeError> {
        if projection.graph_id.is_empty() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "graph_id must not be empty",
            ));
        }

        self.projection_epoch = self.projection_epoch.saturating_add(1);
        self.engine
            .apply_graph_projection(&projection, self.anticipative_enabled)?;
        self.refresh_prework_service_policy_and_state(None);
        self.applied_graph = Some(projection);
        self.refresh_scheduler_topology_summary();
        let _ = self.maybe_rebuild_prework_window_from_current_forecast_plan()?;
        Ok(ProjectionReceipt {
            accepted_epoch: self.projection_epoch,
            applied_at_block_boundary: true,
        })
    }

    pub(crate) fn apply_schedule_projection_state(
        &mut self,
        projection: ScheduleProjection,
    ) -> Result<ProjectionReceipt, RuntimeError> {
        if projection.schedule_id.is_empty() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "schedule_id must not be empty",
            ));
        }

        self.projection_epoch = self.projection_epoch.saturating_add(1);
        self.applied_schedule = Some(projection);
        self.refresh_scheduler_topology_summary();
        self.refresh_prework_service_policy_and_state(None);
        let _ = self.maybe_rebuild_prework_window_from_current_forecast_plan()?;
        Ok(ProjectionReceipt {
            accepted_epoch: self.projection_epoch,
            applied_at_block_boundary: true,
        })
    }
}
