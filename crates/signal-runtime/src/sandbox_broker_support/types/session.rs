//! Sandbox broker client session handles and prepared-session records.

use std::{
    collections::VecDeque,
    process::{Child, ChildStdin},
    sync::{mpsc::Receiver, Arc, Mutex},
    time::Duration,
};

use signal_plugin::PluginIoLayout;

use super::receipt::SandboxBrokerAttachedSession;

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
/// so every read observes `Self::read_timeout`; a second thread drains the
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
    /// True after a successful boundary-level `start-processing`.
    pub processing_started: bool,
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
