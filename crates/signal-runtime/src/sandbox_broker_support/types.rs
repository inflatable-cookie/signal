//! Sandbox broker support types and small helpers.

use std::{
    collections::VecDeque,
    process::{Child, ChildStdin},
    sync::{mpsc::Receiver, Arc, Mutex},
    time::Duration,
};

use signal_plugin::PluginIoLayout;

use crate::{BrokerFailureStage, RuntimeError, RuntimeErrorKind, SignalRuntime};

/// Record of a successfully attached sandbox broker session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SandboxBrokerAttachedSession {
    /// Stable identifier for the attached sandbox.
    pub sandbox_id: String,
    /// Instance identifier assigned by the broker.
    pub instance_id: String,
    /// Processing epoch at which the session was attached.
    pub processing_epoch: u64,
    /// Shared-memory lease identifier for this session.
    pub lease_id: String,
    /// Shared-memory region identifier for this session.
    pub region_id: String,
    /// Human-readable detail from the broker attach receipt.
    pub detail: String,
}

/// Broker receipt `state=` token, parsed into a typed value.
///
/// The wire format is unchanged: states travel as plain tokens
/// (`starting`, `ready`, `attached`, `running`, `crashed`, `timed_out`,
/// `teardown_complete`). Unknown tokens are preserved in [`Self::Other`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SandboxBrokerReceiptState {
    /// Broker process is starting up.
    Starting,
    /// Broker process is ready to accept commands.
    Ready,
    /// A sandbox session is attached.
    Attached,
    /// An execution stream block was processed.
    Running,
    /// The brokered sandbox crashed.
    Crashed,
    /// A bounded deadline-miss (timeout) path was exercised.
    TimedOut,
    /// A teardown sequence completed.
    TeardownComplete,
    /// A CLAP plugin was loaded and its inventory enumerated (g11.012).
    PluginLoaded,
    /// The loaded plugin activated and leased its audio block region.
    PluginActivated,
    /// The plugin's main-port layout is outside the supported v1 shape
    /// (stereo in + stereo out); the chain compiles passthrough.
    LayoutUnsupported,
    /// The child's audio thread is live and serving process requests.
    ProcessingStarted,
    /// The child's audio thread stopped.
    ProcessingStopped,
    /// The plugin deactivated and its audio block region was destroyed.
    PluginDeactivated,
    /// The plugin instance was destroyed and its library closed.
    PluginUnloaded,
    /// A parameter write batch was applied to the loaded instance
    /// (g12.023).
    ParamSet,
    /// A child-owned editor window opened for the loaded instance
    /// (g13.027).
    EditorOpened,
    /// A child-owned editor window closed — as a command receipt
    /// (`reason=host_requested` / `reason=not_open`) or as a spontaneous
    /// child→parent notification (`reason=user_closed`).
    EditorClosed,
    /// Any state token this client does not recognise.
    Other(String),
}

impl SandboxBrokerReceiptState {
    pub(crate) fn parse(token: &str) -> Self {
        match token {
            "starting" => Self::Starting,
            "ready" => Self::Ready,
            "attached" => Self::Attached,
            "running" => Self::Running,
            "crashed" => Self::Crashed,
            "timed_out" => Self::TimedOut,
            "teardown_complete" => Self::TeardownComplete,
            "plugin_loaded" => Self::PluginLoaded,
            "plugin_activated" => Self::PluginActivated,
            "layout_unsupported" => Self::LayoutUnsupported,
            "processing_started" => Self::ProcessingStarted,
            "processing_stopped" => Self::ProcessingStopped,
            "plugin_deactivated" => Self::PluginDeactivated,
            "plugin_unloaded" => Self::PluginUnloaded,
            "param_set" => Self::ParamSet,
            "editor_opened" => Self::EditorOpened,
            "editor_closed" => Self::EditorClosed,
            other => Self::Other(other.to_string()),
        }
    }
}

