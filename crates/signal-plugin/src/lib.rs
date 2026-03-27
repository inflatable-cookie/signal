//! Format-neutral plugin host abstractions for Signal.

use signal_ipc::CorrelationId;

mod blocks;
mod event_codec;
mod events;
mod plugin_block_transport;
mod plugin_event_reports;
mod render_context_codec;

pub use blocks::{AudioBlock, BlockPayload, EventPacket};
pub use event_codec::{read_event_from_slice, write_event_to_slice};
pub use events::{
    MidiEvent, NoteEvent, NoteEventKind, NoteExpressionEvent, NoteExpressionKind,
    ParameterGestureEvent, ParameterGesturePhase, ParameterModulationEvent, ParameterValueEvent,
    PluginEvent,
};
pub use render_context_codec::{read_render_context_from_slice, write_render_context_to_slice};

pub use plugin_block_transport::{
    BlockDispatch, BlockProcessResult, BlockProcessingHeader, CompletionSlot, CompletionState,
    SandboxStateMachine, SharedMemoryLayout, SharedMemoryLease, SharedMemoryRegion,
};
pub use plugin_event_reports::{
    AutomationContinuityReport, AutomationContinuitySegment, BlockSequenceContinuityReport,
    BlockSequenceContinuitySegment, EventPacketContinuityReport, EventPacketContinuitySegment,
    EventPacketSummary, ParameterAutomationSummary,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginFormat {
    Clap,
    Vst3,
    Au,
    Lv2,
    Native,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SandboxPolicy {
    Strict,
    Moderate,
    Disabled,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginTypeId(pub String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginInstanceId(pub String);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginFeature {
    AudioEffect,
    Instrument,
    Analyzer,
    Utility,
    NoteEffect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginAudioBusDirection {
    Input,
    Output,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginAudioBusDescriptor {
    pub bus_id: String,
    pub name: String,
    pub direction: PluginAudioBusDirection,
    pub channels: u16,
    pub is_main: bool,
    pub active: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginParameterDomain {
    GenericNormalized,
    Decibels,
    Hertz,
    Seconds,
    Bypass,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginParameterFlags {
    pub automatable: bool,
    pub modulatable: bool,
    pub supports_gesture: bool,
    pub stepped: bool,
    pub hidden: bool,
    pub read_only: bool,
}

impl PluginParameterFlags {
    pub fn automatable() -> Self {
        Self {
            automatable: true,
            modulatable: true,
            supports_gesture: true,
            stepped: false,
            hidden: false,
            read_only: false,
        }
    }

    pub fn bypass() -> Self {
        Self {
            automatable: true,
            modulatable: false,
            supports_gesture: false,
            stepped: true,
            hidden: false,
            read_only: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PluginParameterDescriptor {
    pub parameter_id: u32,
    pub name: String,
    pub unit: Option<String>,
    pub domain: PluginParameterDomain,
    pub default_normalized: f32,
    pub min_plain: f32,
    pub max_plain: f32,
    pub flags: PluginParameterFlags,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginStateContract {
    pub supports_snapshot: bool,
    pub supports_reset: bool,
    pub supports_bypass: bool,
    pub exposes_latency: bool,
    pub exposes_tail: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginProcessingContract {
    pub max_block_frames: u32,
    pub sample_accurate_automation: bool,
    pub accepts_midi: bool,
    pub accepts_note_events: bool,
    pub supports_note_expression: bool,
    pub produces_midi: bool,
    pub silence_aware: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginLifecycleContract {
    pub requires_main_thread_for_state: bool,
    pub supports_prepare: bool,
    pub supports_activate: bool,
    pub supports_reset_while_active: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginLifecycleState {
    Discovered,
    TypeLoaded,
    InstanceCreated,
    Prepared,
    Active,
    Inactive,
    Released,
    Faulted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginFaultKind {
    InvalidRequest,
    UnsupportedCapability,
    InvalidState,
    ResourceUnavailable,
    ProcessingFailure,
    ProtocolViolation,
    Timeout,
    Crash,
    Fatal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginFaultSeverity {
    Warning,
    Recoverable,
    Critical,
    Fatal,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginFault {
    pub kind: PluginFaultKind,
    pub severity: PluginFaultSeverity,
    pub message: String,
}

impl PluginFault {
    pub fn new(
        kind: PluginFaultKind,
        severity: PluginFaultSeverity,
        message: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            severity,
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginDegradedReason(pub &'static str);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PluginReadiness {
    Starting,
    Ready,
    Degraded { reasons: Vec<PluginDegradedReason> },
    Stopped,
    Failed { fatal: PluginFault },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginProcessConfiguration {
    pub sample_rate_hz: u32,
    pub max_block_frames: u32,
    pub io_layout: PluginIoLayout,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PluginInstanceSnapshot {
    pub plugin_type_id: PluginTypeId,
    pub instance_id: PluginInstanceId,
    pub lifecycle_state: PluginLifecycleState,
    pub readiness: PluginReadiness,
    pub processing: Option<PluginProcessConfiguration>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PluginDescriptor {
    pub plugin_id: String,
    pub vendor: String,
    pub name: String,
    pub format: PluginFormat,
    pub version: Option<String>,
    pub features: Vec<PluginFeature>,
    pub audio_buses: Vec<PluginAudioBusDescriptor>,
    pub parameters: Vec<PluginParameterDescriptor>,
    pub state_contract: PluginStateContract,
    pub processing_contract: PluginProcessingContract,
    pub lifecycle_contract: PluginLifecycleContract,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginIoLayout {
    pub audio_inputs: u16,
    pub audio_outputs: u16,
    pub midi_inputs: u16,
    pub midi_outputs: u16,
}

impl PluginIoLayout {
    pub fn audio_channels(self) -> u16 {
        self.audio_inputs.max(self.audio_outputs)
    }

    pub fn midi_ports(self) -> u16 {
        self.midi_inputs.max(self.midi_outputs)
    }

    pub fn main_audio_buses(self) -> Vec<PluginAudioBusDescriptor> {
        let mut buses = Vec::new();
        if self.audio_inputs > 0 {
            buses.push(PluginAudioBusDescriptor {
                bus_id: "audio:main-in".into(),
                name: "Main Input".into(),
                direction: PluginAudioBusDirection::Input,
                channels: self.audio_inputs,
                is_main: true,
                active: true,
            });
        }
        if self.audio_outputs > 0 {
            buses.push(PluginAudioBusDescriptor {
                bus_id: "audio:main-out".into(),
                name: "Main Output".into(),
                direction: PluginAudioBusDirection::Output,
                channels: self.audio_outputs,
                is_main: true,
                active: true,
            });
        }
        buses
    }
}

impl PluginDescriptor {
    pub fn new(
        plugin_id: impl Into<String>,
        vendor: impl Into<String>,
        name: impl Into<String>,
        format: PluginFormat,
    ) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            vendor: vendor.into(),
            name: name.into(),
            format,
            version: None,
            features: Vec::new(),
            audio_buses: Vec::new(),
            parameters: Vec::new(),
            state_contract: PluginStateContract {
                supports_snapshot: false,
                supports_reset: false,
                supports_bypass: false,
                exposes_latency: false,
                exposes_tail: false,
            },
            processing_contract: PluginProcessingContract {
                max_block_frames: 0,
                sample_accurate_automation: false,
                accepts_midi: false,
                accepts_note_events: false,
                supports_note_expression: false,
                produces_midi: false,
                silence_aware: false,
            },
            lifecycle_contract: PluginLifecycleContract {
                requires_main_thread_for_state: false,
                supports_prepare: true,
                supports_activate: true,
                supports_reset_while_active: false,
            },
        }
    }

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    pub fn with_feature(mut self, feature: PluginFeature) -> Self {
        self.features.push(feature);
        self
    }

    pub fn with_audio_buses(mut self, audio_buses: Vec<PluginAudioBusDescriptor>) -> Self {
        self.audio_buses = audio_buses;
        self
    }

    pub fn with_parameters(mut self, parameters: Vec<PluginParameterDescriptor>) -> Self {
        self.parameters = parameters;
        self
    }

    pub fn with_state_contract(mut self, state_contract: PluginStateContract) -> Self {
        self.state_contract = state_contract;
        self
    }

    pub fn with_processing_contract(
        mut self,
        processing_contract: PluginProcessingContract,
    ) -> Self {
        self.processing_contract = processing_contract;
        self
    }

    pub fn with_lifecycle_contract(mut self, lifecycle_contract: PluginLifecycleContract) -> Self {
        self.lifecycle_contract = lifecycle_contract;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginSandboxRequest {
    pub sandbox_id: String,
    pub format: PluginFormat,
    pub policy: SandboxPolicy,
    pub correlation_id: Option<CorrelationId>,
}

impl PluginSandboxRequest {
    pub fn new(sandbox_id: impl Into<String>, format: PluginFormat, policy: SandboxPolicy) -> Self {
        Self {
            sandbox_id: sandbox_id.into(),
            format,
            policy,
            correlation_id: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SandboxTransport {
    SharedMemory,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginSandboxErrorKind {
    InvalidRequest,
    InvalidState,
    Unsupported,
    Timeout,
    ProtocolViolation,
    Crashed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginSandboxError {
    pub kind: PluginSandboxErrorKind,
    pub message: String,
}

impl PluginSandboxError {
    pub fn new(kind: PluginSandboxErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub fn as_fault(&self) -> PluginFault {
        PluginFault::new(
            PluginFaultKind::from(self.kind),
            PluginFaultSeverity::from(self.kind),
            self.message.clone(),
        )
    }
}

impl From<PluginSandboxErrorKind> for PluginFaultKind {
    fn from(value: PluginSandboxErrorKind) -> Self {
        match value {
            PluginSandboxErrorKind::InvalidRequest => Self::InvalidRequest,
            PluginSandboxErrorKind::InvalidState => Self::InvalidState,
            PluginSandboxErrorKind::Unsupported => Self::UnsupportedCapability,
            PluginSandboxErrorKind::Timeout => Self::Timeout,
            PluginSandboxErrorKind::ProtocolViolation => Self::ProtocolViolation,
            PluginSandboxErrorKind::Crashed => Self::Crash,
        }
    }
}

impl From<PluginSandboxErrorKind> for PluginFaultSeverity {
    fn from(value: PluginSandboxErrorKind) -> Self {
        match value {
            PluginSandboxErrorKind::InvalidRequest
            | PluginSandboxErrorKind::InvalidState
            | PluginSandboxErrorKind::Unsupported => Self::Warning,
            PluginSandboxErrorKind::Timeout => Self::Recoverable,
            PluginSandboxErrorKind::ProtocolViolation => Self::Critical,
            PluginSandboxErrorKind::Crashed => Self::Fatal,
        }
    }
}

impl PluginReadiness {
    pub fn from_fault(fault: PluginFault) -> Self {
        match fault.severity {
            PluginFaultSeverity::Warning | PluginFaultSeverity::Recoverable => Self::Degraded {
                reasons: vec![PluginDegradedReason("plugin-fault")],
            },
            PluginFaultSeverity::Critical | PluginFaultSeverity::Fatal => {
                Self::Failed { fatal: fault }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginSandboxCapabilities {
    pub transport: SandboxTransport,
    pub supports_state: bool,
    pub supports_midi: bool,
    pub max_block_frames: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SandboxControlCommand {
    Handshake,
    LoadPluginType {
        plugin_type_id: PluginTypeId,
        descriptor: PluginDescriptor,
    },
    CreateInstance {
        plugin_type_id: PluginTypeId,
        instance_id: PluginInstanceId,
    },
    DestroyInstance {
        instance_id: PluginInstanceId,
    },
    PrepareInstance {
        instance_id: PluginInstanceId,
        sample_rate_hz: u32,
        max_block_frames: u32,
        io_layout: PluginIoLayout,
        shared_memory: SharedMemoryLayout,
    },
    ActivateInstance {
        instance_id: PluginInstanceId,
        processing_epoch: u64,
    },
    DeactivateInstance {
        instance_id: PluginInstanceId,
    },
    ResetInstance {
        instance_id: PluginInstanceId,
        processing_epoch: u64,
    },
    Shutdown,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SandboxControlRequest {
    pub sandbox_id: String,
    pub format: PluginFormat,
    pub correlation_id: Option<CorrelationId>,
    pub command: SandboxControlCommand,
}

impl SandboxControlRequest {
    pub fn handshake(sandbox_id: impl Into<String>, format: PluginFormat) -> Self {
        Self {
            sandbox_id: sandbox_id.into(),
            format,
            correlation_id: None,
            command: SandboxControlCommand::Handshake,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SandboxControlResponse {
    HandshakeAccepted {
        protocol_version: u32,
        capabilities: PluginSandboxCapabilities,
    },
    PluginTypeLoaded {
        plugin_type_id: PluginTypeId,
    },
    InstanceCreated {
        instance_id: PluginInstanceId,
    },
    InstancePrepared {
        instance_id: PluginInstanceId,
        processing_epoch: u64,
    },
    InstanceActivated {
        instance_id: PluginInstanceId,
        processing_epoch: u64,
    },
    InstanceDeactivated {
        instance_id: PluginInstanceId,
    },
    InstanceReset {
        instance_id: PluginInstanceId,
        processing_epoch: u64,
    },
    InstanceDestroyed {
        instance_id: PluginInstanceId,
    },
    ShutdownAccepted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SandboxWatchdogPolicy {
    pub max_consecutive_deadline_misses: u32,
    pub max_consecutive_heartbeat_misses: u32,
}

impl Default for SandboxWatchdogPolicy {
    fn default() -> Self {
        Self {
            max_consecutive_deadline_misses: 2,
            max_consecutive_heartbeat_misses: 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WatchdogTriggerReason {
    DeadlineMisses,
    HeartbeatMisses,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WatchdogOutcome {
    Healthy,
    RestartRequired {
        reason: WatchdogTriggerReason,
        consecutive_misses: u32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SandboxWatchdogState {
    policy: SandboxWatchdogPolicy,
    consecutive_deadline_misses: u32,
    consecutive_heartbeat_misses: u32,
}

impl SandboxWatchdogState {
    pub fn new(policy: SandboxWatchdogPolicy) -> Self {
        Self {
            policy,
            consecutive_deadline_misses: 0,
            consecutive_heartbeat_misses: 0,
        }
    }

    pub fn policy(&self) -> SandboxWatchdogPolicy {
        self.policy
    }

    pub fn consecutive_deadline_misses(&self) -> u32 {
        self.consecutive_deadline_misses
    }

    pub fn consecutive_heartbeat_misses(&self) -> u32 {
        self.consecutive_heartbeat_misses
    }

    pub fn record_heartbeat_response(&mut self) {
        self.consecutive_heartbeat_misses = 0;
    }

    pub fn record_heartbeat_miss(&mut self) -> WatchdogOutcome {
        self.consecutive_heartbeat_misses = self.consecutive_heartbeat_misses.saturating_add(1);
        if self.consecutive_heartbeat_misses >= self.policy.max_consecutive_heartbeat_misses {
            return WatchdogOutcome::RestartRequired {
                reason: WatchdogTriggerReason::HeartbeatMisses,
                consecutive_misses: self.consecutive_heartbeat_misses,
            };
        }
        WatchdogOutcome::Healthy
    }

    pub fn record_block_completion(&mut self, state: CompletionState) -> WatchdogOutcome {
        match state {
            CompletionState::TimedOut => {
                self.consecutive_deadline_misses =
                    self.consecutive_deadline_misses.saturating_add(1);
                if self.consecutive_deadline_misses >= self.policy.max_consecutive_deadline_misses {
                    return WatchdogOutcome::RestartRequired {
                        reason: WatchdogTriggerReason::DeadlineMisses,
                        consecutive_misses: self.consecutive_deadline_misses,
                    };
                }
            }
            CompletionState::Completed => {
                self.consecutive_deadline_misses = 0;
            }
            _ => {}
        }
        WatchdogOutcome::Healthy
    }

    pub fn reset(&mut self) {
        self.consecutive_deadline_misses = 0;
        self.consecutive_heartbeat_misses = 0;
    }
}

impl Default for SandboxWatchdogState {
    fn default() -> Self {
        Self::new(SandboxWatchdogPolicy::default())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RestartEscalationPolicy {
    pub safe_mode_restart_threshold: u32,
}

impl Default for RestartEscalationPolicy {
    fn default() -> Self {
        Self {
            safe_mode_restart_threshold: 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RestartEscalationState {
    policy: RestartEscalationPolicy,
    watchdog_restart_count: u32,
    safe_mode_requested: bool,
}

impl RestartEscalationState {
    pub fn new(policy: RestartEscalationPolicy) -> Self {
        Self {
            policy,
            watchdog_restart_count: 0,
            safe_mode_requested: false,
        }
    }

    pub fn watchdog_restart_count(&self) -> u32 {
        self.watchdog_restart_count
    }

    pub fn safe_mode_requested(&self) -> bool {
        self.safe_mode_requested
    }

    pub fn record_watchdog_restart(&mut self) -> bool {
        self.watchdog_restart_count = self.watchdog_restart_count.saturating_add(1);
        if self.watchdog_restart_count >= self.policy.safe_mode_restart_threshold {
            self.safe_mode_requested = true;
        }
        self.safe_mode_requested
    }
}

impl Default for RestartEscalationState {
    fn default() -> Self {
        Self::new(RestartEscalationPolicy::default())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoopRange {
    pub start_samples: i64,
    pub end_samples: i64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PluginRenderContext {
    pub sample_rate_hz: u32,
    pub tempo_bpm: f64,
    pub timeline_position_samples: i64,
    pub playing: bool,
    pub bypassed: bool,
    pub loop_range: Option<LoopRange>,
    pub deadline_frames: u32,
}

impl PluginRenderContext {
    pub(crate) const ENCODED_BYTES: usize = 48;
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
