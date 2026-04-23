use super::*;

/// Aggregate event and fault counters collected over a soak run.
///
/// Built from a [`RuntimeSupervisorReport`] via `soak_receipt()`.  Used in
/// integration tests to assert that a lifecycle scenario produced the expected
/// number of restarts, faults, and transport events.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSoakReceipt {
    /// Number of event streams captured in this soak run.
    pub event_stream_count: usize,
    /// Total number of runtime restarts observed.
    pub restart_count: u64,
    /// Total number of runtime stop events observed.
    pub stop_count: u64,
    /// Number of watchdog-triggered restarts observed.
    pub watchdog_restart_count: u32,
    /// Whether safe mode was enabled during the soak run.
    pub safe_mode_enabled: bool,
    /// Whether runtime readiness was degraded at the end of the soak run.
    pub readiness_degraded: bool,
    /// Total number of plugin fault events observed.
    pub plugin_fault_count: usize,
    /// Total number of recovery events observed.
    pub recovery_event_count: usize,
    /// Total number of sandbox lifecycle events observed.
    pub lifecycle_event_count: usize,
    /// Total number of sandbox transport events observed.
    pub transport_event_count: usize,
    /// Total number of sandbox heartbeat events observed.
    pub heartbeat_event_count: usize,
    /// Total number of block dispatch events observed.
    pub block_dispatch_event_count: usize,
    /// Total number of lease rollover events observed.
    pub lease_rollover_event_count: usize,
    /// Total number of broker invalidation events observed.
    pub invalidation_event_count: usize,
    /// Total number of completion slot events observed.
    pub completion_slot_event_count: usize,
    /// Total number of transport fault events observed.
    pub transport_fault_event_count: usize,
    /// Total number of broker failure events observed.
    pub broker_failure_event_count: usize,
    /// Total number of sandbox operation failure events observed.
    pub sandbox_operation_failure_event_count: usize,
    /// Peak number of simultaneously attached transport sessions.
    pub peak_attached_sessions: usize,
    /// Peak number of recovery sessions overlapping with the realtime thread.
    pub peak_recovery_overlap_sessions: usize,
    /// Peak number of lingering (stale) sessions.
    pub peak_lingering_sessions: usize,
    /// Number of pending cleanup waves at the end of the soak run.
    pub pending_cleanup_waves: usize,
    /// Number of plugin sandboxes in the ready state at the end of the soak run.
    pub plugin_ready_sandbox_count: usize,
    /// Number of plugin sandboxes in a degraded state at the end of the soak run.
    pub plugin_degraded_sandbox_count: usize,
    /// Number of plugin sandboxes in a faulted state at the end of the soak run.
    pub plugin_faulted_sandbox_count: usize,
    /// Number of plugin sandboxes currently restarting at the end of the soak run.
    pub plugin_restarting_sandbox_count: usize,
    /// Number of quarantined plugin sandboxes at the end of the soak run.
    pub plugin_quarantined_sandbox_count: usize,
    /// Total number of recall handoff stages.
    pub recall_stage_count: usize,
    /// Number of recall handoff stages with a recovered state.
    pub recovered_recall_stage_count: usize,
    /// Number of recall handoff stages with an unavailable state.
    pub unavailable_recall_stage_count: usize,
    /// Recovery restart intent from the most recent recovery event, if any.
    pub last_recovery_intent: Option<RecoveryRestartIntent>,
    /// Reason for the most recent runtime stop, if any.
    pub last_stop_reason: Option<StopReason>,
    /// Human-readable one-line summary.
    pub summary: String,
}

/// Multi-lane acceptance check for a configured runtime.
///
/// Built via `RuntimeAcceptanceReceipt::capture()`.  Each boolean lane
/// represents a functional readiness dimension: playback, recording, media,
/// clip-processing, plugin, and recovery.  `runtime_ready_lane_count` out of
/// `runtime_lane_count` must be `true` for the runtime to be considered fully
/// ready for a test or production scenario.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeAcceptanceReceipt {
    /// Total number of readiness lanes evaluated.
    pub runtime_lane_count: usize,
    /// Number of readiness lanes that passed.
    pub runtime_ready_lane_count: usize,
    /// Whether the runtime is ready for playback.
    pub playback_ready: bool,
    /// Whether the runtime is ready for recording.
    pub recording_ready: bool,
    /// Whether the media service is ready.
    pub media_ready: bool,
    /// Whether the clip processing pipeline is ready.
    pub clip_processing_ready: bool,
    /// Whether all plugin sandboxes are ready.
    pub plugin_ready: bool,
    /// Whether the recovery subsystem is ready.
    pub recovery_ready: bool,
    /// Minimum number of trace observations required for acceptance.
    pub minimum_trace_observation_count: usize,
    /// Minimum number of soak events required for acceptance.
    pub minimum_soak_event_count: usize,
    /// Human-readable one-line summary.
    pub summary: String,
}

