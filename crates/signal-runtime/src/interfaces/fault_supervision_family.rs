use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DegradedReason(pub &'static str);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginFaultKind {
    Timeout,
    Crash,
    ProtocolViolation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeWatchdogTrigger {
    DeadlineMisses,
    HeartbeatMisses,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WatchdogRestartRecord {
    pub sandbox_id: String,
    pub trigger: RuntimeWatchdogTrigger,
    pub processing_epoch: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSupervisionSnapshot {
    pub watchdog_restart_count: u32,
    pub safe_mode_enabled: bool,
    pub xrun_overload_active: bool,
    pub last_watchdog_trigger: Option<RuntimeWatchdogTrigger>,
    pub last_sandbox_id: Option<String>,
    pub last_processing_epoch: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeTimelineSnapshot {
    pub next_block_sequence: u64,
    pub block_sequence_continuity: BlockSequenceContinuityReport,
    pub transport_epoch: u64,
    pub last_transport_transition: Option<RuntimeTransportTransitionKind>,
    pub last_transport_transition_processing_epoch: Option<u64>,
    pub last_transport_transition_block_sequence: Option<u64>,
    pub last_transport_playing: Option<bool>,
    pub last_transport_tempo_bpm: Option<f64>,
    pub last_transport_timeline_position_samples: Option<i64>,
    pub last_transport_loop_start_samples: Option<i64>,
    pub last_transport_loop_end_samples: Option<i64>,
    pub last_engine_block_start_samples: Option<i64>,
    pub last_engine_block_end_samples: Option<i64>,
    pub loop_wrap_count: u64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeTransportObservationSnapshot {
    pub transport_epoch: u64,
    pub projected_playing: Option<bool>,
    pub projected_tempo_bpm: Option<f64>,
    pub projected_timeline_position_samples: Option<i64>,
    pub projected_loop_start_samples: Option<i64>,
    pub projected_loop_end_samples: Option<i64>,
    pub observed_playing: Option<bool>,
    pub observed_tempo_bpm: Option<f64>,
    pub observed_timeline_position_samples: Option<i64>,
    pub observed_loop_start_samples: Option<i64>,
    pub observed_loop_end_samples: Option<i64>,
    pub last_transition: Option<RuntimeTransportTransitionKind>,
    pub last_transition_processing_epoch: Option<u64>,
    pub last_transition_block_sequence: Option<u64>,
    pub last_engine_block_start_samples: Option<i64>,
    pub last_engine_block_end_samples: Option<i64>,
    pub loop_wrap_count: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeRecordingCaptureState {
    Idle,
    Capturing,
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeRecordingCaptureKind {
    Audio,
    Midi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeRecordingCaptureCheckpointClass {
    Armed,
    Streaming,
    Buffered,
    Committed,
    Failed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeRecordingCaptureCheckpointSnapshot {
    pub capture_kind: RuntimeRecordingCaptureKind,
    pub checkpoint_class: RuntimeRecordingCaptureCheckpointClass,
    pub interruption_class: RuntimeInterruptionClass,
    pub take_id: String,
    pub track_id: String,
    pub capture_start_samples: i64,
    pub capture_path: String,
    pub buffered_block_count: u64,
    pub buffered_frame_count: u64,
    pub buffered_event_count: u64,
    pub captured_channel_count: usize,
    pub peak_level: Option<f32>,
    pub pressure_event_count: u64,
    pub last_error: Option<String>,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeRecordingCaptureStartRequest {
    pub capture_kind: RuntimeRecordingCaptureKind,
    pub take_id: String,
    pub track_id: String,
    pub start_samples: i64,
    pub capture_path: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeRecordingCaptureCommitReceipt {
    pub capture_kind: RuntimeRecordingCaptureKind,
    pub take_id: String,
    pub track_id: String,
    pub start_samples: i64,
    pub duration_samples: u32,
    pub channel_count: usize,
    pub peak_level: f32,
    pub capture_path: String,
    pub committed_checkpoint: RuntimeRecordingCaptureCheckpointSnapshot,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeRecordingCaptureSnapshot {
    pub capture_ready: bool,
    pub state: Option<RuntimeRecordingCaptureState>,
    pub capture_kind: Option<RuntimeRecordingCaptureKind>,
    pub active_take_id: Option<String>,
    pub active_track_id: Option<String>,
    pub capture_start_samples: Option<i64>,
    pub active_capture_path: Option<String>,
    pub buffered_block_count: u64,
    pub buffered_frame_count: u64,
    pub buffered_event_count: u64,
    pub captured_channel_count: usize,
    pub peak_level: Option<f32>,
    pub pressure_event_count: u64,
    pub active_checkpoint: Option<RuntimeRecordingCaptureCheckpointSnapshot>,
    pub last_checkpoint: Option<RuntimeRecordingCaptureCheckpointSnapshot>,
    pub last_committed_take_id: Option<String>,
    pub last_committed_path: Option<String>,
    pub last_committed_duration_samples: Option<u32>,
    pub last_error: Option<String>,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeDeferredServiceClass {
    OfflineRenderQueue,
    OfflineRenderPurge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeDeferredServiceDecision {
    Run,
    Defer,
    Throttle,
    Abort,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeDeferredServicePriorityBand {
    RealtimeCritical,
    RecoveryCritical,
    UserVisible,
    Maintenance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeDeferredServiceReason {
    Ready,
    RealtimeActive,
    PendingCleanup,
    RecoveryDegraded,
    SafeMode,
    InvalidRequest,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeDeferredServiceBackpressureSource {
    RealtimeAudio,
    RecoveryOverlap,
    CleanupBacklog,
    SafeMode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeDeferredServiceCancellationCause {
    InvalidRequest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeDeferredServiceReceipt {
    pub work_class: RuntimeDeferredServiceClass,
    pub decision: RuntimeDeferredServiceDecision,
    pub reason: RuntimeDeferredServiceReason,
    pub priority_band: RuntimeDeferredServicePriorityBand,
    pub blocking_priority_band: Option<RuntimeDeferredServicePriorityBand>,
    pub backpressure_source: Option<RuntimeDeferredServiceBackpressureSource>,
    pub starvation_risk: bool,
    pub starved_work_item_count: usize,
    pub cancellation_cause: Option<RuntimeDeferredServiceCancellationCause>,
    pub cancelled_work_item_count: usize,
    pub interruption_class: RuntimeInterruptionClass,
    pub interruption_rebindable: bool,
    pub queued_work_item_count: usize,
    pub admitted_work_item_count: usize,
    pub completed_work_item_count: usize,
    pub deferred_work_item_count: usize,
    pub runtime_running: bool,
    pub safe_mode_enabled: bool,
    pub readiness_degraded: bool,
    pub pending_cleanup_work_items: usize,
    pub pending_deferred_retry_work_items: usize,
    pub recovery_overlap_session_count: usize,
    pub summary: String,
}

impl RuntimeDeferredServiceReceipt {
    pub fn render_multiline(&self) -> String {
        format!(
            concat!(
                "work_class={:?}",
                "\ndecision={:?}",
                "\nreason={:?}",
                "\npriority_band={:?}",
                "\nblocking_priority_band={:?}",
                "\nbackpressure_source={:?}",
                "\nstarvation_risk={}",
                "\nstarved_work_item_count={}",
                "\ncancellation_cause={:?}",
                "\ncancelled_work_item_count={}",
                "\ninterruption_class={:?}",
                "\ninterruption_rebindable={}",
                "\nqueued_work_item_count={}",
                "\nadmitted_work_item_count={}",
                "\ncompleted_work_item_count={}",
                "\ndeferred_work_item_count={}",
                "\nruntime_running={}",
                "\nsafe_mode_enabled={}",
                "\nreadiness_degraded={}",
                "\npending_cleanup_work_items={}",
                "\npending_deferred_retry_work_items={}",
                "\nrecovery_overlap_session_count={}",
                "\nsummary={}",
            ),
            self.work_class,
            self.decision,
            self.reason,
            self.priority_band,
            self.blocking_priority_band,
            self.backpressure_source,
            self.starvation_risk,
            self.starved_work_item_count,
            self.cancellation_cause,
            self.cancelled_work_item_count,
            self.interruption_class,
            self.interruption_rebindable,
            self.queued_work_item_count,
            self.admitted_work_item_count,
            self.completed_work_item_count,
            self.deferred_work_item_count,
            self.runtime_running,
            self.safe_mode_enabled,
            self.readiness_degraded,
            self.pending_cleanup_work_items,
            self.pending_deferred_retry_work_items,
            self.recovery_overlap_session_count,
            self.summary,
        )
    }

    pub fn render_json(&self) -> String {
        format!(
            concat!(
                "{{",
                "\"work_class\":{},",
                "\"decision\":{},",
                "\"reason\":{},",
                "\"priority_band\":{},",
                "\"blocking_priority_band\":{},",
                "\"backpressure_source\":{},",
                "\"starvation_risk\":{},",
                "\"starved_work_item_count\":{},",
                "\"cancellation_cause\":{},",
                "\"cancelled_work_item_count\":{},",
                "\"interruption_class\":{},",
                "\"interruption_rebindable\":{},",
                "\"queued_work_item_count\":{},",
                "\"admitted_work_item_count\":{},",
                "\"completed_work_item_count\":{},",
                "\"deferred_work_item_count\":{},",
                "\"runtime_running\":{},",
                "\"safe_mode_enabled\":{},",
                "\"readiness_degraded\":{},",
                "\"pending_cleanup_work_items\":{},",
                "\"pending_deferred_retry_work_items\":{},",
                "\"recovery_overlap_session_count\":{},",
                "\"summary\":{}",
                "}}"
            ),
            json_string(&format!("{:?}", self.work_class)),
            json_string(&format!("{:?}", self.decision)),
            json_string(&format!("{:?}", self.reason)),
            json_string(&format!("{:?}", self.priority_band)),
            json_option_string(
                self.blocking_priority_band
                    .as_ref()
                    .map(|value| format!("{value:?}"))
                    .as_deref(),
            ),
            json_option_string(
                self.backpressure_source
                    .as_ref()
                    .map(|value| format!("{value:?}"))
                    .as_deref(),
            ),
            self.starvation_risk,
            self.starved_work_item_count,
            json_option_string(
                self.cancellation_cause
                    .as_ref()
                    .map(|value| format!("{value:?}"))
                    .as_deref(),
            ),
            self.cancelled_work_item_count,
            json_string(&format!("{:?}", self.interruption_class)),
            self.interruption_rebindable,
            self.queued_work_item_count,
            self.admitted_work_item_count,
            self.completed_work_item_count,
            self.deferred_work_item_count,
            self.runtime_running,
            self.safe_mode_enabled,
            self.readiness_degraded,
            self.pending_cleanup_work_items,
            self.pending_deferred_retry_work_items,
            self.recovery_overlap_session_count,
            json_option_string(Some(self.summary.as_str())),
        )
    }
}

impl Default for RuntimeDeferredServiceReceipt {
    fn default() -> Self {
        Self {
            work_class: RuntimeDeferredServiceClass::OfflineRenderQueue,
            decision: RuntimeDeferredServiceDecision::Abort,
            reason: RuntimeDeferredServiceReason::InvalidRequest,
            priority_band: RuntimeDeferredServicePriorityBand::UserVisible,
            blocking_priority_band: None,
            backpressure_source: None,
            starvation_risk: false,
            starved_work_item_count: 0,
            cancellation_cause: Some(RuntimeDeferredServiceCancellationCause::InvalidRequest),
            cancelled_work_item_count: 0,
            interruption_class: RuntimeInterruptionClass::Terminal,
            interruption_rebindable: false,
            queued_work_item_count: 0,
            admitted_work_item_count: 0,
            completed_work_item_count: 0,
            deferred_work_item_count: 0,
            runtime_running: false,
            safe_mode_enabled: false,
            readiness_degraded: false,
            pending_cleanup_work_items: 0,
            pending_deferred_retry_work_items: 0,
            recovery_overlap_session_count: 0,
            summary: "class=OfflineRenderQueue decision=Abort reason=InvalidRequest queued=0 admitted=0 completed=0 deferred=0".to_string(),
        }
    }
}
