use super::super::*;

impl SignalRuntime {
    pub(crate) fn scheduler_state(&self) -> RuntimeSchedulerState {
        match self.readiness {
            RuntimeReadiness::Failed { .. } | RuntimeReadiness::Stopped => {
                RuntimeSchedulerState::Stopped
            }
            RuntimeReadiness::Degraded { .. } => RuntimeSchedulerState::Degraded,
            RuntimeReadiness::Starting => RuntimeSchedulerState::Configured,
            RuntimeReadiness::Ready => {
                if !self.control.running {
                    RuntimeSchedulerState::Configured
                } else if self.engine.snapshot.graph_id.is_none()
                    || self.engine.snapshot.node_count == 0
                {
                    RuntimeSchedulerState::ReadyIdle
                } else if !self.anticipative_enabled
                    || !self.engine.snapshot.prework_cache_enabled
                    || self.engine.snapshot.anticipative_phase_count == 0
                {
                    RuntimeSchedulerState::RealtimeOnly
                } else {
                    RuntimeSchedulerState::Anticipative
                }
            }
        }
    }

    pub(crate) fn scheduler_phase(&self, state: RuntimeSchedulerState) -> RuntimeExecutionPhase {
        if matches!(
            state,
            RuntimeSchedulerState::Stopped | RuntimeSchedulerState::Configured
        ) {
            return RuntimeExecutionPhase::Idle;
        }
        if matches!(state, RuntimeSchedulerState::Degraded) {
            return RuntimeExecutionPhase::Degraded;
        }
        let last_prework_epoch = self
            .engine
            .snapshot
            .last_prework_service_processing_epoch
            .unwrap_or(0);
        let last_realtime_epoch = self.engine.snapshot.last_processing_epoch.unwrap_or(0);
        if last_prework_epoch > 0 && last_prework_epoch > last_realtime_epoch {
            return RuntimeExecutionPhase::Prework;
        }
        if self.engine.snapshot.processed_blocks == 0 {
            return if self.engine.snapshot.graph_id.is_some()
                || self.engine.snapshot.prework_pending_target_count > 0
            {
                RuntimeExecutionPhase::Priming
            } else {
                RuntimeExecutionPhase::Idle
            };
        }
        RuntimeExecutionPhase::Realtime
    }

    pub(crate) fn scheduler_snapshot(&self) -> RuntimeSchedulerSnapshot {
        let state = self.scheduler_state();
        RuntimeSchedulerSnapshot {
            state,
            phase: self.scheduler_phase(state),
            graph_applied: self.applied_graph.is_some(),
            schedule_applied: self.applied_schedule.is_some(),
            transport_projected: self.applied_transport.is_some(),
            anticipative_enabled: self.anticipative_enabled,
            active_graph_id: self.engine.snapshot.graph_id.clone(),
            phase_count: self.engine.snapshot.phase_count,
            lane_count: self.engine.snapshot.lane_count,
            dispatch_count: self.engine.snapshot.dispatch_count,
            pending_prework_target_count: self.engine.snapshot.prework_pending_target_count,
            processed_block_count: self.engine.snapshot.processed_blocks,
        }
    }

    pub(crate) fn scheduler_topology_summary(&self) -> RuntimeSchedulerTopologySummary {
        self.engine.snapshot.scheduler_topology.clone()
    }

    pub(crate) fn execution_topology_summary(&self) -> RuntimeExecutionTopologySummary {
        RuntimeExecutionTopologySummary::from_snapshot(&self.engine.snapshot)
            .with_plugin_chain_snapshot(&self.plugin_chain_snapshot())
    }
}
