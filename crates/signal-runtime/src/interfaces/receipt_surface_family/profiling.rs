use super::*;

/// Point-in-time performance snapshot combining engine, host, and fault
/// metrics into a single flat receipt.
///
/// Built via `RuntimeObservationReport::profiling_receipt()` or
/// `RuntimeSupervisorReport::profiling_receipt()`.  Host hardware fields are
/// `None` when no `RuntimeHostIoSummary` is available.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeProfilingReceipt {
    /// Sample rate in Hz at the time of the observation.
    pub sample_rate_hz: u32,
    /// Block size in frames at the time of the observation.
    pub block_size: usize,
    /// Total number of blocks processed by the engine.
    pub engine_processed_blocks: u64,
    /// Block sequence number of the most recently processed block, if any.
    pub engine_last_block_sequence: Option<u64>,
    /// Number of nodes in the active processing graph.
    pub engine_node_count: usize,
    /// Number of plugin chain stages in the active graph.
    pub engine_stage_count: usize,
    /// Total graph latency in samples.
    pub engine_total_latency_samples: u32,
    /// Total plugin tail length in samples.
    pub engine_total_tail_samples: u32,
    /// Current CPU load as a percentage.
    pub runtime_cpu_load_percent: f32,
    /// Current graph latency in milliseconds.
    pub runtime_graph_latency_ms: f32,
    /// Cumulative xrun count.
    pub runtime_xrun_count: u64,
    /// Number of currently active plugin sandboxes.
    pub active_plugin_sandboxes: u32,
    /// Whether the runtime readiness is degraded.
    pub readiness_degraded: bool,
    /// Whether the transport gate is active.
    pub transport_gate_active: bool,
    /// Whether the plugin gate is active.
    pub plugin_gate_active: bool,
    /// Number of bound plugin sandboxes in a degraded state.
    pub degraded_bound_plugin_sandboxes: usize,
    /// Number of plugin chain stages with a missing sandbox binding.
    pub missing_bound_plugin_sandboxes: usize,
    /// Number of recovery sessions overlapping with the realtime thread.
    pub recovery_overlap_sessions: usize,
    /// Number of lingering (stale) sessions.
    pub lingering_sessions: usize,
    /// Number of sessions with a detach fault.
    pub detach_faulted_sessions: usize,
    /// Total number of plugin chain stages.
    pub plugin_chain_stage_count: usize,
    /// Number of degraded plugin chain stages.
    pub plugin_chain_degraded_stage_count: usize,
    /// Number of plugin chain stages with a missing sandbox binding.
    pub plugin_chain_missing_binding_stage_count: usize,
    /// Sum of planned latency contributions across all plugin chain stages in samples.
    pub plugin_chain_total_planned_latency_samples: u32,
    /// Sum of realized latency contributions across all plugin chain stages in samples.
    pub plugin_chain_total_realized_latency_samples: u32,
    /// Sum of tail length contributions across all plugin chain stages in samples.
    pub plugin_chain_total_tail_samples: u32,
    /// Peak output level from the most recent block, if available.
    pub output_peak: Option<f32>,
    /// RMS output level from the most recent block, if available.
    pub output_rms: Option<f32>,
    /// Total host audio callback count, if host I/O is available.
    pub host_callback_count: Option<u64>,
    /// Host audio callback interval in milliseconds, if available.
    pub host_callback_interval_ms: Option<f32>,
    /// Host output latency in milliseconds, if available.
    pub host_output_latency_ms: Option<f32>,
    /// Host graph latency in milliseconds, if available.
    pub host_graph_latency_ms: Option<f32>,
    /// Host estimated output latency in milliseconds, if available.
    pub host_estimated_output_latency_ms: Option<f32>,
    /// Number of xruns reported by the host backend, if available.
    pub host_backend_xrun_count: Option<u64>,
    /// Number of callback overruns reported by the host, if available.
    pub host_callback_overrun_count: Option<u64>,
    /// Number of device loss events reported by the host, if available.
    pub host_device_loss_count: Option<u64>,
    /// Number of restart attempts by the host backend, if available.
    pub host_restart_attempt_count: Option<u64>,
    /// Number of restart failures by the host backend, if available.
    pub host_restart_failure_count: Option<u64>,
    /// Number of output samples copied by the host audio pump, if available.
    pub host_copied_output_samples: Option<u64>,
    /// Number of zero-filled output samples produced by the host, if available.
    pub host_zero_filled_output_samples: Option<u64>,
    /// Number of output samples dropped by the host, if available.
    pub host_dropped_output_samples: Option<u64>,
    /// Fault diagnostic receipt for the current observation.
    pub fault_diagnostic_receipt: RuntimeFaultDiagnosticReceipt,
    /// Human-readable one-line summary.
    pub summary: String,
}

impl RuntimeObservationReport {
    /// Returns a point-in-time profiling receipt derived from this observation report.
    pub fn profiling_receipt(&self) -> RuntimeProfilingReceipt {
        build_runtime_profiling_receipt(self, None)
    }
}

impl RuntimeSupervisorReport {
    /// Returns a point-in-time profiling receipt derived from the embedded observation report.
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
