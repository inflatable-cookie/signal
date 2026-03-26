use signal_ipc::SharedMemoryTransportPayload;
use signal_plugin::{CompletionState, SandboxWatchdogState, WatchdogTriggerReason};
use signal_runtime::PluginSandboxInstanceStateRecord;

#[derive(Clone, Debug)]
pub(crate) struct LifecycleRunSummary {
    pub(crate) sandbox_id: String,
    pub(crate) control_requests: usize,
    pub(crate) control_responses: usize,
    pub(crate) heartbeat_responses: usize,
    pub(crate) processed_blocks: usize,
    pub(crate) engine_processed_blocks: usize,
    pub(crate) last_control_message: String,
    pub(crate) last_completion_state: CompletionState,
    pub(crate) last_block_sequence: u64,
    pub(crate) last_engine_graph_id: Option<String>,
    pub(crate) last_engine_output_peak: Option<f32>,
    pub(crate) last_engine_output_rms: Option<f32>,
    pub(crate) last_output_event_count: usize,
    pub(crate) last_parameter_event_count: usize,
    pub(crate) last_parameter_gesture_event_count: usize,
    pub(crate) last_parameter_modulation_event_count: usize,
    pub(crate) last_note_event_count: usize,
    pub(crate) last_note_expression_event_count: usize,
    pub(crate) last_midi_event_count: usize,
    pub(crate) last_generated_event_bytes: u32,
    pub(crate) last_output_first_sample: Option<f32>,
    pub(crate) deadline_misses: u32,
    pub(crate) heartbeat_misses: u32,
    pub(crate) watchdog_triggered: bool,
    pub(crate) watchdog_trigger_reason: Option<WatchdogTriggerReason>,
    pub(crate) current_watchdog_triggered: bool,
    pub(crate) watchdog: SandboxWatchdogState,
    pub(crate) processing_epoch: u64,
    pub(crate) shared_memory_lease_id: String,
    pub(crate) transport: Option<SharedMemoryTransportPayload>,
    pub(crate) last_plugin_state: Option<PluginSandboxInstanceStateRecord>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct RecoveryHistory {
    pub(crate) control_requests: usize,
    pub(crate) control_responses: usize,
    pub(crate) heartbeat_responses: usize,
    pub(crate) processed_blocks: usize,
    pub(crate) engine_processed_blocks: usize,
    pub(crate) deadline_misses: u32,
    pub(crate) heartbeat_misses: u32,
    pub(crate) watchdog_triggered: bool,
    pub(crate) watchdog_trigger_reason: Option<WatchdogTriggerReason>,
}

impl LifecycleRunSummary {
    pub(crate) fn recovery_history(&self) -> RecoveryHistory {
        RecoveryHistory {
            control_requests: self.control_requests,
            control_responses: self.control_responses,
            heartbeat_responses: self.heartbeat_responses,
            processed_blocks: self.processed_blocks,
            engine_processed_blocks: self.engine_processed_blocks,
            deadline_misses: self.deadline_misses,
            heartbeat_misses: self.heartbeat_misses,
            watchdog_triggered: self.watchdog_triggered,
            watchdog_trigger_reason: self.watchdog_trigger_reason,
        }
    }

    pub(crate) fn apply_recovery_history(&mut self, history: RecoveryHistory) {
        self.control_requests = self
            .control_requests
            .saturating_add(history.control_requests);
        self.control_responses = self
            .control_responses
            .saturating_add(history.control_responses);
        self.heartbeat_responses = self
            .heartbeat_responses
            .saturating_add(history.heartbeat_responses);
        self.processed_blocks = self
            .processed_blocks
            .saturating_add(history.processed_blocks);
        self.engine_processed_blocks = self
            .engine_processed_blocks
            .saturating_add(history.engine_processed_blocks);
        self.deadline_misses = self.deadline_misses.saturating_add(history.deadline_misses);
        self.heartbeat_misses = self
            .heartbeat_misses
            .saturating_add(history.heartbeat_misses);
        self.watchdog_triggered |= history.watchdog_triggered;
        self.current_watchdog_triggered = false;
        if self.watchdog_trigger_reason.is_none() {
            self.watchdog_trigger_reason = history.watchdog_trigger_reason;
        }
    }
}