impl RuntimeAcceptanceReceipt {
    /// Captures a readiness acceptance receipt from the current runtime observation state.
    pub fn capture(runtime: &impl RuntimeObservationApi) -> Self {
        build_runtime_acceptance_receipt(RuntimeAcceptanceReceiptInput {
            readiness: runtime.get_readiness(),
            effective_config: runtime.get_effective_config(),
            control_snapshot: runtime.get_control_snapshot(),
            scheduler_topology_summary: runtime.get_scheduler_topology_summary(),
            recording_capture_snapshot: runtime.get_recording_capture_snapshot(),
            media_service_snapshot: runtime.get_media_service_snapshot(),
            clip_processing_pipeline_snapshot: runtime.get_clip_processing_pipeline_snapshot(),
            plugin_lifecycle_snapshot: runtime.get_plugin_lifecycle_snapshot(),
        })
    }
}

impl RuntimeSupervisorReport {
    /// Produces a soak receipt summarising event and fault counters from this supervisor report.
    pub fn soak_receipt(&self) -> RuntimeSoakReceipt {
        build_runtime_soak_receipt(RuntimeSoakReceiptInput {
            observation: &self.observation,
            event_stream_count: self.events.len(),
        })
    }
}

const RUNTIME_ACCEPTANCE_MIN_TRACE_OBSERVATIONS: usize = 128;
const RUNTIME_ACCEPTANCE_MIN_SOAK_EVENTS: usize = 64;

pub(crate) struct RuntimeSoakReceiptInput<'a> {
    pub(crate) observation: &'a RuntimeObservationReport,
    pub(crate) event_stream_count: usize,
}

pub(crate) struct RuntimeAcceptanceReceiptInput {
    pub(crate) readiness: RuntimeReadiness,
    pub(crate) effective_config: EffectiveRuntimeConfig,
    pub(crate) control_snapshot: RuntimeControlSnapshot,
    pub(crate) scheduler_topology_summary: RuntimeSchedulerTopologySummary,
    pub(crate) recording_capture_snapshot: RuntimeRecordingCaptureSnapshot,
    pub(crate) media_service_snapshot: RuntimeMediaServiceSnapshot,
    pub(crate) clip_processing_pipeline_snapshot: RuntimeClipProcessingPipelineSnapshot,
    pub(crate) plugin_lifecycle_snapshot: RuntimePluginLifecycleSnapshot,
}

pub(crate) fn build_runtime_soak_receipt(input: RuntimeSoakReceiptInput<'_>) -> RuntimeSoakReceipt {
    let observation = input.observation;
    let event_stream_count = input.event_stream_count;
    let recall_handoff = RuntimePluginRecallHandoffSnapshot::from_plugin_chain_snapshot(
        &observation.plugin_chain_snapshot,
    );
    RuntimeSoakReceipt {
        event_stream_count,
        restart_count: observation.control_snapshot.restart_count,
        stop_count: observation.control_snapshot.stop_count,
        watchdog_restart_count: observation.supervision_snapshot.watchdog_restart_count,
        safe_mode_enabled: observation.supervision_snapshot.safe_mode_enabled,
        readiness_degraded: observation.degradation_summary.readiness_degraded,
        plugin_fault_count: observation.observation.plugin_fault_count(),
        recovery_event_count: observation.observation.recovery_event_count(),
        lifecycle_event_count: observation.observation.lifecycle_event_count(),
        transport_event_count: observation.observation.transport_event_count(),
        heartbeat_event_count: observation.observation.heartbeat_event_count(),
        block_dispatch_event_count: observation.observation.block_dispatch_event_count(),
        lease_rollover_event_count: observation.observation.lease_rollover_event_count(),
        invalidation_event_count: observation.observation.invalidation_event_count(),
        completion_slot_event_count: observation.observation.completion_slot_event_count(),
        transport_fault_event_count: observation.observation.transport_fault_event_count(),
        broker_failure_event_count: observation.observation.broker_failure_event_count(),
        sandbox_operation_failure_event_count: observation
            .observation
            .sandbox_operation_failure_event_count(),
        peak_attached_sessions: observation.transport_concurrency_snapshot.peak_attached_sessions,
        peak_recovery_overlap_sessions: observation
            .transport_concurrency_snapshot
            .peak_recovery_overlap_sessions,
        peak_lingering_sessions: observation.transport_concurrency_snapshot.peak_lingering_sessions,
        pending_cleanup_waves: observation.transport_concurrency_snapshot.pending_cleanup_waves.len(),
        plugin_ready_sandbox_count: observation.plugin_lifecycle_snapshot.ready_sandbox_count,
        plugin_degraded_sandbox_count: observation.plugin_lifecycle_snapshot.degraded_sandbox_count,
        plugin_faulted_sandbox_count: observation.plugin_lifecycle_snapshot.faulted_sandbox_count,
        plugin_restarting_sandbox_count: observation
            .plugin_lifecycle_snapshot
            .restarting_sandbox_count,
        plugin_quarantined_sandbox_count: observation
            .plugin_lifecycle_snapshot
            .quarantined_sandbox_count,
        recall_stage_count: recall_handoff.stage_count,
        recovered_recall_stage_count: recall_handoff.recovered_stage_count,
        unavailable_recall_stage_count: recall_handoff.unavailable_stage_count,
        last_recovery_intent: observation
            .observation
            .last_recovery_event()
            .map(|record| record.intent),
        last_stop_reason: observation.control_snapshot.last_stop_reason,
        summary: format!(
            "events={} restarts={} watchdog_restarts={} safe_mode={} degraded={} recoveries={} transport_faults={} sandboxes={}/{}/{}/{} recall={}/{}/{}",
            event_stream_count,
            observation.control_snapshot.restart_count,
            observation.supervision_snapshot.watchdog_restart_count,
            observation.supervision_snapshot.safe_mode_enabled,
            observation.degradation_summary.readiness_degraded,
            observation.observation.recovery_event_count(),
            observation.observation.transport_fault_event_count(),
            observation.plugin_lifecycle_snapshot.ready_sandbox_count,
            observation.plugin_lifecycle_snapshot.degraded_sandbox_count,
            observation.plugin_lifecycle_snapshot.faulted_sandbox_count,
            observation.plugin_lifecycle_snapshot.quarantined_sandbox_count,
            recall_handoff.stage_count,
            recall_handoff.recovered_stage_count,
            recall_handoff.unavailable_stage_count,
        ),
    }
}