impl std::fmt::Display for SandboxBrokerReceiptState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let token = match self {
            Self::Starting => "starting",
            Self::Ready => "ready",
            Self::Attached => "attached",
            Self::Running => "running",
            Self::Crashed => "crashed",
            Self::TimedOut => "timed_out",
            Self::TeardownComplete => "teardown_complete",
            Self::PluginLoaded => "plugin_loaded",
            Self::PluginActivated => "plugin_activated",
            Self::LayoutUnsupported => "layout_unsupported",
            Self::ProcessingStarted => "processing_started",
            Self::ProcessingStopped => "processing_stopped",
            Self::PluginDeactivated => "plugin_deactivated",
            Self::PluginUnloaded => "plugin_unloaded",
            Self::ParamSet => "param_set",
            Self::EditorOpened => "editor_opened",
            Self::EditorClosed => "editor_closed",
            Self::Other(other) => other.as_str(),
        };
        f.write_str(token)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SandboxBrokerReceiptLine {
    pub(crate) state: SandboxBrokerReceiptState,
    pub(crate) sandbox_id: String,
    pub(crate) instance_id: Option<String>,
    pub(crate) processing_epoch: Option<u64>,
    pub(crate) lease_id: Option<String>,
    pub(crate) region_id: Option<String>,
    /// Extra `key=value` tokens (plugin inventory, shm coordinates);
    /// values remain wire-encoded until interpreted.
    pub(crate) extra: Vec<(String, String)>,
    pub(crate) detail: String,
}

impl SandboxBrokerReceiptLine {
    pub(crate) fn extra_value(&self, key: &str) -> Option<&str> {
        self.extra
            .iter()
            .find(|(extra_key, _)| extra_key == key)
            .map(|(_, value)| value.as_str())
    }
}

/// One plugin parameter from the child's load-time inventory (read-only
/// phase 1; descriptor fields enriched in g12.013).
#[derive(Clone, Debug, PartialEq)]
pub struct SandboxPluginParameter {
    /// Stable plugin-format parameter id.
    pub parameter_id: u32,
    /// Human-readable parameter name.
    pub name: String,
    /// Minimum plain value.
    pub min_value: f32,
    /// Maximum plain value.
    pub max_value: f32,
    /// Default value (normalized).
    pub default_value: f32,
    /// Display unit (e.g. "dB", "Hz"); `None` when the format reports none.
    pub unit: Option<String>,
    /// Discrete step count across the plain range (`Some(1)` = toggle);
    /// `None` for continuous parameters.
    pub step_count: Option<u32>,
    /// Whether the host may automate this parameter. Legacy receipts
    /// without a flags token parse as automatable (the pre-g12 assumption).
    pub is_automatable: bool,
    /// Whether this is the plugin's bypass parameter.
    pub is_bypass: bool,
}

/// Receipt of a successful `load-plugin`: the child's parameter inventory
/// and port summary.
#[derive(Clone, Debug, PartialEq)]
pub struct SandboxPluginInventory {
    /// Parameters enumerated by the child at load.
    pub parameters: Vec<SandboxPluginParameter>,
    /// Human-readable detail from the receipt.
    pub detail: String,
}

/// Receipt of a successful `activate`: everything the parent needs to attach
/// the shared-memory audio block region.
#[derive(Clone, Debug, PartialEq)]
pub struct SandboxPluginAudioLease {
    /// Region identifier assigned by the child's shared-memory broker.
    pub region_id: String,
    /// Lease identifier for the audio block region.
    pub lease_id: String,
    /// Filesystem path of the region's backing file.
    pub shm_path: String,
    /// Total region size in bytes.
    pub shm_bytes: u32,
    /// Largest block the region carries.
    pub max_frames: u32,
    /// Interleaved channel count (2 in v1).
    pub channels: u32,
    /// Human-readable detail from the receipt.
    pub detail: String,
}

/// Outcome of an `activate` request.
#[derive(Clone, Debug, PartialEq)]
pub enum SandboxPluginActivateOutcome {
    /// The plugin activated; the audio block region is ready to attach.
    Activated(SandboxPluginAudioLease),
    /// The plugin's main-port layout is unsupported in phase 1; the caller
    /// should compile the chain as passthrough.
    LayoutUnsupported {
        /// Human-readable detail from the receipt.
        detail: String,
    },
}

