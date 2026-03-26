use signal_hardware::{AudioSampleFormat, HardwareDiagnosticsSnapshot, HardwareLifecycleContract};
use signal_plugin::{CompletionState, PluginRenderContext, WatchdogTriggerReason};
use signal_runtime::{
    PluginSandboxInstanceStateRecord, RecoveryRestartIntent, RuntimeExecutionTopologySummary,
    RuntimeHostAudioStreamState, RuntimeHostAudioTransferPolicy, StopReason,
};

#[derive(Clone, Debug, PartialEq)]
pub struct LocalPayloadSummary {
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
pub struct LocalPluginDispatchSummary {
    pub processing_epoch: u64,
    pub block_sequence: u64,
    pub render_context: PluginRenderContext,
    pub automation_value: Option<f32>,
    pub render_bypass_count: u32,
    pub last_render_bypassed: bool,
    pub last_render_latency_samples: u32,
    pub last_render_tail_samples: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LocalExecutionSummary {
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
pub struct LocalTransportSummary {
    pub sandbox_id: String,
    pub shared_memory_lease_id: String,
    pub shared_memory_region_id: String,
    pub shared_memory_path: String,
    pub shared_memory_bytes: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LocalFaultSummary {
    pub deadline_misses: u32,
    pub heartbeat_misses: u32,
    pub watchdog_triggered: bool,
    pub watchdog_trigger_reason: Option<WatchdogTriggerReason>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocalAudioStreamState {
    Stopped,
    Running,
    Faulted,
}

impl From<LocalAudioStreamState> for RuntimeHostAudioStreamState {
    fn from(value: LocalAudioStreamState) -> Self {
        match value {
            LocalAudioStreamState::Stopped => RuntimeHostAudioStreamState::Stopped,
            LocalAudioStreamState::Running => RuntimeHostAudioStreamState::Running,
            LocalAudioStreamState::Faulted => RuntimeHostAudioStreamState::Faulted,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocalAudioTransferPolicy {
    pub max_callback_frames: usize,
    pub max_transfer_channels: u16,
    pub zero_fill_unwritten_output: bool,
}

impl From<LocalAudioTransferPolicy> for RuntimeHostAudioTransferPolicy {
    fn from(value: LocalAudioTransferPolicy) -> Self {
        Self {
            max_callback_frames: value.max_callback_frames,
            max_transfer_channels: value.max_transfer_channels,
            zero_fill_unwritten_output: value.zero_fill_unwritten_output,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LocalAudioPumpSummary {
    pub stream_state: LocalAudioStreamState,
    pub transfer_policy: LocalAudioTransferPolicy,
    pub callback_count: u64,
    pub last_callback_index: Option<u64>,
    pub total_callback_frames: u64,
    pub total_runtime_output_frames: u64,
    pub copied_output_samples: u64,
    pub zero_filled_output_samples: u64,
    pub dropped_output_samples: u64,
    pub last_callback_output_peak: Option<f32>,
    pub last_runtime_graph_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalHardwareSummary {
    pub device_id: String,
    pub device_name: String,
    pub sample_rate: u32,
    pub buffer_size: usize,
    pub input_channels: u16,
    pub output_channels: u16,
    pub sample_format: AudioSampleFormat,
    pub lifecycle: HardwareLifecycleContract,
    pub simulated: bool,
    pub backend_diagnostics: HardwareDiagnosticsSnapshot,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LocalRuntimeHostSummary {
    pub backend_name: &'static str,
    pub hardware: LocalHardwareSummary,
    pub audio_pump: LocalAudioPumpSummary,
    pub scan_roots: Vec<String>,
    pub execution: LocalExecutionSummary,
    pub transport: LocalTransportSummary,
    pub topology: RuntimeExecutionTopologySummary,
    pub plugin_dispatch: Option<LocalPluginDispatchSummary>,
    pub last_payload: LocalPayloadSummary,
    pub faults: LocalFaultSummary,
}
