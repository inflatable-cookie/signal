use super::*;

impl SignalRuntime {
    fn classify_block_deadline_pressure(
        budget_utilization_percent: f32,
        budget_overrun_ns: u64,
    ) -> RuntimeBlockDeadlinePressure {
        if budget_overrun_ns > 0 {
            RuntimeBlockDeadlinePressure::Overrun
        } else if budget_utilization_percent >= BLOCK_DEADLINE_CRITICAL_UTILIZATION_PERCENT {
            RuntimeBlockDeadlinePressure::Critical
        } else if budget_utilization_percent >= BLOCK_DEADLINE_ELEVATED_UTILIZATION_PERCENT {
            RuntimeBlockDeadlinePressure::Elevated
        } else {
            RuntimeBlockDeadlinePressure::Normal
        }
    }

    pub(crate) fn record_block_execution_timing_ns(
        &mut self,
        frame_count: usize,
        execution_time_ns: u64,
    ) {
        let budget_ns = if frame_count == 0 {
            0
        } else {
            ((frame_count as u128) * 1_000_000_000u128 / self.config.sample_rate.0 as u128) as u64
        };
        let budget_ns = budget_ns.max(1);
        let budget_utilization_percent =
            (execution_time_ns as f64 / budget_ns as f64 * 100.0) as f32;
        let budget_overrun_ns = execution_time_ns.saturating_sub(budget_ns);
        let pressure =
            Self::classify_block_deadline_pressure(budget_utilization_percent, budget_overrun_ns);

        self.diagnostics.cpu_load_percent = budget_utilization_percent.max(0.0);
        self.diagnostics.graph_latency_ms = (execution_time_ns as f64 / 1_000_000.0) as f32;

        self.engine.snapshot.last_block_execution_time_ns = Some(execution_time_ns);
        self.engine.snapshot.last_block_deadline_budget_ns = Some(budget_ns);
        self.engine.snapshot.last_block_budget_utilization_percent =
            Some(budget_utilization_percent.max(0.0));
        self.engine.snapshot.last_block_budget_overrun_ns =
            (budget_overrun_ns > 0).then_some(budget_overrun_ns);
        self.engine.snapshot.last_block_deadline_pressure = pressure;
        if budget_overrun_ns > 0 {
            self.engine.snapshot.budget_overrun_count =
                self.engine.snapshot.budget_overrun_count.saturating_add(1);
        }
        self.engine.snapshot.peak_block_execution_time_ns = self
            .engine
            .snapshot
            .peak_block_execution_time_ns
            .max(execution_time_ns);
        self.engine.snapshot.peak_block_budget_utilization_percent = self
            .engine
            .snapshot
            .peak_block_budget_utilization_percent
            .max(budget_utilization_percent);
        self.engine.snapshot.peak_block_budget_overrun_ns = self
            .engine
            .snapshot
            .peak_block_budget_overrun_ns
            .max(budget_overrun_ns);
    }

    pub(crate) fn invalidate_plugin_render_state_for_sandbox(&mut self, sandbox_id: &str) {
        self.engine
            .latest_plugin_node_renders
            .retain(|_, render| render.sandbox_id != sandbox_id);
    }

    pub fn increment_xruns(&mut self) {
        self.diagnostics.xruns = self.diagnostics.xruns.saturating_add(1);
    }

    pub fn record_plugin_sandbox_instance_state(
        &mut self,
        state: PluginSandboxInstanceStateRecord,
    ) {
        self.plugin_lifecycle.record_instance_state(state.clone());
        self.emit(RuntimeEvent::PluginSandboxInstanceState { state });
    }

    pub fn record_heartbeat_cycle(
        &mut self,
        sandbox_id: impl Into<String>,
        stage: HeartbeatCycleStage,
        processing_epoch: Option<u64>,
        block_sequence: Option<u64>,
    ) {
        self.emit(RuntimeEvent::HeartbeatCycle {
            sandbox_id: sandbox_id.into(),
            stage,
            processing_epoch,
            block_sequence,
        });
    }

    pub fn record_block_dispatch(
        &mut self,
        sandbox_id: impl Into<String>,
        lease_id: impl Into<String>,
        processing_epoch: u64,
        block_sequence: u64,
        frame_count: u32,
        stage: BlockDispatchStage,
        completion_state: Option<CompletionState>,
    ) {
        self.emit(RuntimeEvent::BlockDispatch {
            sandbox_id: sandbox_id.into(),
            lease_id: lease_id.into(),
            processing_epoch,
            block_sequence,
            frame_count,
            stage,
            completion_state,
        });
    }

    pub fn record_broker_invalidation(
        &mut self,
        sandbox_id: impl Into<String>,
        lease_id: impl Into<String>,
        processing_epoch: u64,
        block_sequence: Option<u64>,
        stage: BrokerInvalidationStage,
        reason: impl Into<String>,
    ) {
        self.emit(RuntimeEvent::BrokerInvalidation {
            sandbox_id: sandbox_id.into(),
            lease_id: lease_id.into(),
            processing_epoch,
            block_sequence,
            stage,
            reason: reason.into(),
        });
    }

    pub fn record_completion_slot_transition(
        &mut self,
        sandbox_id: impl Into<String>,
        lease_id: impl Into<String>,
        processing_epoch: u64,
        block_sequence: u64,
        stage: CompletionSlotStage,
    ) {
        self.emit(RuntimeEvent::CompletionSlotTransition {
            sandbox_id: sandbox_id.into(),
            lease_id: lease_id.into(),
            processing_epoch,
            block_sequence,
            stage,
        });
    }

    pub fn record_broker_failure(
        &mut self,
        sandbox_id: impl Into<String>,
        lease_id: Option<String>,
        processing_epoch: Option<u64>,
        block_sequence: Option<u64>,
        stage: BrokerFailureStage,
        detail: impl Into<String>,
    ) {
        self.emit(RuntimeEvent::BrokerFailure {
            sandbox_id: sandbox_id.into(),
            lease_id,
            processing_epoch,
            block_sequence,
            stage,
            detail: detail.into(),
        });
    }

    pub fn record_sandbox_operation_failure(
        &mut self,
        sandbox_id: impl Into<String>,
        lease_id: Option<String>,
        processing_epoch: Option<u64>,
        operation: impl Into<String>,
        error_kind: impl Into<String>,
        stage: SandboxOperationFailureStage,
        detail: impl Into<String>,
    ) {
        self.emit(RuntimeEvent::SandboxOperationFailure {
            sandbox_id: sandbox_id.into(),
            lease_id,
            processing_epoch,
            operation: operation.into(),
            error_kind: error_kind.into(),
            stage,
            detail: detail.into(),
        });
    }
}