pub(crate) fn build_runtime_acceptance_receipt(
    input: RuntimeAcceptanceReceiptInput,
) -> RuntimeAcceptanceReceipt {
    let playback_ready = input.effective_config.block_size > 0
        && input.effective_config.sample_rate.0 > 0
        && input.scheduler_topology_summary.compatible;
    let recording_ready = input.recording_capture_snapshot.capture_ready
        || input.recording_capture_snapshot.last_checkpoint.is_some();
    let media_ready = input.media_service_snapshot.indexed_asset_count > 0
        && !input.media_service_snapshot.invalidation_active
        && matches!(
            input.media_service_snapshot.indexing_state,
            RuntimeMediaIndexingState::Ready
        )
        && matches!(
            input.media_service_snapshot.preview_state,
            RuntimeMediaPreviewState::Ready | RuntimeMediaPreviewState::Previewing
        );
    let clip_processing_ready = input.clip_processing_pipeline_snapshot.clip_count > 0
        && input.clip_processing_pipeline_snapshot.ready_clip_count
            == input.clip_processing_pipeline_snapshot.clip_count
        && input
            .clip_processing_pipeline_snapshot
            .pending_media_clip_count
            == 0
        && input
            .clip_processing_pipeline_snapshot
            .pending_warp_clip_count
            == 0
        && input.clip_processing_pipeline_snapshot.invalid_clip_count == 0;
    let plugin_ready = input.plugin_lifecycle_snapshot.sandbox_count > 0
        && input.plugin_lifecycle_snapshot.ready_sandbox_count
            == input.plugin_lifecycle_snapshot.sandbox_count
        && input.plugin_lifecycle_snapshot.faulted_sandbox_count == 0
        && input.plugin_lifecycle_snapshot.quarantined_sandbox_count == 0;
    let recovery_ready = !matches!(input.readiness, RuntimeReadiness::Failed { .. })
        || input.control_snapshot.restart_count > 0;
    let runtime_ready_lane_count = [
        playback_ready,
        recording_ready,
        media_ready,
        clip_processing_ready,
        plugin_ready,
        recovery_ready,
    ]
    .into_iter()
    .filter(|ready| *ready)
    .count();

    RuntimeAcceptanceReceipt {
        runtime_lane_count: 6,
        runtime_ready_lane_count,
        playback_ready,
        recording_ready,
        media_ready,
        clip_processing_ready,
        plugin_ready,
        recovery_ready,
        minimum_trace_observation_count: RUNTIME_ACCEPTANCE_MIN_TRACE_OBSERVATIONS,
        minimum_soak_event_count: RUNTIME_ACCEPTANCE_MIN_SOAK_EVENTS,
        summary: format!(
            "runtime_lanes={}/{} playback={} recording={} media={} clip_processing={} plugin={} recovery={} trace_target={} soak_target={}",
            runtime_ready_lane_count,
            6,
            playback_ready,
            recording_ready,
            media_ready,
            clip_processing_ready,
            plugin_ready,
            recovery_ready,
            RUNTIME_ACCEPTANCE_MIN_TRACE_OBSERVATIONS,
            RUNTIME_ACCEPTANCE_MIN_SOAK_EVENTS,
        ),
    }
}
