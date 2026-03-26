use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeProfilingReceipt {
    pub sample_rate_hz: u32,
    pub block_size: usize,
    pub engine_processed_blocks: u64,
    pub engine_last_block_sequence: Option<u64>,
    pub engine_node_count: usize,
    pub engine_stage_count: usize,
    pub engine_total_latency_samples: u32,
    pub engine_total_tail_samples: u32,
    pub runtime_cpu_load_percent: f32,
    pub runtime_graph_latency_ms: f32,
    pub runtime_xrun_count: u64,
    pub active_plugin_sandboxes: u32,
    pub readiness_degraded: bool,
    pub transport_gate_active: bool,
    pub plugin_gate_active: bool,
    pub degraded_bound_plugin_sandboxes: usize,
    pub missing_bound_plugin_sandboxes: usize,
    pub recovery_overlap_sessions: usize,
    pub lingering_sessions: usize,
    pub detach_faulted_sessions: usize,
    pub plugin_chain_stage_count: usize,
    pub plugin_chain_degraded_stage_count: usize,
    pub plugin_chain_missing_binding_stage_count: usize,
    pub plugin_chain_total_planned_latency_samples: u32,
    pub plugin_chain_total_realized_latency_samples: u32,
    pub plugin_chain_total_tail_samples: u32,
    pub output_peak: Option<f32>,
    pub output_rms: Option<f32>,
    pub host_callback_count: Option<u64>,
    pub host_callback_interval_ms: Option<f32>,
    pub host_output_latency_ms: Option<f32>,
    pub host_graph_latency_ms: Option<f32>,
    pub host_estimated_output_latency_ms: Option<f32>,
    pub host_backend_xrun_count: Option<u64>,
    pub host_callback_overrun_count: Option<u64>,
    pub host_device_loss_count: Option<u64>,
    pub host_restart_attempt_count: Option<u64>,
    pub host_restart_failure_count: Option<u64>,
    pub host_copied_output_samples: Option<u64>,
    pub host_zero_filled_output_samples: Option<u64>,
    pub host_dropped_output_samples: Option<u64>,
    pub fault_diagnostic_receipt: RuntimeFaultDiagnosticReceipt,
    pub summary: String,
}

impl RuntimeObservationReport {
    pub fn profiling_receipt(&self) -> RuntimeProfilingReceipt {
        build_runtime_profiling_receipt(self, None)
    }
}

impl RuntimeSupervisorReport {
    pub fn profiling_receipt(&self) -> RuntimeProfilingReceipt {
        self.observation.profiling_receipt()
    }
}