/// Decode a wire-encoded token value (see the broker's `encode_wire_token`).
pub(crate) fn decode_wire_token(value: &str) -> String {
    let mut decoded = Vec::with_capacity(value.len());
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 3 <= bytes.len() {
            let hex = value.get(index + 1..index + 3);
            if let Some(byte) = hex.and_then(|hex| u8::from_str_radix(hex, 16).ok()) {
                decoded.push(byte);
                index += 3;
                continue;
            }
        }
        decoded.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

/// Parse the `params=` inventory blob:
/// `id:name:min:max:default[:unit:steps:flags];...`.
///
/// The three descriptor tokens are additive (g12.013) and version-tolerant
/// both ways: a legacy five-field entry parses with `None`/legacy defaults
/// (automatable, not bypass), and entries with unknown trailing tokens
/// parse by ignoring them.
pub(crate) fn parse_parameter_inventory(blob: &str) -> Vec<SandboxPluginParameter> {
    blob.split(';')
        .filter(|entry| !entry.is_empty())
        .filter_map(|entry| {
            let mut fields = entry.split(':');
            let parameter_id = fields.next()?.parse::<u32>().ok()?;
            let name = decode_wire_token(fields.next()?);
            let min_value = fields.next()?.parse::<f32>().ok()?;
            let max_value = fields.next()?.parse::<f32>().ok()?;
            let default_value = fields.next()?.parse::<f32>().ok()?;
            let unit = fields
                .next()
                .map(decode_wire_token)
                .filter(|unit| !unit.is_empty());
            let step_count = fields.next().and_then(|value| value.parse::<u32>().ok());
            let (is_automatable, is_bypass) = match fields.next() {
                Some(flags) => (flags.contains('a'), flags.contains('b')),
                None => (true, false),
            };
            Some(SandboxPluginParameter {
                parameter_id,
                name,
                min_value,
                max_value,
                default_value,
                unit,
                step_count,
                is_automatable,
                is_bypass,
            })
        })
        .collect()
}

/// Receipt of a successful `open-editor` (g13.027): the child-owned
/// editor window is up, sized to the plugin's initial content size.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SandboxEditorOpened {
    /// Initial editor content width (logical units).
    pub width: u32,
    /// Initial editor content height (logical units).
    pub height: u32,
    /// Human-readable detail from the receipt.
    pub detail: String,
}

/// Receipt of a `close-editor` (g13.027). Tolerant wire: `closed` is
/// `false` when no editor with that instance was open (the user closed it
/// first, or it never opened).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SandboxEditorClosed {
    /// Whether an open editor was actually closed by this command.
    pub closed: bool,
    /// Human-readable detail from the receipt.
    pub detail: String,
}

/// The decoded editor instance of a spontaneous user-close notification,
/// or `None` for ordinary command receipts.
pub(crate) fn user_closed_editor_instance(receipt: &SandboxBrokerReceiptLine) -> Option<String> {
    if receipt.state != SandboxBrokerReceiptState::EditorClosed {
        return None;
    }
    if receipt.extra_value("reason") != Some("user_closed") {
        return None;
    }
    Some(
        receipt
            .extra_value("editor_instance")
            .map(decode_wire_token)
            .unwrap_or_default(),
    )
}

/// Receipt returned by a broker teardown request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SandboxBrokerTeardownReceipt {
    /// Teardown receipt state reported by the broker.
    pub state: SandboxBrokerReceiptState,
    /// Instance identifier from the receipt, if present.
    pub instance_id: Option<String>,
    /// Processing epoch from the receipt, if present.
    pub processing_epoch: Option<u64>,
    /// Shared-memory lease identifier from the receipt, if present.
    pub lease_id: Option<String>,
    /// Shared-memory region identifier from the receipt, if present.
    pub region_id: Option<String>,
    /// Human-readable detail from the receipt.
    pub detail: String,
}

