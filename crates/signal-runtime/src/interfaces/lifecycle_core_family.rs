use super::*;

/// Category of a [`RuntimeError`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeErrorKind {
    /// Caller supplied a structurally invalid request.
    InvalidRequest,
    /// The requested capability is not available in this runtime build.
    UnsupportedCapability,
    /// The operation is not legal in the current lifecycle state.
    InvalidState,
    /// A required resource (sandbox, transport slot, file) is unavailable.
    ResourceUnavailable,
    /// A plugin sandbox returned an error or faulted.
    PluginFailure,
    /// The audio hardware backend reported an unrecoverable error.
    HardwareFailure,
    /// An operation exceeded its deadline or liveness window.
    Timeout,
    /// An unrecoverable error; the runtime cannot continue.
    Fatal,
}

/// Error returned by runtime control-plane operations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeError {
    /// Category of this error.
    pub kind: RuntimeErrorKind,
    /// Human-readable description of the error.
    pub message: String,
}

impl RuntimeError {
    /// Constructs a new error with the given kind and message.
    pub fn new(kind: RuntimeErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

/// Request sent by the client during the handshake phase.
///
/// The runtime inspects `client_version` for compatibility and
/// `anticipative_preferred` to decide whether to enable the prework scheduler
/// by default.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HandshakeRequest {
    /// Version string of the connecting client.
    pub client_version: String,
    /// Whether the client prefers the anticipative (prework) scheduler.
    pub anticipative_preferred: bool,
    /// Optional hint for the maximum sample rate the client expects to use.
    pub max_sample_rate_hint: Option<u32>,
}

/// Runtime capabilities returned in response to [`HandshakeRequest`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HandshakeResponse {
    /// Version string of the runtime.
    pub runtime_version: String,
    /// Protocol version negotiated for this session.
    pub protocol_version: u32,
    /// Whether the runtime supports the anticipative (prework) scheduler.
    pub supports_anticipative: bool,
    /// Whether the runtime supports dynamic reconfiguration without a restart.
    pub supports_dynamic_reconfigure: bool,
    /// Maximum number of output channels supported by the runtime.
    pub max_channels: u32,
    /// Maximum sample rate supported by the runtime in Hz.
    pub max_sample_rate: u32,
}

/// Dynamic configuration applied via `configure()`.
///
/// Unlike `RuntimeConfig` this can be reapplied without restarting the
/// runtime (when `supports_dynamic_reconfigure` is `true`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeConfigRequest {
    /// Target sample rate for audio processing.
    pub sample_rate: SampleRate,
    /// Audio block size in frames.
    pub block_size: usize,
    /// Whether to enable the anticipative (prework) scheduler.
    pub anticipative_enabled: bool,
    /// Whether realtime-safe mode (xrun-triggered sandbox suspension) is enabled.
    pub realtime_safe_mode: bool,
    /// Maximum permissible graph latency in milliseconds, if constrained.
    pub max_graph_latency_ms: Option<u32>,
    /// Maximum background service CPU load as a percentage, if constrained.
    pub max_background_load_percent: Option<u8>,
}

impl RuntimeConfigRequest {
    /// Constructs a minimal request with defaults: anticipative on, safe mode off.
    pub fn new(sample_rate: u32, block_size: usize) -> Self {
        Self {
            sample_rate: SampleRate(sample_rate),
            block_size,
            anticipative_enabled: true,
            realtime_safe_mode: false,
            max_graph_latency_ms: None,
            max_background_load_percent: None,
        }
    }
}

/// Reason the caller is stopping the runtime.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopReason {
    /// Explicit user-initiated stop.
    UserRequested,
    /// Stop driven by a hardware reconfiguration.
    DeviceReconfigure,
    /// Stop to escape a degraded state and re-enter a clean lifecycle.
    DegradedModeRecovery,
}

/// Intent of a recovery restart cycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecoveryRestartIntent {
    /// Plugin process crashed; full sandbox replacement required.
    CrashRecovery,
    /// Watchdog fired (deadline or heartbeat misses); sandbox restart required.
    WatchdogRecovery,
}

/// Parameters for a runtime restart, optionally with a new config.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RestartRequest {
    /// If `Some`, the runtime applies this config before re-entering the
    /// running state.
    pub reconfigure: Option<RuntimeConfigRequest>,
}

/// Enables or disables safe mode (xrun-triggered sandbox suspension).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SafeModeRequest {
    /// Whether safe mode should be enabled.
    pub enabled: bool,
}

/// Per-node planning record embedded in `RuntimeEngineBlockSnapshot`.
///
/// Captures the scheduling shape (lane, group, latency) and bus contract of a
/// node that the planner placed for the current block.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePlannedGraphNode {
    /// Unique identifier for this graph node.
    pub node_id: String,
    /// Execution class determining real-time vs anticipative scheduling.
    pub execution_class: GraphNodeExecutionClass,
    /// Planning group this node belongs to.
    pub group: GraphNodePlanningGroup,
    /// Latency introduced by this node in samples.
    pub latency_samples: u32,
    /// Topology role of this node within the execution graph.
    pub topology_role: GraphNodeTopologyRole,
    /// ID of the track lane this node belongs to, if any.
    pub track_lane_id: Option<String>,
    /// ID of the bus group this node belongs to, if any.
    pub bus_group_id: Option<String>,
    /// ID of the console group this node belongs to, if any.
    pub console_group_id: Option<String>,
    /// ID of the send-return pair this node belongs to, if any.
    pub send_return_id: Option<String>,
    /// ID of the input bus for this node.
    pub input_bus_id: String,
    /// ID of the output bus for this node.
    pub output_bus_id: String,
    /// Channel layout on the input bus.
    pub input_channels: ChannelLayout,
    /// Channel layout on the output bus.
    pub output_channels: ChannelLayout,
    /// Multichannel layout summary for the input bus.
    pub input_layout: RuntimeMultichannelLayoutSummary,
    /// Multichannel layout summary for the output bus.
    pub output_layout: RuntimeMultichannelLayoutSummary,
    /// Intent classification for the input bus.
    pub input_bus_intent: RuntimeBusIntent,
    /// Intent classification for the output bus.
    pub output_bus_intent: RuntimeBusIntent,
    /// Secondary (sidechain) input route summary, if applicable.
    pub secondary_input: Option<RuntimeSecondaryInputRouteSummary>,
    /// Spatial execution summary, if this node is a spatial processor.
    pub spatial_execution: Option<RuntimeSpatialExecutionSummary>,
    /// ID of the plugin sandbox backing this node, if any.
    pub plugin_sandbox_id: Option<String>,
}

/// Output of one processed audio block.
///
/// Contains the full `RuntimeEngineBlockSnapshot`, the rendered output
/// `AudioBuffer`, and per-node metering data.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeEngineBlockResult {
    /// Engine block snapshot captured after processing this block.
    pub snapshot: RuntimeEngineBlockSnapshot,
    /// Rendered audio output for this block.
    pub output: AudioBuffer,
    /// Per-bus metering data captured during this block.
    pub meter_sources: Vec<RuntimeMeterSourceSnapshot>,
}

/// Declares the number of parallel streams in a schedule sent to the runtime.
///
/// Pass to `apply_schedule_projection()` to tell the scheduler how many
/// concurrent anticipative lanes are available.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScheduleProjection {
    /// Unique identifier for the schedule being projected.
    pub schedule_id: String,
    /// Number of parallel anticipative streams available in this projection.
    pub stream_count: usize,
}