pub(crate) fn build_runtime_profiling_receipt(
    observation: &RuntimeObservationReport,
    host_io: Option<&RuntimeHostIoSummary>,
) -> RuntimeProfilingReceipt {
    let plugin_chain = &observation.execution_topology_summary.plugin_chain;
    let fault_diagnostic_receipt = RuntimeFaultDiagnosticReceipt::capture(
        &observation.fault_status,
        &observation.interruption_summary,
        &observation.degradation_summary,
        &observation.engine_block_snapshot,
        observation.last_deferred_service_receipt.as_ref(),
        host_io,
    );
    let fault_diagnostic_primary_family = fault_diagnostic_receipt.primary_family;
    RuntimeProfilingReceipt {
        sample_rate_hz: observation.effective_config.sample_rate.0,
        block_size: observation.effective_config.block_size,
        engine_processed_blocks: observation.engine_block_snapshot.processed_blocks,
        engine_last_block_sequence: observation.engine_block_snapshot.last_block_sequence,
        engine_node_count: observation.engine_block_snapshot.node_count,
        engine_stage_count: observation.engine_block_snapshot.stage_count,
        engine_total_latency_samples: observation.engine_block_snapshot.total_latency_samples,
        engine_total_tail_samples: observation.engine_block_snapshot.total_tail_samples,
        runtime_cpu_load_percent: observation.diagnostics_snapshot.cpu_load_percent,
        runtime_graph_latency_ms: observation.diagnostics_snapshot.graph_latency_ms,
        runtime_xrun_count: observation.diagnostics_snapshot.xruns,
        active_plugin_sandboxes: observation.diagnostics_snapshot.active_plugin_sandboxes,
        readiness_degraded: observation.degradation_summary.readiness_degraded,
        transport_gate_active: observation.degradation_summary.transport_gate_active,
        plugin_gate_active: observation.degradation_summary.plugin_gate_active,
        degraded_bound_plugin_sandboxes: observation
            .degradation_summary
            .degraded_bound_plugin_sandboxes,
        missing_bound_plugin_sandboxes: observation
            .degradation_summary
            .missing_bound_plugin_sandboxes,
        recovery_overlap_sessions: observation.degradation_summary.recovery_overlap_sessions,
        lingering_sessions: observation.degradation_summary.lingering_sessions,
        detach_faulted_sessions: observation.degradation_summary.detach_faulted_sessions,
        plugin_chain_stage_count: plugin_chain.stage_count,
        plugin_chain_degraded_stage_count: plugin_chain.degraded_stage_count,
        plugin_chain_missing_binding_stage_count: plugin_chain.missing_binding_stage_count,
        plugin_chain_total_planned_latency_samples: plugin_chain.total_planned_latency_samples,
        plugin_chain_total_realized_latency_samples: plugin_chain.total_realized_latency_samples,
        plugin_chain_total_tail_samples: plugin_chain.total_tail_samples,
        output_peak: observation.diagnostics_snapshot.last_output_peak,
        output_rms: observation.diagnostics_snapshot.last_output_rms,
        host_callback_count: host_io.map(|host_io| host_io.audio_pump.callback_count),
        host_callback_interval_ms: host_io.map(|host_io| host_io.clocking.callback_interval_ms),
        host_output_latency_ms: host_io.map(|host_io| host_io.latency.output_latency_ms),
        host_graph_latency_ms: host_io.map(|host_io| host_io.latency.graph_latency_ms),
        host_estimated_output_latency_ms: host_io
            .map(|host_io| host_io.latency.estimated_output_latency_ms),
        host_backend_xrun_count: host_io.map(|host_io| host_io.hardware.xrun_count),
        host_callback_overrun_count: host_io.map(|host_io| host_io.hardware.callback_overrun_count),
        host_device_loss_count: host_io.map(|host_io| host_io.hardware.device_loss_count),
        host_restart_attempt_count: host_io.map(|host_io| host_io.hardware.restart_attempt_count),
        host_restart_failure_count: host_io.map(|host_io| host_io.hardware.restart_failure_count),
        host_copied_output_samples: host_io.map(|host_io| host_io.audio_pump.copied_output_samples),
        host_zero_filled_output_samples: host_io
            .map(|host_io| host_io.audio_pump.zero_filled_output_samples),
        host_dropped_output_samples: host_io
            .map(|host_io| host_io.audio_pump.dropped_output_samples),
        fault_diagnostic_receipt,
        summary: format!(
            "sample_rate={} block_size={} engine_blocks={} cpu_load={:.3} xruns={} host_callbacks={:?} degraded={} gates={}/{} plugin_chain={}/degraded={}/missing={} sessions={}/{}/{} primary_family={:?}",
            observation.effective_config.sample_rate.0,
            observation.effective_config.block_size,
            observation.engine_block_snapshot.processed_blocks,
            observation.diagnostics_snapshot.cpu_load_percent,
            observation.diagnostics_snapshot.xruns,
            host_io.map(|host_io| host_io.audio_pump.callback_count),
            observation.degradation_summary.readiness_degraded,
            observation.degradation_summary.transport_gate_active,
            observation.degradation_summary.plugin_gate_active,
            plugin_chain.stage_count,
            plugin_chain.degraded_stage_count,
            plugin_chain.missing_binding_stage_count,
            observation.degradation_summary.recovery_overlap_sessions,
            observation.degradation_summary.lingering_sessions,
            observation.degradation_summary.detach_faulted_sessions,
            fault_diagnostic_primary_family,
        ),
    }
}