/// Default receipt read timeout.
///
/// Generous because test harnesses spawn the broker via `cargo run`, which can
/// pay a relink cost before the first `starting` receipt appears. Override per
/// session with [`SandboxBrokerSpawnConfig::read_timeout_ms`] or globally with
/// the `SIGNAL_PLUGIN_SANDBOX_BROKER_READ_TIMEOUT_MS` environment variable.
pub(crate) const DEFAULT_BROKER_READ_TIMEOUT: Duration = Duration::from_secs(10);

/// Number of trailing stderr lines retained for diagnostics.
pub(crate) const STDERR_TAIL_LINES: usize = 16;

/// Handle to a running sandbox broker child process.
///
/// Receipt lines are read on a dedicated thread and forwarded over a channel
/// so every read observes [`Self::read_timeout`]; a second thread drains the
/// child's stderr to EOF (keeping a bounded tail for diagnostics) so a chatty
/// broker can never deadlock on a full stderr pipe. A timed-out or torn
/// session is marked failed and its child process is killed; subsequent
/// commands fail fast.
pub struct SandboxBrokerClientSession {
    pub(crate) child: Child,
    pub(crate) stdin: ChildStdin,
    pub(crate) receipts: Receiver<std::io::Result<String>>,
    pub(crate) stderr_tail: Arc<Mutex<VecDeque<String>>>,
    pub(crate) read_timeout: Duration,
    pub(crate) failed: bool,
    /// Receipt lines pulled off the channel while polling for spontaneous
    /// notifications that turned out to be command receipts; consumed
    /// before the channel on the next read.
    pub(crate) pushback: VecDeque<String>,
    /// Editor instances reported closed by the child on its own
    /// (`reason=user_closed`), drained via
    /// [`Self::take_editor_closed_notifications`].
    pub(crate) editor_closed_notifications: VecDeque<String>,
}

/// Environment variable overrides for spawning a sandbox broker process.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct SandboxBrokerSpawnConfig {
    /// Environment variable overrides to apply when spawning the broker process.
    pub env: Vec<(String, String)>,
    /// Receipt read timeout in milliseconds for this session.
    ///
    /// Falls back to `SIGNAL_PLUGIN_SANDBOX_BROKER_READ_TIMEOUT_MS`, then to
    /// the built-in default of ten seconds.
    pub read_timeout_ms: Option<u64>,
}

/// Combines the live broker client session with the attached session record.
pub struct SandboxBrokerSession {
    /// Live child-process client session.
    pub client: SandboxBrokerClientSession,
    /// Attached session record returned by the broker.
    pub attached: SandboxBrokerAttachedSession,
    /// Summary from the broker prepare phase, if completed.
    pub prepared_summary: Option<String>,
    /// Summary from the broker teardown phase, if completed.
    pub teardown_summary: Option<String>,
}

/// Summary of blocks processed through a broker execution sequence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SandboxBrokerExecutionSummary {
    /// Number of audio blocks processed in this broker execution sequence.
    pub processed_blocks: usize,
    /// Human-readable detail from the execution sequence.
    pub detail: String,
}

/// Record describing a prepared (pre-activated) sandbox session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedSandboxSessionRecord {
    /// Plugin type identifier for the prepared sandbox.
    pub plugin_type_id: String,
    /// Instance identifier assigned to the prepared sandbox.
    pub instance_id: String,
    /// Sample rate in Hz used for preparation.
    pub sample_rate_hz: u32,
    /// Maximum block size in frames used for preparation.
    pub max_block_frames: u32,
    /// Number of audio input channels.
    pub audio_inputs: u16,
    /// Number of audio output channels.
    pub audio_outputs: u16,
    /// Number of MIDI input channels.
    pub midi_inputs: u16,
    /// Number of MIDI output channels.
    pub midi_outputs: u16,
    /// Processing epoch at which preparation occurred, if known.
    pub processing_epoch: Option<u64>,
    /// Shared-memory lease identifier for this session.
    pub lease_id: String,
    /// Shared-memory region identifier for this session.
    pub region_id: String,
    /// Human-readable summary of the prepared session, if available.
    pub summary: Option<String>,
}

