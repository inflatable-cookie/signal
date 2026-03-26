//! Format-neutral plugin host abstractions for Signal.

use signal_ipc::CorrelationId;

mod plugin_block_transport;
mod plugin_event_reports;

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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParameterValueEvent {
    pub offset_frames: u32,
    pub parameter_id: u32,
    pub normalized_value: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParameterModulationEvent {
    pub offset_frames: u32,
    pub parameter_id: u32,
    pub amount: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParameterGesturePhase {
    Begin,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParameterGestureEvent {
    pub offset_frames: u32,
    pub parameter_id: u32,
    pub phase: ParameterGesturePhase,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MidiEvent {
    pub offset_frames: u32,
    pub status: u8,
    pub data1: u8,
    pub data2: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoteEventKind {
    NoteOn,
    NoteOff,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoteExpressionKind {
    Pressure,
    Timbre,
    Tuning,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NoteEvent {
    pub offset_frames: u32,
    pub note_id: i32,
    pub port_index: u16,
    pub channel: u8,
    pub key: u8,
    pub velocity: f32,
    pub kind: NoteEventKind,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NoteExpressionEvent {
    pub offset_frames: u32,
    pub note_id: i32,
    pub port_index: u16,
    pub channel: u8,
    pub key: u8,
    pub expression: NoteExpressionKind,
    pub value: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PluginEvent {
    ParameterValue(ParameterValueEvent),
    ParameterModulation(ParameterModulationEvent),
    ParameterGesture(ParameterGestureEvent),
    Note(NoteEvent),
    NoteExpression(NoteExpressionEvent),
    Midi(MidiEvent),
}

impl PluginEvent {
    const ENCODED_BYTES: usize = 24;

    pub fn is_parameter_value(self) -> bool {
        matches!(self, Self::ParameterValue(_))
    }

    pub fn is_parameter_modulation(self) -> bool {
        matches!(self, Self::ParameterModulation(_))
    }

    pub fn is_parameter_gesture(self) -> bool {
        matches!(self, Self::ParameterGesture(_))
    }

    pub fn is_note(self) -> bool {
        matches!(self, Self::Note(_))
    }

    pub fn is_note_expression(self) -> bool {
        matches!(self, Self::NoteExpression(_))
    }

    pub fn is_midi(self) -> bool {
        matches!(self, Self::Midi(_))
    }

    fn write_to_slice(&self, bytes: &mut [u8]) -> Result<(), &'static str> {
        if bytes.len() < Self::ENCODED_BYTES {
            return Err("event region entry is too small for encoded event");
        }

        bytes[..Self::ENCODED_BYTES].fill(0);
        match self {
            Self::ParameterValue(event) => {
                bytes[0] = 1;
                bytes[4..8].copy_from_slice(&event.offset_frames.to_le_bytes());
                bytes[8..12].copy_from_slice(&event.parameter_id.to_le_bytes());
                bytes[12..16].copy_from_slice(&event.normalized_value.to_le_bytes());
            }
            Self::ParameterModulation(event) => {
                bytes[0] = 2;
                bytes[4..8].copy_from_slice(&event.offset_frames.to_le_bytes());
                bytes[8..12].copy_from_slice(&event.parameter_id.to_le_bytes());
                bytes[12..16].copy_from_slice(&event.amount.to_le_bytes());
            }
            Self::ParameterGesture(event) => {
                bytes[0] = 3;
                bytes[1] = match event.phase {
                    ParameterGesturePhase::Begin => 1,
                    ParameterGesturePhase::End => 2,
                };
                bytes[4..8].copy_from_slice(&event.offset_frames.to_le_bytes());
                bytes[8..12].copy_from_slice(&event.parameter_id.to_le_bytes());
            }
            Self::Note(event) => {
                bytes[0] = 4;
                bytes[1] = match event.kind {
                    NoteEventKind::NoteOn => 1,
                    NoteEventKind::NoteOff => 2,
                };
                bytes[4..8].copy_from_slice(&event.offset_frames.to_le_bytes());
                bytes[8..12].copy_from_slice(&event.note_id.to_le_bytes());
                bytes[12..14].copy_from_slice(&event.port_index.to_le_bytes());
                bytes[14] = event.channel;
                bytes[15] = event.key;
                bytes[16..20].copy_from_slice(&event.velocity.to_le_bytes());
            }
            Self::NoteExpression(event) => {
                bytes[0] = 5;
                bytes[1] = match event.expression {
                    NoteExpressionKind::Pressure => 1,
                    NoteExpressionKind::Timbre => 2,
                    NoteExpressionKind::Tuning => 3,
                };
                bytes[4..8].copy_from_slice(&event.offset_frames.to_le_bytes());
                bytes[8..12].copy_from_slice(&event.note_id.to_le_bytes());
                bytes[12..14].copy_from_slice(&event.port_index.to_le_bytes());
                bytes[14] = event.channel;
                bytes[15] = event.key;
                bytes[16..20].copy_from_slice(&event.value.to_le_bytes());
            }
            Self::Midi(event) => {
                bytes[0] = 6;
                bytes[4..8].copy_from_slice(&event.offset_frames.to_le_bytes());
                bytes[8] = event.status;
                bytes[9] = event.data1;
                bytes[10] = event.data2;
            }
        }
        Ok(())
    }

    fn read_from_slice(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < Self::ENCODED_BYTES {
            return Err("event region entry is too small for encoded event");
        }

        match bytes[0] {
            1 => Ok(Self::ParameterValue(ParameterValueEvent {
                offset_frames: u32::from_le_bytes(
                    bytes[4..8]
                        .try_into()
                        .map_err(|_| "parameter event offset decode")?,
                ),
                parameter_id: u32::from_le_bytes(
                    bytes[8..12]
                        .try_into()
                        .map_err(|_| "parameter event id decode")?,
                ),
                normalized_value: f32::from_le_bytes(
                    bytes[12..16]
                        .try_into()
                        .map_err(|_| "parameter event value decode")?,
                ),
            })),
            2 => Ok(Self::ParameterModulation(ParameterModulationEvent {
                offset_frames: u32::from_le_bytes(
                    bytes[4..8]
                        .try_into()
                        .map_err(|_| "parameter modulation offset decode")?,
                ),
                parameter_id: u32::from_le_bytes(
                    bytes[8..12]
                        .try_into()
                        .map_err(|_| "parameter modulation id decode")?,
                ),
                amount: f32::from_le_bytes(
                    bytes[12..16]
                        .try_into()
                        .map_err(|_| "parameter modulation amount decode")?,
                ),
            })),
            3 => Ok(Self::ParameterGesture(ParameterGestureEvent {
                offset_frames: u32::from_le_bytes(
                    bytes[4..8]
                        .try_into()
                        .map_err(|_| "parameter gesture offset decode")?,
                ),
                parameter_id: u32::from_le_bytes(
                    bytes[8..12]
                        .try_into()
                        .map_err(|_| "parameter gesture id decode")?,
                ),
                phase: match bytes[1] {
                    1 => ParameterGesturePhase::Begin,
                    2 => ParameterGesturePhase::End,
                    _ => return Err("unknown parameter gesture phase"),
                },
            })),
            4 => Ok(Self::Note(NoteEvent {
                offset_frames: u32::from_le_bytes(
                    bytes[4..8]
                        .try_into()
                        .map_err(|_| "note event offset decode")?,
                ),
                note_id: i32::from_le_bytes(
                    bytes[8..12]
                        .try_into()
                        .map_err(|_| "note event id decode")?,
                ),
                port_index: u16::from_le_bytes(
                    bytes[12..14]
                        .try_into()
                        .map_err(|_| "note event port decode")?,
                ),
                channel: bytes[14],
                key: bytes[15],
                velocity: f32::from_le_bytes(
                    bytes[16..20]
                        .try_into()
                        .map_err(|_| "note event velocity decode")?,
                ),
                kind: match bytes[1] {
                    1 => NoteEventKind::NoteOn,
                    2 => NoteEventKind::NoteOff,
                    _ => return Err("unknown note event kind"),
                },
            })),
            5 => Ok(Self::NoteExpression(NoteExpressionEvent {
                offset_frames: u32::from_le_bytes(
                    bytes[4..8]
                        .try_into()
                        .map_err(|_| "note expression offset decode")?,
                ),
                note_id: i32::from_le_bytes(
                    bytes[8..12]
                        .try_into()
                        .map_err(|_| "note expression id decode")?,
                ),
                port_index: u16::from_le_bytes(
                    bytes[12..14]
                        .try_into()
                        .map_err(|_| "note expression port decode")?,
                ),
                channel: bytes[14],
                key: bytes[15],
                expression: match bytes[1] {
                    1 => NoteExpressionKind::Pressure,
                    2 => NoteExpressionKind::Timbre,
                    3 => NoteExpressionKind::Tuning,
                    _ => return Err("unknown note expression kind"),
                },
                value: f32::from_le_bytes(
                    bytes[16..20]
                        .try_into()
                        .map_err(|_| "note expression value decode")?,
                ),
            })),
            6 => Ok(Self::Midi(MidiEvent {
                offset_frames: u32::from_le_bytes(
                    bytes[4..8]
                        .try_into()
                        .map_err(|_| "midi event offset decode")?,
                ),
                status: bytes[8],
                data1: bytes[9],
                data2: bytes[10],
            })),
            _ => Err("unknown plugin event type"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AudioBlock {
    pub channel_count: u16,
    pub frame_count: u32,
    pub samples: Vec<f32>,
}

impl AudioBlock {
    pub fn new(
        channel_count: u16,
        frame_count: u32,
        samples: Vec<f32>,
    ) -> Result<Self, &'static str> {
        if samples.len() != channel_count as usize * frame_count as usize {
            return Err("audio block samples do not match channel_count * frame_count");
        }

        Ok(Self {
            channel_count,
            frame_count,
            samples,
        })
    }

    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    pub fn first_sample(&self) -> Option<f32> {
        self.samples.first().copied()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EventPacket {
    pub events: Vec<PluginEvent>,
}

impl EventPacket {
    pub fn new(events: Vec<PluginEvent>) -> Self {
        Self { events }
    }

    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    pub fn encoded_bytes(&self) -> u32 {
        (4 + self.events.len() * PluginEvent::ENCODED_BYTES) as u32
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BlockPayload {
    pub audio: AudioBlock,
    pub events: EventPacket,
}

impl BlockPayload {
    pub fn new(audio: AudioBlock, events: EventPacket) -> Self {
        Self { audio, events }
    }
}

impl PluginRenderContext {
    pub const ENCODED_BYTES: usize = 48;

    pub fn write_to_slice(&self, bytes: &mut [u8]) -> Result<(), &'static str> {
        if bytes.len() < Self::ENCODED_BYTES {
            return Err("render-context region is too small for encoded context");
        }

        bytes[0..4].copy_from_slice(&self.sample_rate_hz.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.deadline_frames.to_le_bytes());
        bytes[8..16].copy_from_slice(&self.tempo_bpm.to_le_bytes());
        bytes[16..24].copy_from_slice(&self.timeline_position_samples.to_le_bytes());
        bytes[24] = u8::from(self.playing);
        bytes[25] = u8::from(self.bypassed);
        bytes[26] = u8::from(self.loop_range.is_some());
        bytes[27] = 0;

        let (loop_start, loop_end) = self
            .loop_range
            .map(|range| (range.start_samples, range.end_samples))
            .unwrap_or_default();
        bytes[28..36].copy_from_slice(&loop_start.to_le_bytes());
        bytes[36..44].copy_from_slice(&loop_end.to_le_bytes());
        bytes[44..48].fill(0);
        Ok(())
    }

    pub fn read_from_slice(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < Self::ENCODED_BYTES {
            return Err("render-context region is too small for encoded context");
        }

        let sample_rate_hz =
            u32::from_le_bytes(bytes[0..4].try_into().map_err(|_| "sample_rate decode")?);
        let deadline_frames =
            u32::from_le_bytes(bytes[4..8].try_into().map_err(|_| "deadline decode")?);
        let tempo_bpm = f64::from_le_bytes(bytes[8..16].try_into().map_err(|_| "tempo decode")?);
        let timeline_position_samples =
            i64::from_le_bytes(bytes[16..24].try_into().map_err(|_| "timeline decode")?);
        let has_loop = bytes[26] != 0;
        let loop_start =
            i64::from_le_bytes(bytes[28..36].try_into().map_err(|_| "loop start decode")?);
        let loop_end = i64::from_le_bytes(bytes[36..44].try_into().map_err(|_| "loop end decode")?);

        Ok(Self {
            sample_rate_hz,
            tempo_bpm,
            timeline_position_samples,
            playing: bytes[24] != 0,
            bypassed: bytes[25] != 0,
            loop_range: has_loop.then_some(LoopRange {
                start_samples: loop_start,
                end_samples: loop_end,
            }),
            deadline_frames,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AudioBlock, AutomationContinuityReport, BlockDispatch, BlockPayload, BlockProcessResult,
        BlockSequenceContinuityReport, CompletionSlot, CompletionState, EventPacket, LoopRange,
        MidiEvent, NoteEvent, NoteEventKind, NoteExpressionEvent, NoteExpressionKind,
        ParameterAutomationSummary, ParameterGestureEvent, ParameterGesturePhase,
        ParameterModulationEvent, ParameterValueEvent, PluginDescriptor, PluginEvent,
        PluginFaultKind, PluginFaultSeverity, PluginFormat, PluginInstanceId, PluginIoLayout,
        PluginLifecycleState, PluginParameterDomain, PluginParameterFlags, PluginReadiness,
        PluginRenderContext, PluginSandboxCapabilities, PluginSandboxError, PluginSandboxErrorKind,
        RestartEscalationPolicy, RestartEscalationState, SandboxControlRequest,
        SandboxControlResponse, SandboxStateMachine, SandboxTransport, SandboxWatchdogPolicy,
        SandboxWatchdogState, SharedMemoryLayout, SharedMemoryLease, WatchdogOutcome,
        WatchdogTriggerReason,
    };
    use signal_ipc::{SharedMemoryTransportKind, SharedMemoryTransportPayload};

    fn test_render_context() -> PluginRenderContext {
        PluginRenderContext {
            sample_rate_hz: 48_000,
            tempo_bpm: 120.0,
            timeline_position_samples: 0,
            playing: true,
            bypassed: false,
            loop_range: Some(LoopRange {
                start_samples: 0,
                end_samples: 96_000,
            }),
            deadline_frames: 512,
        }
    }

    fn test_payload(dispatch: &BlockDispatch) -> BlockPayload {
        let sample_count =
            dispatch.header.channel_count as usize * dispatch.header.frame_count as usize;
        let audio = AudioBlock::new(
            dispatch.header.channel_count,
            dispatch.header.frame_count,
            (0..sample_count).map(|index| index as f32).collect(),
        )
        .expect("audio block");
        let events = EventPacket::new(vec![
            PluginEvent::ParameterValue(ParameterValueEvent {
                offset_frames: 32,
                parameter_id: 7,
                normalized_value: 0.5,
            }),
            PluginEvent::ParameterGesture(ParameterGestureEvent {
                offset_frames: 40,
                parameter_id: 7,
                phase: ParameterGesturePhase::Begin,
            }),
            PluginEvent::ParameterGesture(ParameterGestureEvent {
                offset_frames: 48,
                parameter_id: 7,
                phase: ParameterGesturePhase::End,
            }),
            PluginEvent::NoteExpression(NoteExpressionEvent {
                offset_frames: 56,
                note_id: 7,
                port_index: 0,
                channel: 0,
                key: 60,
                expression: NoteExpressionKind::Pressure,
                value: 0.6,
            }),
            PluginEvent::Midi(MidiEvent {
                offset_frames: 64,
                status: 0x90,
                data1: 60,
                data2: 96,
            }),
        ]);
        BlockPayload::new(audio, events)
    }

    #[test]
    fn shared_memory_layout_regions_do_not_overlap() {
        let layout = SharedMemoryLayout::single_block(2048, 512);
        assert!(layout.audio_output.offset_bytes >= layout.audio_input.size_bytes);
        assert!(layout.completion.offset_bytes > layout.render_context.offset_bytes);
        assert_eq!(layout.total_bytes(), layout.completion.offset_bytes + 64);
    }

    #[test]
    fn sandbox_state_machine_advances_through_processing_states() {
        let mut machine = SandboxStateMachine::new();
        let dispatch = BlockDispatch::new(
            PluginInstanceId("instance-1".into()),
            7,
            3,
            512,
            PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 1,
            },
            test_render_context(),
            1024,
        );

        machine.begin_block(&dispatch);
        assert_eq!(machine.slot().state, CompletionState::ReadyForProcessing);

        assert!(machine.mark_processing());
        assert_eq!(machine.slot().state, CompletionState::Processing);

        assert!(machine.mark_completed(7, 3));
        assert_eq!(machine.slot().state, CompletionState::Completed);
    }

    #[test]
    fn completion_rejects_mismatched_epoch_or_sequence() {
        let mut machine = SandboxStateMachine::new();
        let dispatch = BlockDispatch::new(
            PluginInstanceId("instance-2".into()),
            5,
            11,
            256,
            PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 0,
            },
            test_render_context(),
            512,
        );

        machine.begin_block(&dispatch);
        machine.mark_processing();

        assert!(!machine.mark_completed(4, 11));
        assert_eq!(machine.slot().state, CompletionState::Processing);
    }

    #[test]
    fn handshake_request_and_response_capture_protocol_defaults() {
        let request = SandboxControlRequest::handshake("sandbox-a", PluginFormat::Clap);
        let response = SandboxControlResponse::HandshakeAccepted {
            protocol_version: 1,
            capabilities: PluginSandboxCapabilities {
                transport: SandboxTransport::SharedMemory,
                supports_state: true,
                supports_midi: true,
                max_block_frames: 2048,
            },
        };

        assert_eq!(request.sandbox_id, "sandbox-a");
        assert_eq!(request.format, PluginFormat::Clap);
        assert!(matches!(
            response,
            SandboxControlResponse::HandshakeAccepted { .. }
        ));
    }

    #[test]
    fn plugin_descriptor_carries_neutral_contract_metadata() {
        let descriptor =
            PluginDescriptor::new("plugin:test", "Signal", "Test Plugin", PluginFormat::Clap)
                .with_version("1.2.3")
                .with_feature(super::PluginFeature::AudioEffect)
                .with_audio_buses(
                    PluginIoLayout {
                        audio_inputs: 2,
                        audio_outputs: 2,
                        midi_inputs: 1,
                        midi_outputs: 0,
                    }
                    .main_audio_buses(),
                )
                .with_parameters(vec![super::PluginParameterDescriptor {
                    parameter_id: 9,
                    name: "Cutoff".into(),
                    unit: Some("Hz".into()),
                    domain: PluginParameterDomain::Hertz,
                    default_normalized: 0.5,
                    min_plain: 20.0,
                    max_plain: 20_000.0,
                    flags: PluginParameterFlags::automatable(),
                }])
                .with_state_contract(super::PluginStateContract {
                    supports_snapshot: true,
                    supports_reset: true,
                    supports_bypass: true,
                    exposes_latency: false,
                    exposes_tail: false,
                })
                .with_processing_contract(super::PluginProcessingContract {
                    max_block_frames: 2048,
                    sample_accurate_automation: true,
                    accepts_midi: true,
                    accepts_note_events: true,
                    supports_note_expression: true,
                    produces_midi: false,
                    silence_aware: true,
                })
                .with_lifecycle_contract(super::PluginLifecycleContract {
                    requires_main_thread_for_state: true,
                    supports_prepare: true,
                    supports_activate: true,
                    supports_reset_while_active: false,
                });

        assert_eq!(descriptor.version.as_deref(), Some("1.2.3"));
        assert_eq!(descriptor.audio_buses.len(), 2);
        assert_eq!(descriptor.parameters.len(), 1);
        assert!(descriptor.state_contract.supports_snapshot);
        assert!(descriptor.processing_contract.sample_accurate_automation);
        assert!(descriptor.lifecycle_contract.requires_main_thread_for_state);
    }

    #[test]
    fn plugin_sandbox_errors_map_into_plugin_fault_readiness_taxonomy() {
        let protocol_error = PluginSandboxError::new(
            PluginSandboxErrorKind::ProtocolViolation,
            "sandbox protocol mismatch",
        )
        .as_fault();
        let crash_error =
            PluginSandboxError::new(PluginSandboxErrorKind::Crashed, "sandbox process exited")
                .as_fault();

        assert_eq!(protocol_error.kind, PluginFaultKind::ProtocolViolation);
        assert_eq!(protocol_error.severity, PluginFaultSeverity::Critical);
        assert!(matches!(
            PluginReadiness::from_fault(protocol_error),
            PluginReadiness::Failed { .. }
        ));

        assert_eq!(crash_error.kind, PluginFaultKind::Crash);
        assert_eq!(crash_error.severity, PluginFaultSeverity::Fatal);

        let snapshot = super::PluginInstanceSnapshot {
            plugin_type_id: super::PluginTypeId("plugin:test".into()),
            instance_id: PluginInstanceId("instance:test".into()),
            lifecycle_state: PluginLifecycleState::Prepared,
            readiness: PluginReadiness::Starting,
            processing: Some(super::PluginProcessConfiguration {
                sample_rate_hz: 48_000,
                max_block_frames: 512,
                io_layout: PluginIoLayout {
                    audio_inputs: 2,
                    audio_outputs: 2,
                    midi_inputs: 1,
                    midi_outputs: 0,
                },
            }),
        };
        assert_eq!(snapshot.lifecycle_state, PluginLifecycleState::Prepared);
    }

    #[test]
    fn shared_memory_lease_tracks_epoch_invalidations() {
        let layout = SharedMemoryLayout::single_block(2048, 512);
        let mut lease = SharedMemoryLease::new("lease-a", 3, layout);

        assert_eq!(lease.total_bytes(), layout.total_bytes());
        assert!(lease.is_epoch_valid(3));
        assert!(lease.invalidate_epoch(3));
        assert!(!lease.is_epoch_valid(3));
        assert_eq!(lease.invalidated_epochs(), &[3]);
    }

    #[test]
    fn shared_memory_lease_binds_transport_metadata() {
        let lease = SharedMemoryLease::new("lease-a", 4, SharedMemoryLayout::single_block(256, 64))
            .with_transport(SharedMemoryTransportPayload {
                region_id: "region-a".into(),
                transport_kind: SharedMemoryTransportKind::MappedFile,
                backing_path: "/tmp/region-a.signal-shm".into(),
                total_bytes: 320,
            });

        let transport = lease.transport().expect("transport binding");
        assert_eq!(transport.region_id, "region-a");
        assert_eq!(transport.total_bytes, 320);
    }

    #[test]
    fn block_dispatch_round_trips_through_shared_memory_regions() {
        let dispatch = BlockDispatch::new(
            PluginInstanceId("instance-dispatch".into()),
            5,
            9,
            256,
            PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 0,
            },
            test_render_context(),
            512,
        );
        let mut bytes = vec![0; dispatch.layout.total_bytes() as usize];

        dispatch
            .write_to_shared_memory(&mut bytes)
            .expect("write dispatch");
        let decoded = BlockDispatch::read_from_shared_memory(
            PluginInstanceId("instance-dispatch".into()),
            dispatch.io_layout,
            dispatch.layout,
            &bytes,
        )
        .expect("decode dispatch");

        assert_eq!(decoded.header, dispatch.header);
        assert_eq!(decoded.render_context, dispatch.render_context);
    }

    #[test]
    fn block_process_result_round_trips_through_completion_region() {
        let dispatch = BlockDispatch::new(
            PluginInstanceId("instance-result".into()),
            3,
            4,
            128,
            PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 0,
                midi_outputs: 0,
            },
            test_render_context(),
            256,
        );
        let mut bytes = vec![0; dispatch.layout.total_bytes() as usize];
        let result = BlockProcessResult {
            slot: CompletionSlot {
                state: CompletionState::Completed,
                processing_epoch: 3,
                block_sequence: 4,
            },
            generated_event_bytes: 64,
            fallback_applied: false,
        };

        result
            .write_to_shared_memory(dispatch.layout, &mut bytes)
            .expect("write result");
        let decoded = BlockProcessResult::read_from_shared_memory(dispatch.layout, &bytes)
            .expect("decode result");

        assert_eq!(decoded, result);
    }

    #[test]
    fn block_payload_round_trips_through_audio_and_event_regions() {
        let dispatch = BlockDispatch::new(
            PluginInstanceId("instance-payload".into()),
            11,
            6,
            128,
            PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 1,
            },
            test_render_context(),
            256,
        );
        let payload = test_payload(&dispatch);
        let mut bytes = vec![0; dispatch.layout.total_bytes() as usize];

        dispatch
            .write_input_payload(&mut bytes, &payload)
            .expect("write input payload");
        let decoded_input = dispatch
            .read_input_payload(&bytes)
            .expect("decode input payload");
        assert_eq!(decoded_input, payload);

        dispatch
            .write_output_payload(&mut bytes, &payload)
            .expect("write output payload");
        let decoded_output = dispatch
            .read_output_payload(&bytes)
            .expect("decode output payload");
        assert_eq!(decoded_output, payload);
    }

    #[test]
    fn event_packet_summary_counts_richer_event_types() {
        let packet = EventPacket::new(vec![
            PluginEvent::ParameterValue(ParameterValueEvent {
                offset_frames: 0,
                parameter_id: 3,
                normalized_value: 0.1,
            }),
            PluginEvent::ParameterGesture(ParameterGestureEvent {
                offset_frames: 4,
                parameter_id: 3,
                phase: ParameterGesturePhase::Begin,
            }),
            PluginEvent::ParameterModulation(ParameterModulationEvent {
                offset_frames: 8,
                parameter_id: 9,
                amount: -0.2,
            }),
            PluginEvent::Note(NoteEvent {
                offset_frames: 16,
                note_id: 7,
                port_index: 0,
                channel: 0,
                key: 60,
                velocity: 0.8,
                kind: NoteEventKind::NoteOn,
            }),
            PluginEvent::NoteExpression(NoteExpressionEvent {
                offset_frames: 24,
                note_id: 7,
                port_index: 0,
                channel: 0,
                key: 60,
                expression: NoteExpressionKind::Pressure,
                value: 0.7,
            }),
            PluginEvent::NoteExpression(NoteExpressionEvent {
                offset_frames: 28,
                note_id: 7,
                port_index: 0,
                channel: 1,
                key: 61,
                expression: NoteExpressionKind::Timbre,
                value: 0.5,
            }),
            PluginEvent::NoteExpression(NoteExpressionEvent {
                offset_frames: 30,
                note_id: 7,
                port_index: 0,
                channel: 2,
                key: 62,
                expression: NoteExpressionKind::Tuning,
                value: 0.2,
            }),
            PluginEvent::Midi(MidiEvent {
                offset_frames: 32,
                status: 0xB0,
                data1: 1,
                data2: 100,
            }),
        ]);

        let summary = packet.summary();
        assert_eq!(summary.total_events, 8);
        assert_eq!(summary.parameter_value_events, 1);
        assert_eq!(summary.parameter_gesture_events, 1);
        assert_eq!(summary.parameter_modulation_events, 1);
        assert_eq!(summary.note_events, 1);
        assert_eq!(summary.note_expression_events, 3);
        assert_eq!(summary.note_expression_pressure_events, 1);
        assert_eq!(summary.note_expression_timbre_events, 1);
        assert_eq!(summary.note_expression_tuning_events, 1);
        assert_eq!(summary.midi_events, 1);
    }

    #[test]
    fn parameter_automation_summary_tracks_values_modulation_and_gestures() {
        let packet = EventPacket::new(vec![
            PluginEvent::ParameterGesture(ParameterGestureEvent {
                offset_frames: 0,
                parameter_id: 77,
                phase: ParameterGesturePhase::Begin,
            }),
            PluginEvent::ParameterValue(ParameterValueEvent {
                offset_frames: 4,
                parameter_id: 77,
                normalized_value: 0.2,
            }),
            PluginEvent::ParameterModulation(ParameterModulationEvent {
                offset_frames: 8,
                parameter_id: 77,
                amount: -0.1,
            }),
            PluginEvent::ParameterValue(ParameterValueEvent {
                offset_frames: 16,
                parameter_id: 77,
                normalized_value: 0.6,
            }),
            PluginEvent::ParameterGesture(ParameterGestureEvent {
                offset_frames: 20,
                parameter_id: 77,
                phase: ParameterGesturePhase::End,
            }),
            PluginEvent::ParameterValue(ParameterValueEvent {
                offset_frames: 24,
                parameter_id: 9,
                normalized_value: 0.9,
            }),
        ]);

        let summary = packet.parameter_automation_summary(77);
        assert_eq!(
            summary,
            ParameterAutomationSummary {
                parameter_id: 77,
                value_events: 2,
                modulation_events: 1,
                gesture_begin_events: 1,
                gesture_end_events: 1,
                first_value: Some(0.2),
                last_value: Some(0.6),
                last_modulation: Some(-0.1),
            }
        );
    }

    #[test]
    fn automation_continuity_report_tracks_segments_and_lease_rollovers() {
        let mut report = AutomationContinuityReport::default();
        report.record(
            2,
            "lease-a",
            ParameterAutomationSummary {
                parameter_id: 77,
                value_events: 1,
                modulation_events: 1,
                gesture_begin_events: 1,
                gesture_end_events: 0,
                first_value: Some(0.1),
                last_value: Some(0.1),
                last_modulation: Some(0.02),
            },
        );
        report.record(
            2,
            "lease-a",
            ParameterAutomationSummary {
                parameter_id: 77,
                value_events: 1,
                modulation_events: 1,
                gesture_begin_events: 0,
                gesture_end_events: 1,
                first_value: Some(0.15),
                last_value: Some(0.15),
                last_modulation: Some(0.04),
            },
        );
        report.record(
            3,
            "lease-b",
            ParameterAutomationSummary {
                parameter_id: 77,
                value_events: 2,
                modulation_events: 2,
                gesture_begin_events: 1,
                gesture_end_events: 1,
                first_value: Some(0.2),
                last_value: Some(0.25),
                last_modulation: Some(0.06),
            },
        );

        assert_eq!(report.parameter_id, 77);
        assert_eq!(report.segment_count(), 2);
        assert_eq!(report.lease_rollovers, 1);
        assert_eq!(report.first_epoch(), Some(2));
        assert_eq!(report.last_epoch(), Some(3));
        assert_eq!(report.segment_epochs(), vec![2, 3]);

        let aggregate = report.aggregate();
        assert_eq!(aggregate.value_events, 4);
        assert_eq!(aggregate.modulation_events, 4);
        assert_eq!(aggregate.gesture_begin_events, 2);
        assert_eq!(aggregate.gesture_end_events, 2);
        assert_eq!(aggregate.first_value, Some(0.1));
        assert_eq!(aggregate.last_value, Some(0.25));
        assert_eq!(aggregate.last_modulation, Some(0.06));
    }

    #[test]
    fn block_sequence_continuity_report_tracks_rollovers_and_gaps() {
        let mut report = BlockSequenceContinuityReport::default();
        report.record(2, "lease-a", 0);
        report.record(2, "lease-a", 1);
        report.record(2, "lease-a", 3);
        report.record(3, "lease-b", 4);
        report.record(3, "lease-b", 5);

        assert_eq!(report.segment_count(), 3);
        assert_eq!(report.segment_epochs(), vec![2, 2, 3]);
        assert_eq!(report.first_block_sequence(), Some(0));
        assert_eq!(report.last_block_sequence(), Some(5));
        assert_eq!(report.sequence_gaps, 1);
        assert_eq!(report.lease_rollovers, 1);
    }

    #[test]
    fn sandbox_watchdog_requires_restart_after_consecutive_timeouts() {
        let mut watchdog = SandboxWatchdogState::new(SandboxWatchdogPolicy {
            max_consecutive_deadline_misses: 2,
            max_consecutive_heartbeat_misses: 3,
        });

        assert_eq!(
            watchdog.record_block_completion(CompletionState::TimedOut),
            WatchdogOutcome::Healthy
        );
        assert_eq!(
            watchdog.record_block_completion(CompletionState::TimedOut),
            WatchdogOutcome::RestartRequired {
                reason: WatchdogTriggerReason::DeadlineMisses,
                consecutive_misses: 2,
            }
        );
    }

    #[test]
    fn sandbox_watchdog_resets_heartbeat_misses_after_response() {
        let mut watchdog = SandboxWatchdogState::new(SandboxWatchdogPolicy {
            max_consecutive_deadline_misses: 2,
            max_consecutive_heartbeat_misses: 2,
        });

        assert_eq!(watchdog.record_heartbeat_miss(), WatchdogOutcome::Healthy);
        watchdog.record_heartbeat_response();
        assert_eq!(watchdog.consecutive_heartbeat_misses(), 0);
    }

    #[test]
    fn restart_escalation_requests_safe_mode_after_threshold() {
        let mut escalation = RestartEscalationState::new(RestartEscalationPolicy {
            safe_mode_restart_threshold: 2,
        });

        assert!(!escalation.record_watchdog_restart());
        assert_eq!(escalation.watchdog_restart_count(), 1);
        assert!(escalation.record_watchdog_restart());
        assert!(escalation.safe_mode_requested());
    }
}
