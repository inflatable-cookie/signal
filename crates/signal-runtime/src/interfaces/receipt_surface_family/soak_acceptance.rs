use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSoakReceipt {
    pub event_stream_count: usize,
    pub restart_count: u64,
    pub stop_count: u64,
    pub watchdog_restart_count: u32,
    pub safe_mode_enabled: bool,
    pub readiness_degraded: bool,
    pub plugin_fault_count: usize,
    pub recovery_event_count: usize,
    pub lifecycle_event_count: usize,
    pub transport_event_count: usize,
    pub heartbeat_event_count: usize,
    pub block_dispatch_event_count: usize,
    pub lease_rollover_event_count: usize,
    pub invalidation_event_count: usize,
    pub completion_slot_event_count: usize,
    pub transport_fault_event_count: usize,
    pub broker_failure_event_count: usize,
    pub sandbox_operation_failure_event_count: usize,
    pub peak_attached_sessions: usize,
    pub peak_recovery_overlap_sessions: usize,
    pub peak_lingering_sessions: usize,
    pub pending_cleanup_waves: usize,
    pub plugin_ready_sandbox_count: usize,
    pub plugin_degraded_sandbox_count: usize,
    pub plugin_faulted_sandbox_count: usize,
    pub plugin_restarting_sandbox_count: usize,
    pub plugin_quarantined_sandbox_count: usize,
    pub recall_stage_count: usize,
    pub recovered_recall_stage_count: usize,
    pub unavailable_recall_stage_count: usize,
    pub last_recovery_intent: Option<RecoveryRestartIntent>,
    pub last_stop_reason: Option<StopReason>,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeAcceptanceReceipt {
    pub runtime_lane_count: usize,
    pub runtime_ready_lane_count: usize,
    pub playback_ready: bool,
    pub recording_ready: bool,
    pub media_ready: bool,
    pub clip_processing_ready: bool,
    pub plugin_ready: bool,
    pub recovery_ready: bool,
    pub minimum_trace_observation_count: usize,
    pub minimum_soak_event_count: usize,
    pub summary: String,
}

impl RuntimeAcceptanceReceipt {
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