/// Specification for spawning and preparing a brokered plugin sandbox.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedBrokerSandboxSpec {
    /// Plugin type identifier for the sandbox to be spawned.
    pub plugin_type_id: String,
    /// Default I/O layout used when the broker does not specify one.
    pub default_io_layout: PluginIoLayout,
    /// Fallback instance identifier used if the broker does not assign one.
    pub fallback_instance_id: String,
    /// Environment configuration for spawning the broker process.
    pub spawn_config: SandboxBrokerSpawnConfig,
}

pub(crate) fn split_broker_args(value: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_arg = false;
    let mut chars = value.chars();

    'outer: while let Some(ch) = chars.next() {
        match ch {
            c if c.is_whitespace() => {
                if in_arg {
                    args.push(std::mem::take(&mut current));
                    in_arg = false;
                }
            }
            '\\' => {
                in_arg = true;
                if let Some(escaped) = chars.next() {
                    current.push(escaped);
                }
            }
            '\'' => {
                in_arg = true;
                for inner in chars.by_ref() {
                    if inner == '\'' {
                        continue 'outer;
                    }
                    current.push(inner);
                }
                break;
            }
            '"' => {
                in_arg = true;
                while let Some(inner) = chars.next() {
                    match inner {
                        '"' => continue 'outer,
                        '\\' => {
                            if let Some(escaped) = chars.next() {
                                current.push(escaped);
                            }
                        }
                        other => current.push(other),
                    }
                }
                break;
            }
            other => {
                in_arg = true;
                current.push(other);
            }
        }
    }
    if in_arg {
        args.push(current);
    }
    args
}

pub(crate) fn parse_broker_receipt_line(line: &str) -> std::io::Result<SandboxBrokerReceiptLine> {
    let mut state = None;
    let mut sandbox_id = None;
    let mut instance_id = None;
    let mut processing_epoch = None;
    let mut lease_id = None;
    let mut region_id = None;
    let mut extra = Vec::new();
    let mut detail = None;

    for token in line.split_whitespace().skip(1) {
        let Some((key, value)) = token.split_once('=') else {
            continue;
        };
        match key {
            "state" => state = Some(SandboxBrokerReceiptState::parse(value)),
            "sandbox_id" => sandbox_id = Some(value.to_string()),
            "instance_id" if value != "-" => instance_id = Some(value.to_string()),
            "epoch" if value != "-" => processing_epoch = value.parse::<u64>().ok(),
            "lease_id" if value != "-" => lease_id = Some(value.to_string()),
            "region_id" if value != "-" => region_id = Some(value.to_string()),
            "detail" => detail = Some(value.to_string()),
            other => extra.push((other.to_string(), value.to_string())),
        }
    }

    Ok(SandboxBrokerReceiptLine {
        state: state.ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "broker receipt missing state",
            )
        })?,
        sandbox_id: sandbox_id.ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "broker receipt missing sandbox_id",
            )
        })?,
        instance_id,
        processing_epoch,
        lease_id,
        region_id,
        extra,
        detail: detail.ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "broker receipt missing detail",
            )
        })?,
    })
}

pub(crate) fn io_runtime_error(error: std::io::Error) -> RuntimeError {
    RuntimeError::new(RuntimeErrorKind::ResourceUnavailable, error.to_string())
}

pub(crate) fn record_broker_failure_and_convert(
    runtime: &mut SignalRuntime,
    sandbox_id: &str,
    lease_id: Option<String>,
    processing_epoch: Option<u64>,
    block_sequence: Option<u64>,
    stage: BrokerFailureStage,
    error: std::io::Error,
) -> RuntimeError {
    let detail = error.to_string();
    runtime.record_broker_failure(
        sandbox_id,
        lease_id,
        processing_epoch,
        block_sequence,
        stage,
        detail.clone(),
    );
    RuntimeError::new(RuntimeErrorKind::ResourceUnavailable, detail)
}
