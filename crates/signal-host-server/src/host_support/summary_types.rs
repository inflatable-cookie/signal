use signal_plugin::{CompletionState, WatchdogTriggerReason};
use signal_runtime::{PluginSandboxInstanceStateRecord, RecoveryRestartIntent, StopReason};

#[derive(Clone, Debug, PartialEq)]
pub struct ServerPayloadSummary {
    pub event_count: usize,
    pub parameter_event_count: usize,
    pub parameter_gesture_event_count: usize,
    pub parameter_modulation_event_count: usize,
    pub note_event_count: usize,
    pub note_expression_event_count: usize,
    pub midi_event_count: usize,
    pub generated_event_bytes: u32,
    pub first_output_sample: Option<f32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServerExecutionSummary {
    pub control_requests: usize,
    pub control_responses: usize,
    pub heartbeat_responses: usize,
    pub processed_blocks: usize,
    pub engine_processed_blocks: usize,
    pub last_control_message: String,
    pub last_completion_state: CompletionState,
    pub last_block_sequence: u64,
    pub last_engine_graph_id: Option<String>,
    pub last_engine_output_peak: Option<f32>,
    pub last_engine_output_rms: Option<f32>,
    pub processing_epoch: u64,
    pub restart_count: u64,
    pub teardown_count: u64,
    pub last_recovery_intent: Option<RecoveryRestartIntent>,
    pub last_stop_reason: Option<StopReason>,
    pub last_plugin_state: Option<PluginSandboxInstanceStateRecord>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServerTransportSummary {
    pub sandbox_id: String,
    pub shared_memory_lease_id: String,
    pub shared_memory_region_id: String,
    pub shared_memory_path: String,
    pub shared_memory_bytes: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServerFaultSummary {
    pub deadline_misses: u32,
    pub heartbeat_misses: u32,
    pub watchdog_triggered: bool,
    pub watchdog_trigger_reason: Option<WatchdogTriggerReason>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ServerRuntimeHostSummary {
    pub scan_roots: Vec<String>,
    pub execution: ServerExecutionSummary,
    pub transport: ServerTransportSummary,
    pub last_payload: ServerPayloadSummary,
    pub faults: ServerFaultSummary,
}
