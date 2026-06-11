use std::{
    collections::VecDeque,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process::{Child, ChildStdin, Command, Stdio},
    sync::{
        mpsc::{self, Receiver, RecvTimeoutError},
        Arc, Mutex,
    },
    time::Duration,
};

use signal_plugin::PluginIoLayout;

use crate::{
    BrokerFailureStage, BrokerInvalidationStage, CompletionSlotStage,
    PluginSandboxInstanceStateRecord, PluginSandboxLifecycleStage, PluginSandboxSpec,
    PluginSandboxTransportStage, RecoveryRestartIntent, RuntimeError, RuntimeErrorKind,
    RuntimeLv2PreparedNegotiationRecord, SignalRuntime, StopReason,
};

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
/// (`starting`, `ready`, `attached`, `running`, `crashed`,
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
    /// A teardown sequence completed.
    TeardownComplete,
    /// Any state token this client does not recognise.
    Other(String),
}

impl SandboxBrokerReceiptState {
    fn parse(token: &str) -> Self {
        match token {
            "starting" => Self::Starting,
            "ready" => Self::Ready,
            "attached" => Self::Attached,
            "running" => Self::Running,
            "crashed" => Self::Crashed,
            "teardown_complete" => Self::TeardownComplete,
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
            Self::TeardownComplete => "teardown_complete",
            Self::Other(other) => other.as_str(),
        };
        f.write_str(token)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SandboxBrokerReceiptLine {
    state: SandboxBrokerReceiptState,
    sandbox_id: String,
    instance_id: Option<String>,
    processing_epoch: Option<u64>,
    lease_id: Option<String>,
    region_id: Option<String>,
    detail: String,
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
const DEFAULT_BROKER_READ_TIMEOUT: Duration = Duration::from_secs(10);

/// Number of trailing stderr lines retained for diagnostics.
const STDERR_TAIL_LINES: usize = 16;

/// Handle to a running sandbox broker child process.
///
/// Receipt lines are read on a dedicated thread and forwarded over a channel
/// so every read observes [`Self::read_timeout`]; a second thread drains the
/// child's stderr to EOF (keeping a bounded tail for diagnostics) so a chatty
/// broker can never deadlock on a full stderr pipe. A timed-out or torn
/// session is marked failed and its child process is killed; subsequent
/// commands fail fast.
pub struct SandboxBrokerClientSession {
    child: Child,
    stdin: ChildStdin,
    receipts: Receiver<std::io::Result<String>>,
    stderr_tail: Arc<Mutex<VecDeque<String>>>,
    read_timeout: Duration,
    failed: bool,
}

/// Plugin format served by a sandbox broker.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SandboxBrokerFlavor {
    /// Demo plugin sandbox broker.
    Demo,
    /// Audio Unit (AU) plugin sandbox broker.
    Au,
    /// LV2 plugin sandbox broker.
    Lv2,
    /// VST3 plugin sandbox broker.
    Vst3,
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
    /// Plugin format served by this broker session.
    pub flavor: SandboxBrokerFlavor,
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
    /// LV2-specific prepared negotiation record, if applicable.
    pub lv2_prepared_negotiation: Option<RuntimeLv2PreparedNegotiationRecord>,
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
    /// Plugin format served by this broker.
    pub flavor: SandboxBrokerFlavor,
    /// Environment configuration for spawning the broker process.
    pub spawn_config: SandboxBrokerSpawnConfig,
    /// LV2-specific prepared negotiation record, if applicable.
    pub lv2_prepared_negotiation: Option<RuntimeLv2PreparedNegotiationRecord>,
}

impl SandboxBrokerClientSession {
    /// Returns `true` if the `SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND` environment variable is set.
    pub fn broker_enabled() -> bool {
        std::env::var_os("SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND").is_some()
    }

    /// Spawns a sandbox broker child process using environment-variable configuration.
    ///
    /// Configuration environment variables:
    ///
    /// - `SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND` — executable to spawn
    ///   (required; its presence enables the broker path).
    /// - `SIGNAL_PLUGIN_SANDBOX_BROKER_ARGS` — arguments for the command,
    ///   split with shell-style quoting: whitespace separates arguments,
    ///   single or double quotes group an argument containing whitespace
    ///   (e.g. `--root "/Library/Audio/Plug-Ins/My Plugins"`), and a
    ///   backslash escapes the next character outside single quotes.
    /// - `SIGNAL_PLUGIN_SANDBOX_BROKER_WORKDIR` — working directory for the
    ///   child process (optional).
    /// - `SIGNAL_PLUGIN_SANDBOX_BROKER_READ_TIMEOUT_MS` — receipt read
    ///   timeout in milliseconds (optional; default ten seconds; per-session
    ///   [`SandboxBrokerSpawnConfig::read_timeout_ms`] takes precedence).
    pub fn spawn_from_env(config: &SandboxBrokerSpawnConfig) -> Result<Self, RuntimeError> {
        let command = std::env::var("SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND").map_err(|_| {
            RuntimeError::new(
                RuntimeErrorKind::ResourceUnavailable,
                "missing SIGNAL_PLUGIN_SANDBOX_BROKER_COMMAND",
            )
        })?;
        let args = std::env::var("SIGNAL_PLUGIN_SANDBOX_BROKER_ARGS")
            .ok()
            .map(|value| split_broker_args(&value))
            .unwrap_or_default();
        let read_timeout = config
            .read_timeout_ms
            .or_else(|| {
                std::env::var("SIGNAL_PLUGIN_SANDBOX_BROKER_READ_TIMEOUT_MS")
                    .ok()
                    .and_then(|value| value.parse::<u64>().ok())
            })
            .map(Duration::from_millis)
            .unwrap_or(DEFAULT_BROKER_READ_TIMEOUT);

        let mut process = Command::new(&command);
        process
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(workdir) = std::env::var_os("SIGNAL_PLUGIN_SANDBOX_BROKER_WORKDIR") {
            process.current_dir(PathBuf::from(workdir));
        }
        for (key, value) in &config.env {
            process.env(key, value);
        }
        let mut child = process.spawn().map_err(io_runtime_error)?;

        let stdin = child.stdin.take().ok_or_else(|| {
            RuntimeError::new(
                RuntimeErrorKind::ResourceUnavailable,
                "sandbox broker missing stdin pipe",
            )
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            RuntimeError::new(
                RuntimeErrorKind::ResourceUnavailable,
                "sandbox broker missing stdout pipe",
            )
        })?;
        let stderr = child.stderr.take().ok_or_else(|| {
            RuntimeError::new(
                RuntimeErrorKind::ResourceUnavailable,
                "sandbox broker missing stderr pipe",
            )
        })?;

        // Receipt reader thread: forwards stdout lines over a channel so
        // every receipt read can observe the configured timeout.
        let (receipt_sender, receipts) = mpsc::channel::<std::io::Result<String>>();
        std::thread::Builder::new()
            .name("sandbox-broker-stdout".into())
            .spawn(move || {
                let mut reader = BufReader::new(stdout);
                loop {
                    let mut line = String::new();
                    match reader.read_line(&mut line) {
                        Ok(0) => break,
                        Ok(_) => {
                            if receipt_sender.send(Ok(line)).is_err() {
                                break;
                            }
                        }
                        Err(error) => {
                            let _ = receipt_sender.send(Err(error));
                            break;
                        }
                    }
                }
            })
            .map_err(io_runtime_error)?;

        // Stderr drain thread: reads to EOF so a chatty broker can never
        // block on a full stderr pipe; retains a bounded tail for diagnostics.
        let stderr_tail = Arc::new(Mutex::new(VecDeque::with_capacity(STDERR_TAIL_LINES)));
        let drain_tail = Arc::clone(&stderr_tail);
        std::thread::Builder::new()
            .name("sandbox-broker-stderr".into())
            .spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    let Ok(line) = line else { break };
                    if let Ok(mut tail) = drain_tail.lock() {
                        if tail.len() == STDERR_TAIL_LINES {
                            tail.pop_front();
                        }
                        tail.push_back(line);
                    }
                }
            })
            .map_err(io_runtime_error)?;

        Ok(Self {
            child,
            stdin,
            receipts,
            stderr_tail,
            read_timeout,
            failed: false,
        })
    }

    /// Returns the retained tail of the broker's stderr output for diagnostics.
    pub fn stderr_tail(&self) -> Vec<String> {
        self.stderr_tail
            .lock()
            .map(|tail| tail.iter().cloned().collect())
            .unwrap_or_default()
    }

    fn stderr_tail_suffix(&self) -> String {
        let tail = self.stderr_tail();
        if tail.is_empty() {
            String::new()
        } else {
            format!("; broker stderr tail: {}", tail.join(" | "))
        }
    }

    /// Marks the session failed and kills the child process so it cannot
    /// linger after a timeout or torn pipe.
    fn mark_failed(&mut self) {
        self.failed = true;
        let _ = self.child.kill();
        let _ = self.child.wait();
    }

    /// Reads the initial `starting` and `ready` receipts from the broker process.
    pub fn read_startup_receipts(&mut self) -> Result<(), RuntimeError> {
        let starting = self.read_receipt().map_err(io_runtime_error)?;
        let ready = self.read_receipt().map_err(io_runtime_error)?;
        if starting.state != SandboxBrokerReceiptState::Starting
            || ready.state != SandboxBrokerReceiptState::Ready
        {
            return Err(RuntimeError::new(
                RuntimeErrorKind::ResourceUnavailable,
                format!(
                    "unexpected broker startup sequence: {} then {}",
                    starting.state, ready.state
                ),
            ));
        }
        Ok(())
    }

    /// Sends an attach command for the given plugin flavor and reads the attached session receipt.
    pub fn attach(
        &mut self,
        flavor: SandboxBrokerFlavor,
        fallback_sandbox_id: &str,
        fallback_instance_id: &str,
    ) -> std::io::Result<SandboxBrokerAttachedSession> {
        self.write_command(attach_command_for(flavor))?;
        let attached = self.read_receipt()?;
        if attached.state != SandboxBrokerReceiptState::Attached {
            return Err(std::io::Error::other(format!(
                "unexpected broker attach state: {} ({})",
                attached.state, attached.detail
            )));
        }

        Ok(SandboxBrokerAttachedSession {
            sandbox_id: attached.sandbox_id,
            instance_id: attached
                .instance_id
                .unwrap_or_else(|| fallback_instance_id.to_string()),
            processing_epoch: attached.processing_epoch.unwrap_or(1),
            lease_id: attached
                .lease_id
                .unwrap_or_else(|| format!("lease:{fallback_sandbox_id}")),
            region_id: attached
                .region_id
                .unwrap_or_else(|| format!("region:{fallback_sandbox_id}")),
            detail: attached.detail,
        })
    }

    /// Sends a teardown command and returns the teardown receipt.
    pub fn request_teardown(
        &mut self,
        flavor: SandboxBrokerFlavor,
    ) -> std::io::Result<SandboxBrokerTeardownReceipt> {
        self.write_command(teardown_command_for(flavor))?;
        let teardown = self.read_receipt()?;
        Ok(SandboxBrokerTeardownReceipt {
            state: teardown.state,
            instance_id: teardown.instance_id,
            processing_epoch: teardown.processing_epoch,
            lease_id: teardown.lease_id,
            region_id: teardown.region_id,
            detail: teardown.detail,
        })
    }

    /// Sends a `stream-vst3` command and collects the execution stream until the broker re-attaches.
    pub fn request_vst3_execution_stream(
        &mut self,
    ) -> std::io::Result<SandboxBrokerExecutionSummary> {
        self.write_command("stream-vst3")?;
        let mut processed_blocks = 0usize;

        loop {
            let receipt = self.read_receipt()?;
            match receipt.state {
                SandboxBrokerReceiptState::Running => {
                    processed_blocks += 1;
                }
                SandboxBrokerReceiptState::Attached => {
                    return Ok(SandboxBrokerExecutionSummary {
                        processed_blocks,
                        detail: receipt.detail,
                    });
                }
                SandboxBrokerReceiptState::Crashed => {
                    return Err(std::io::Error::other(format!(
                        "sandbox broker execution stream crashed: {}",
                        receipt.detail
                    )));
                }
                other => {
                    return Err(std::io::Error::other(format!(
                        "unexpected broker execution stream state: {} ({})",
                        other, receipt.detail
                    )));
                }
            }
        }
    }

    /// Sends a `refresh-vst3` command and reads the resulting receipt.
    pub fn request_vst3_refresh(&mut self) -> std::io::Result<SandboxBrokerExecutionSummary> {
        self.write_command("refresh-vst3")?;
        let receipt = self.read_receipt()?;
        match receipt.state {
            SandboxBrokerReceiptState::Attached => Ok(SandboxBrokerExecutionSummary {
                processed_blocks: 0,
                detail: receipt.detail,
            }),
            SandboxBrokerReceiptState::Crashed => Err(std::io::Error::other(format!(
                "sandbox broker refresh crashed: {}",
                receipt.detail
            ))),
            other => Err(std::io::Error::other(format!(
                "unexpected broker refresh state: {} ({})",
                other, receipt.detail
            ))),
        }
    }

    /// Sends a `timeout-vst3` command and reads the resulting receipt.
    pub fn request_vst3_timeout(&mut self) -> std::io::Result<SandboxBrokerExecutionSummary> {
        self.write_command("timeout-vst3")?;
        let receipt = self.read_receipt()?;
        match receipt.state {
            SandboxBrokerReceiptState::Attached => Ok(SandboxBrokerExecutionSummary {
                processed_blocks: 0,
                detail: receipt.detail,
            }),
            SandboxBrokerReceiptState::Crashed => Err(std::io::Error::other(format!(
                "sandbox broker timeout path crashed: {}",
                receipt.detail
            ))),
            other => Err(std::io::Error::other(format!(
                "unexpected broker timeout state: {} ({})",
                other, receipt.detail
            ))),
        }
    }

    /// Sends a `stream-lv2` command and collects the execution stream until the broker re-attaches.
    pub fn request_lv2_execution_stream(
        &mut self,
    ) -> std::io::Result<SandboxBrokerExecutionSummary> {
        self.write_command("stream-lv2")?;
        let mut processed_blocks = 0usize;

        loop {
            let receipt = self.read_receipt()?;
            match receipt.state {
                SandboxBrokerReceiptState::Running => {
                    processed_blocks += 1;
                }
                SandboxBrokerReceiptState::Attached => {
                    return Ok(SandboxBrokerExecutionSummary {
                        processed_blocks,
                        detail: receipt.detail,
                    });
                }
                SandboxBrokerReceiptState::Crashed => {
                    return Err(std::io::Error::other(format!(
                        "sandbox broker lv2 execution stream crashed: {}",
                        receipt.detail
                    )));
                }
                other => {
                    return Err(std::io::Error::other(format!(
                        "unexpected broker lv2 execution stream state: {} ({})",
                        other, receipt.detail
                    )));
                }
            }
        }
    }

    /// Sends a shutdown command, reads the final receipt, and waits for the child process to exit.
    pub fn shutdown(mut self) -> std::io::Result<()> {
        self.write_command("shutdown")?;
        match self.read_receipt() {
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => {}
            Err(error) => return Err(error),
        }
        let status = self.child.wait()?;
        if !status.success() {
            return Err(std::io::Error::other(
                "sandbox broker exited unsuccessfully",
            ));
        }
        Ok(())
    }

    fn read_receipt(&mut self) -> std::io::Result<SandboxBrokerReceiptLine> {
        if self.failed {
            return Err(std::io::Error::other(
                "sandbox broker session already failed",
            ));
        }
        match self.receipts.recv_timeout(self.read_timeout) {
            Ok(Ok(line)) => parse_broker_receipt_line(&line),
            Ok(Err(error)) => {
                self.mark_failed();
                Err(error)
            }
            Err(RecvTimeoutError::Timeout) => {
                let timeout = self.read_timeout;
                let suffix = self.stderr_tail_suffix();
                self.mark_failed();
                Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    format!("sandbox broker receipt read timed out after {timeout:?}{suffix}"),
                ))
            }
            Err(RecvTimeoutError::Disconnected) => {
                let suffix = self.stderr_tail_suffix();
                self.mark_failed();
                Err(std::io::Error::new(
                    std::io::ErrorKind::UnexpectedEof,
                    format!("sandbox broker closed stdout{suffix}"),
                ))
            }
        }
    }

    fn write_command(&mut self, command: &str) -> std::io::Result<()> {
        if self.failed {
            return Err(std::io::Error::other(
                "sandbox broker session already failed",
            ));
        }
        writeln!(self.stdin, "{command}")?;
        self.stdin.flush()
    }
}

fn attach_command_for(flavor: SandboxBrokerFlavor) -> &'static str {
    match flavor {
        SandboxBrokerFlavor::Demo => "attach-demo",
        SandboxBrokerFlavor::Au => "attach-au",
        SandboxBrokerFlavor::Lv2 => "attach-lv2",
        SandboxBrokerFlavor::Vst3 => "attach-vst3",
    }
}

fn teardown_command_for(flavor: SandboxBrokerFlavor) -> &'static str {
    match flavor {
        SandboxBrokerFlavor::Demo => "teardown-demo",
        SandboxBrokerFlavor::Au => "teardown-au",
        SandboxBrokerFlavor::Lv2 => "teardown-lv2",
        SandboxBrokerFlavor::Vst3 => "teardown-vst3",
    }
}

/// Records lifecycle events and instance state for a successfully prepared broker sandbox.
pub fn record_broker_sandbox_prepared(
    runtime: &mut SignalRuntime,
    request: &PluginSandboxSpec,
    record: PreparedSandboxSessionRecord,
) {
    runtime.record_plugin_sandbox_lifecycle(
        request.sandbox_id.as_str(),
        PluginSandboxLifecycleStage::SandboxHandshaken,
        record.processing_epoch,
    );
    runtime.record_plugin_sandbox_lifecycle(
        request.sandbox_id.as_str(),
        PluginSandboxLifecycleStage::PluginTypeLoaded,
        record.processing_epoch,
    );
    runtime.record_plugin_sandbox_lifecycle(
        request.sandbox_id.as_str(),
        PluginSandboxLifecycleStage::InstanceCreated,
        record.processing_epoch,
    );
    runtime.record_plugin_sandbox_lifecycle(
        request.sandbox_id.as_str(),
        PluginSandboxLifecycleStage::InstancePrepared,
        record.processing_epoch,
    );
    runtime.record_plugin_sandbox_instance_state(PluginSandboxInstanceStateRecord {
        sandbox_id: request.sandbox_id.clone(),
        plugin_type_id: record.plugin_type_id,
        instance_id: record.instance_id,
        lifecycle_state: "Prepared".into(),
        readiness_state: "Ready".into(),
        degraded_reasons: Vec::new(),
        active: true,
        processing_epoch: record.processing_epoch,
        processing_sample_rate_hz: Some(record.sample_rate_hz),
        processing_max_block_frames: Some(record.max_block_frames),
        audio_inputs: Some(record.audio_inputs),
        audio_outputs: Some(record.audio_outputs),
        midi_inputs: Some(record.midi_inputs),
        midi_outputs: Some(record.midi_outputs),
        last_fault: None,
    });
    if let Some(negotiation) = record.lv2_prepared_negotiation {
        runtime.record_plugin_sandbox_lv2_prepared_negotiation(
            request.sandbox_id.as_str(),
            negotiation,
        );
    }
    runtime.record_plugin_sandbox_lifecycle(
        request.sandbox_id.as_str(),
        PluginSandboxLifecycleStage::TransportAttached,
        record.processing_epoch,
    );
    runtime.record_plugin_sandbox_transport(
        request.sandbox_id.as_str(),
        record.lease_id,
        record.region_id,
        PluginSandboxTransportStage::Attached,
        record.processing_epoch,
        record.summary,
    );
}

/// Spawns a broker process, attaches a sandbox session, and records the prepared lifecycle events.
pub fn ensure_broker_sandbox_session(
    runtime: &mut SignalRuntime,
    request: &PluginSandboxSpec,
    plugin_type_id: &str,
    default_io_layout: PluginIoLayout,
    fallback_instance_id: &str,
    flavor: SandboxBrokerFlavor,
    spawn_config: SandboxBrokerSpawnConfig,
    prepared_summary: Option<String>,
    teardown_summary: Option<String>,
    lv2_prepared_negotiation: Option<RuntimeLv2PreparedNegotiationRecord>,
) -> Result<SandboxBrokerSession, RuntimeError> {
    let mut client = SandboxBrokerClientSession::spawn_from_env(&spawn_config)?;
    client.read_startup_receipts()?;
    let attached = client
        .attach(flavor, request.sandbox_id.as_str(), fallback_instance_id)
        .map_err(|error| {
            record_broker_failure_and_convert(
                runtime,
                request.sandbox_id.as_str(),
                None,
                None,
                None,
                BrokerFailureStage::PreparePlanCreate,
                error,
            )
        })?;

    record_broker_sandbox_prepared(
        runtime,
        request,
        PreparedSandboxSessionRecord {
            plugin_type_id: plugin_type_id.to_string(),
            instance_id: attached.instance_id.clone(),
            sample_rate_hz: runtime.config().sample_rate.0,
            max_block_frames: runtime.config().graph.block_size as u32,
            audio_inputs: default_io_layout.audio_inputs,
            audio_outputs: default_io_layout.audio_outputs,
            midi_inputs: default_io_layout.midi_inputs,
            midi_outputs: default_io_layout.midi_outputs,
            processing_epoch: Some(attached.processing_epoch),
            lease_id: attached.lease_id.clone(),
            region_id: attached.region_id.clone(),
            lv2_prepared_negotiation,
            summary: Some(match &prepared_summary {
                Some(summary) => format!("broker:{} | {}", attached.detail, summary),
                None => format!("broker:{}", attached.detail),
            }),
        },
    );

    Ok(SandboxBrokerSession {
        client,
        attached,
        flavor,
        prepared_summary,
        teardown_summary,
    })
}

/// Prepares a sandbox session via the broker if enabled, otherwise via the direct path.
pub fn ensure_prepared_sandbox_session<BrokerPrepareFn, DirectPrepareFn, AfterBrokerAttachFn>(
    runtime: &mut SignalRuntime,
    request: &PluginSandboxSpec,
    broker_spec: PreparedBrokerSandboxSpec,
    broker_prepare: BrokerPrepareFn,
    direct_prepare: DirectPrepareFn,
    after_broker_attach: AfterBrokerAttachFn,
) -> Result<Option<SandboxBrokerSession>, RuntimeError>
where
    BrokerPrepareFn:
        FnOnce(&mut SignalRuntime) -> Result<(Option<String>, Option<String>), RuntimeError>,
    DirectPrepareFn:
        FnOnce(&mut SignalRuntime) -> Result<PreparedSandboxSessionRecord, RuntimeError>,
    AfterBrokerAttachFn: FnOnce(
        &mut SignalRuntime,
        &PluginSandboxSpec,
        &mut SandboxBrokerSession,
    ) -> Result<(), RuntimeError>,
{
    if SandboxBrokerClientSession::broker_enabled() {
        let (prepared_summary, teardown_summary) = broker_prepare(runtime)?;
        let mut broker_session = ensure_broker_sandbox_session(
            runtime,
            request,
            broker_spec.plugin_type_id.as_str(),
            broker_spec.default_io_layout,
            broker_spec.fallback_instance_id.as_str(),
            broker_spec.flavor,
            broker_spec.spawn_config,
            prepared_summary,
            teardown_summary,
            broker_spec.lv2_prepared_negotiation,
        )?;
        after_broker_attach(runtime, request, &mut broker_session)?;
        Ok(Some(broker_session))
    } else {
        let record = direct_prepare(runtime)?;
        record_broker_sandbox_prepared(runtime, request, record);
        Ok(None)
    }
}

/// Records lifecycle and fault events for a prepare failure caused by a protocol violation.
pub fn record_protocol_violation_prepare_failure(
    runtime: &mut SignalRuntime,
    request: &PluginSandboxSpec,
    plugin_type_id: String,
    instance_id: String,
    default_io_layout: PluginIoLayout,
    lifecycle_stage: Option<PluginSandboxLifecycleStage>,
    detail: String,
) -> RuntimeError {
    if let Some(stage) = lifecycle_stage {
        runtime.record_plugin_sandbox_lifecycle(request.sandbox_id.as_str(), stage, None);
    } else {
        runtime.record_plugin_sandbox_lifecycle(
            request.sandbox_id.as_str(),
            PluginSandboxLifecycleStage::PluginTypeLoaded,
            None,
        );
    }
    runtime.record_plugin_sandbox_fault(
        request.sandbox_id.as_str(),
        crate::PluginFaultKind::ProtocolViolation,
        detail.clone(),
        None,
    );
    runtime.record_plugin_sandbox_instance_state(PluginSandboxInstanceStateRecord {
        sandbox_id: request.sandbox_id.clone(),
        plugin_type_id,
        instance_id,
        lifecycle_state: "Faulted".into(),
        readiness_state: "Faulted".into(),
        degraded_reasons: vec![detail.clone()],
        active: false,
        processing_epoch: None,
        processing_sample_rate_hz: Some(runtime.config().sample_rate.0),
        processing_max_block_frames: Some(runtime.config().graph.block_size as u32),
        audio_inputs: Some(default_io_layout.audio_inputs),
        audio_outputs: Some(default_io_layout.audio_outputs),
        midi_inputs: Some(default_io_layout.midi_inputs),
        midi_outputs: Some(default_io_layout.midi_outputs),
        last_fault: Some(crate::PluginSandboxInstanceFaultRecord {
            kind: "ProtocolViolation".into(),
            severity: "Error".into(),
            message: detail.clone(),
        }),
    });
    RuntimeError::new(RuntimeErrorKind::InvalidRequest, detail)
}

/// Records a transport attached event with an execution summary and appends it to the session's prepared summary.
pub fn record_broker_attached_execution_summary(
    runtime: &mut SignalRuntime,
    request: &PluginSandboxSpec,
    session: &mut SandboxBrokerSession,
    execution_summary: String,
) {
    runtime.record_plugin_sandbox_transport(
        request.sandbox_id.as_str(),
        session.attached.lease_id.as_str(),
        session.attached.region_id.as_str(),
        PluginSandboxTransportStage::Attached,
        Some(session.attached.processing_epoch),
        Some(execution_summary.clone()),
    );
    session.prepared_summary = Some(match session.prepared_summary.take() {
        Some(summary) => format!("{summary} | {execution_summary}"),
        None => execution_summary,
    });
}

/// Runs the full VST3 broker execution sequence: two streams, a refresh, a refreshed stream, and a timeout.
pub fn run_vst3_broker_execution_sequence(
    runtime: &mut SignalRuntime,
    request: &PluginSandboxSpec,
    session: &mut SandboxBrokerSession,
) -> Result<(), RuntimeError> {
    let first_execution = session
        .client
        .request_vst3_execution_stream()
        .map_err(|error| {
            record_broker_failure_and_convert(
                runtime,
                request.sandbox_id.as_str(),
                Some(session.attached.lease_id.clone()),
                Some(session.attached.processing_epoch),
                None,
                BrokerFailureStage::PreparePlanCreate,
                error,
            )
        })?;
    let second_execution = session
        .client
        .request_vst3_execution_stream()
        .map_err(|error| {
            record_broker_failure_and_convert(
                runtime,
                request.sandbox_id.as_str(),
                Some(session.attached.lease_id.clone()),
                Some(session.attached.processing_epoch),
                None,
                BrokerFailureStage::PreparePlanCreate,
                error,
            )
        })?;
    let refresh = session.client.request_vst3_refresh().map_err(|error| {
        record_broker_failure_and_convert(
            runtime,
            request.sandbox_id.as_str(),
            Some(session.attached.lease_id.clone()),
            Some(session.attached.processing_epoch),
            None,
            BrokerFailureStage::PreparePlanCreate,
            error,
        )
    })?;
    let refreshed_execution = session
        .client
        .request_vst3_execution_stream()
        .map_err(|error| {
            record_broker_failure_and_convert(
                runtime,
                request.sandbox_id.as_str(),
                Some(session.attached.lease_id.clone()),
                Some(session.attached.processing_epoch),
                None,
                BrokerFailureStage::PreparePlanCreate,
                error,
            )
        })?;
    let timeout = session.client.request_vst3_timeout().map_err(|error| {
        record_broker_failure_and_convert(
            runtime,
            request.sandbox_id.as_str(),
            Some(session.attached.lease_id.clone()),
            Some(session.attached.processing_epoch),
            None,
            BrokerFailureStage::PreparePlanCreate,
            error,
        )
    })?;
    record_broker_attached_execution_summary(
        runtime,
        request,
        session,
        format!(
            "broker:{} | broker:{} | broker:{} | broker:{} | broker:{}",
            first_execution.detail,
            second_execution.detail,
            refresh.detail,
            refreshed_execution.detail,
            timeout.detail
        ),
    );
    Ok(())
}

/// Marks the start of a recovery overlap by setting the active sandbox count to two.
pub fn begin_recovery_overlap(runtime: &mut SignalRuntime) {
    runtime.set_active_plugin_sandboxes(2);
}

/// Completes a recovery overlap restart by resetting the sandbox count and optionally promoting the new transport session.
pub fn complete_recovery_overlap_restart(
    runtime: &mut SignalRuntime,
    sandbox_id: &str,
    lease_id: Option<&str>,
    region_id: Option<&str>,
) {
    runtime.set_active_plugin_sandboxes(1);
    if let (Some(lease_id), Some(region_id)) = (lease_id, region_id) {
        runtime.promote_transport_session_to_steady_state(sandbox_id, lease_id, region_id);
    }
}

/// Rolls back a recovery overlap by setting the active sandbox count to zero.
pub fn rollback_recovery_overlap(runtime: &mut SignalRuntime) {
    runtime.set_active_plugin_sandboxes(0);
}

/// Records a recovery cycle and emits broker invalidation events for any invalidated completion or lease slots.
pub fn begin_brokered_recovery_cycle<InvalidateFn>(
    runtime: &mut SignalRuntime,
    sandbox_id: &str,
    lease_id: &str,
    processing_epoch: u64,
    last_block_sequence: u64,
    intent: RecoveryRestartIntent,
    mut invalidate_active_epoch: InvalidateFn,
) where
    InvalidateFn: FnMut(u64) -> (bool, bool),
{
    runtime.record_recovery_cycle(
        sandbox_id,
        intent,
        StopReason::DegradedModeRecovery,
        Some(processing_epoch),
    );
    let (completion_invalidated, lease_invalidated) = invalidate_active_epoch(processing_epoch);
    let recovery_reason = match intent {
        RecoveryRestartIntent::CrashRecovery => "crash recovery teardown",
        RecoveryRestartIntent::WatchdogRecovery => "watchdog recovery teardown",
    };
    if completion_invalidated {
        runtime.record_completion_slot_transition(
            sandbox_id,
            lease_id,
            processing_epoch,
            last_block_sequence,
            CompletionSlotStage::Invalidated,
        );
        runtime.record_broker_invalidation(
            sandbox_id,
            lease_id,
            processing_epoch,
            Some(last_block_sequence),
            BrokerInvalidationStage::CompletionRegionInvalidated,
            recovery_reason,
        );
    }
    if lease_invalidated {
        runtime.record_broker_invalidation(
            sandbox_id,
            lease_id,
            processing_epoch,
            Some(last_block_sequence),
            BrokerInvalidationStage::LeaseEpochInvalidated,
            recovery_reason,
        );
    }
}

/// Returns an error if the overlap was requested but the competing attach unexpectedly succeeded.
pub fn handle_overlap_prepare_contention(
    requested: bool,
    competing_attach_result: Result<(), RuntimeError>,
) -> Result<(), RuntimeError> {
    if !requested {
        return Ok(());
    }

    Err(match competing_attach_result {
        Ok(()) => RuntimeError::new(
            RuntimeErrorKind::ResourceUnavailable,
            "expected overlapping replacement attach contention",
        ),
        Err(error) => error,
    })
}

/// Commits a recovery overlap restart or rolls back on failure, propagating the first error encountered.
pub fn complete_recovery_overlap_restart_or_rollback(
    restart_result: Result<(), RuntimeError>,
    inject_replacement_start_failure: bool,
    start_result: Option<Result<(), RuntimeError>>,
) -> Result<(), RuntimeError> {
    if let Err(error) = restart_result {
        return Err(error);
    }

    if inject_replacement_start_failure {
        return Err(RuntimeError::new(
            RuntimeErrorKind::ResourceUnavailable,
            "injected replacement start failure during overlap recovery",
        ));
    }

    if let Some(Err(error)) = start_result {
        return Err(error);
    }
    Ok(())
}

/// Commits a lingering recovery restart, promotes the new transport session, or rolls back on failure.
pub fn complete_lingering_recovery_restart_or_rollback(
    runtime: &mut SignalRuntime,
    sandbox_id: &str,
    restart_result: Result<(), RuntimeError>,
    replacement_transport: Option<(&str, &str)>,
    start_result: Result<(), RuntimeError>,
) -> Result<(), RuntimeError> {
    restart_result?;
    complete_recovery_overlap_restart(runtime, sandbox_id, None, None);

    let (lease_id, region_id) = replacement_transport.unwrap_or_default();
    complete_recovery_overlap_restart(
        runtime,
        sandbox_id,
        (!lease_id.is_empty()).then_some(lease_id),
        (!region_id.is_empty()).then_some(region_id),
    );

    if let Err(error) = start_result {
        rollback_recovery_overlap(runtime);
        return Err(error);
    }

    Ok(())
}

/// Outcome of tearing down the old transport session during a recovery overlap.
pub enum RecoveryOverlapOldTransportTeardownOutcome {
    /// Teardown succeeded; proceed with the replacement session.
    Continue,
    /// Teardown failed; keep the replacement session and roll back the overlap.
    RollbackKeepReplacement(RuntimeError),
    /// Teardown failed; clear the overlap and roll back entirely.
    RollbackClearOverlap(RuntimeError),
}

/// Processes teardown of the old transport session during a recovery overlap, returning the appropriate outcome.
pub fn handle_recovery_overlap_old_transport_teardown(
    runtime: &mut SignalRuntime,
    sandbox_id: &str,
    lease_id: &str,
    region_id: &str,
    processing_epoch: u64,
    last_block_sequence: u64,
    deferred_teardown_failure: bool,
    destroy_result: Result<(), String>,
    injected_old_transport_teardown_failure: bool,
    transport_teardown_result: Result<(), String>,
) -> RecoveryOverlapOldTransportTeardownOutcome {
    let detail = "recovery overlap old transport teardown";
    record_broker_transport_detach_requested(
        runtime,
        sandbox_id,
        lease_id,
        region_id,
        processing_epoch,
        detail,
    );

    if deferred_teardown_failure {
        let error = std::io::Error::other("deferred old transport teardown during recovery retry");
        record_broker_transport_detach_failure(
            runtime,
            sandbox_id,
            lease_id,
            region_id,
            processing_epoch,
            Some(last_block_sequence),
            crate::BrokerFailureStage::TransportTeardown,
            error.to_string(),
        );
        return RecoveryOverlapOldTransportTeardownOutcome::RollbackKeepReplacement(
            io_runtime_error(error),
        );
    }

    if let Err(error) = destroy_result {
        record_broker_transport_detach_failure(
            runtime,
            sandbox_id,
            lease_id,
            region_id,
            processing_epoch,
            Some(last_block_sequence),
            crate::BrokerFailureStage::TransportDestroy,
            error,
        );
        runtime.end_transport_session(sandbox_id, lease_id, region_id);
        return RecoveryOverlapOldTransportTeardownOutcome::RollbackClearOverlap(
            RuntimeError::new(
                RuntimeErrorKind::ResourceUnavailable,
                "failed to destroy recovery overlap old transport region",
            ),
        );
    }

    if injected_old_transport_teardown_failure {
        let error = std::io::Error::other(
            "injected old transport teardown failure during overlap recovery",
        );
        record_broker_transport_detach_failure(
            runtime,
            sandbox_id,
            lease_id,
            region_id,
            processing_epoch,
            Some(last_block_sequence),
            crate::BrokerFailureStage::TransportTeardown,
            error.to_string(),
        );
        runtime.end_transport_session(sandbox_id, lease_id, region_id);
        return RecoveryOverlapOldTransportTeardownOutcome::RollbackClearOverlap(io_runtime_error(
            error,
        ));
    }

    if let Err(error) = transport_teardown_result {
        record_broker_transport_detach_failure(
            runtime,
            sandbox_id,
            lease_id,
            region_id,
            processing_epoch,
            Some(last_block_sequence),
            crate::BrokerFailureStage::TransportTeardown,
            error,
        );
        runtime.end_transport_session(sandbox_id, lease_id, region_id);
        return RecoveryOverlapOldTransportTeardownOutcome::RollbackClearOverlap(
            RuntimeError::new(
                RuntimeErrorKind::ResourceUnavailable,
                "failed to tear down recovery overlap old transport",
            ),
        );
    }

    record_broker_sandbox_detached(
        runtime,
        sandbox_id,
        lease_id,
        region_id,
        processing_epoch,
        detail,
        false,
    );
    runtime.end_transport_session(sandbox_id, lease_id, region_id);
    RecoveryOverlapOldTransportTeardownOutcome::Continue
}

/// Records a transport `DetachRequested` stage event for the given sandbox session.
pub fn record_broker_transport_detach_requested(
    runtime: &mut SignalRuntime,
    sandbox_id: &str,
    lease_id: &str,
    region_id: &str,
    processing_epoch: u64,
    detail: impl Into<String>,
) {
    runtime.record_plugin_sandbox_transport(
        sandbox_id,
        lease_id,
        region_id,
        PluginSandboxTransportStage::DetachRequested,
        Some(processing_epoch),
        Some(detail.into()),
    );
}

/// Records a transport `DetachFault` stage event for the given sandbox session.
pub fn record_broker_transport_detach_fault(
    runtime: &mut SignalRuntime,
    sandbox_id: &str,
    lease_id: &str,
    region_id: &str,
    processing_epoch: u64,
    detail: impl Into<String>,
) {
    runtime.record_plugin_sandbox_transport(
        sandbox_id,
        lease_id,
        region_id,
        PluginSandboxTransportStage::DetachFault,
        Some(processing_epoch),
        Some(detail.into()),
    );
}

/// Records a broker failure and the resulting transport detach fault for the given sandbox session.
pub fn record_broker_transport_detach_failure(
    runtime: &mut SignalRuntime,
    sandbox_id: &str,
    lease_id: &str,
    region_id: &str,
    processing_epoch: u64,
    last_block_sequence: Option<u64>,
    stage: crate::BrokerFailureStage,
    detail: impl Into<String>,
) {
    let detail = detail.into();
    runtime.record_broker_failure(
        sandbox_id,
        Some(lease_id.to_string()),
        Some(processing_epoch),
        last_block_sequence,
        stage,
        detail.clone(),
    );
    record_broker_transport_detach_fault(
        runtime,
        sandbox_id,
        lease_id,
        region_id,
        processing_epoch,
        detail,
    );
}

/// Records transport `Detached` and lifecycle teardown events for a broker sandbox.
pub fn record_broker_sandbox_detached(
    runtime: &mut SignalRuntime,
    sandbox_id: &str,
    lease_id: &str,
    region_id: &str,
    processing_epoch: u64,
    detail: impl Into<String>,
    record_instance_destroyed: bool,
) {
    runtime.record_plugin_sandbox_transport(
        sandbox_id,
        lease_id,
        region_id,
        PluginSandboxTransportStage::Detached,
        Some(processing_epoch),
        Some(detail.into()),
    );
    runtime.record_plugin_sandbox_lifecycle(
        sandbox_id,
        PluginSandboxLifecycleStage::TransportTornDown,
        Some(processing_epoch),
    );
    if record_instance_destroyed {
        runtime.record_plugin_sandbox_lifecycle(
            sandbox_id,
            PluginSandboxLifecycleStage::InstanceDestroyed,
            Some(processing_epoch),
        );
    }
}

/// Records broker detach events and ends the transport session for the given sandbox.
pub fn complete_broker_transport_detach(
    runtime: &mut SignalRuntime,
    sandbox_id: &str,
    lease_id: &str,
    region_id: &str,
    processing_epoch: u64,
    detail: impl Into<String>,
    record_instance_destroyed: bool,
) {
    record_broker_sandbox_detached(
        runtime,
        sandbox_id,
        lease_id,
        region_id,
        processing_epoch,
        detail,
        record_instance_destroyed,
    );
    runtime.end_transport_session(sandbox_id, lease_id, region_id);
}

/// Requests teardown from the broker process, records the detach events, and shuts down the child.
pub fn teardown_broker_sandbox_session(
    runtime: &mut SignalRuntime,
    sandbox_id: &str,
    mut session: SandboxBrokerSession,
) -> Result<(), RuntimeError> {
    record_broker_transport_detach_requested(
        runtime,
        sandbox_id,
        session.attached.lease_id.as_str(),
        session.attached.region_id.as_str(),
        session.attached.processing_epoch,
        "broker_teardown_requested",
    );

    let teardown_receipt = session
        .client
        .request_teardown(session.flavor)
        .map_err(|error| {
            record_broker_failure_and_convert(
                runtime,
                sandbox_id,
                Some(session.attached.lease_id.clone()),
                Some(session.attached.processing_epoch),
                None,
                BrokerFailureStage::TransportTeardown,
                error,
            )
        })?;
    if teardown_receipt.state != SandboxBrokerReceiptState::TeardownComplete {
        return Err(record_broker_failure_and_convert(
            runtime,
            sandbox_id,
            Some(session.attached.lease_id.clone()),
            Some(session.attached.processing_epoch),
            None,
            BrokerFailureStage::TransportTeardown,
            std::io::Error::other(format!(
                "unexpected broker teardown state: {} ({})",
                teardown_receipt.state, teardown_receipt.detail
            )),
        ));
    }

    let detail = match &session.teardown_summary {
        Some(teardown_summary) => format!("{} | {teardown_summary}", teardown_receipt.detail),
        None => teardown_receipt.detail,
    };
    record_broker_sandbox_detached(
        runtime,
        sandbox_id,
        session.attached.lease_id.as_str(),
        session.attached.region_id.as_str(),
        session.attached.processing_epoch,
        detail,
        true,
    );

    session.client.shutdown().map_err(|error| {
        record_broker_failure_and_convert(
            runtime,
            sandbox_id,
            Some(session.attached.lease_id.clone()),
            Some(session.attached.processing_epoch),
            None,
            BrokerFailureStage::TransportTeardown,
            error,
        )
    })?;
    Ok(())
}

/// Finalises a brokered recovery transport detach, recording failures or completing cleanly as appropriate.
pub fn finalize_brokered_recovery_transport_detach(
    runtime: &mut SignalRuntime,
    sandbox_id: &str,
    lease_id: &str,
    region_id: &str,
    processing_epoch: u64,
    last_block_sequence: u64,
    detail: &str,
    record_instance_destroyed: bool,
    destroy_error: Option<String>,
    teardown_error: Option<String>,
) {
    record_broker_transport_detach_requested(
        runtime,
        sandbox_id,
        lease_id,
        region_id,
        processing_epoch,
        detail,
    );

    let destroy_failed = destroy_error.is_some();
    let teardown_failed = teardown_error.is_some();

    if let Some(error) = destroy_error {
        record_broker_transport_detach_failure(
            runtime,
            sandbox_id,
            lease_id,
            region_id,
            processing_epoch,
            Some(last_block_sequence),
            crate::BrokerFailureStage::TransportDestroy,
            error,
        );
    }

    if let Some(error) = teardown_error {
        record_broker_transport_detach_failure(
            runtime,
            sandbox_id,
            lease_id,
            region_id,
            processing_epoch,
            Some(last_block_sequence),
            crate::BrokerFailureStage::TransportTeardown,
            error,
        );
    }

    if !destroy_failed && !teardown_failed {
        complete_broker_transport_detach(
            runtime,
            sandbox_id,
            lease_id,
            region_id,
            processing_epoch,
            detail,
            record_instance_destroyed,
        );
    }
}

/// Splits `SIGNAL_PLUGIN_SANDBOX_BROKER_ARGS` with shell-style quoting.
///
/// Whitespace separates arguments; single or double quotes group an argument
/// containing whitespace; a backslash escapes the next character outside
/// single quotes. Unterminated quotes consume the rest of the input.
fn split_broker_args(value: &str) -> Vec<String> {
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

fn parse_broker_receipt_line(line: &str) -> std::io::Result<SandboxBrokerReceiptLine> {
    let mut state = None;
    let mut sandbox_id = None;
    let mut instance_id = None;
    let mut processing_epoch = None;
    let mut lease_id = None;
    let mut region_id = None;
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
            _ => {}
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
        detail: detail.ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "broker receipt missing detail",
            )
        })?,
    })
}

fn io_runtime_error(error: std::io::Error) -> RuntimeError {
    RuntimeError::new(RuntimeErrorKind::ResourceUnavailable, error.to_string())
}

fn record_broker_failure_and_convert(
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

#[cfg(test)]
mod tests {
    use super::{parse_broker_receipt_line, split_broker_args, SandboxBrokerReceiptState};

    #[test]
    fn splits_plain_whitespace_args() {
        assert_eq!(
            split_broker_args("run -q -p signal-plugin-sandbox"),
            vec!["run", "-q", "-p", "signal-plugin-sandbox"]
        );
        assert!(split_broker_args("   ").is_empty());
        assert!(split_broker_args("").is_empty());
    }

    #[test]
    fn splits_quoted_paths_with_spaces() {
        assert_eq!(
            split_broker_args("--root \"/Library/Audio/Plug-Ins/My Plugins\" -v"),
            vec!["--root", "/Library/Audio/Plug-Ins/My Plugins", "-v"]
        );
        assert_eq!(
            split_broker_args("--name 'demo plugin'"),
            vec!["--name", "demo plugin"]
        );
        assert_eq!(
            split_broker_args(r"--path /tmp/with\ space"),
            vec!["--path", "/tmp/with space"]
        );
        assert_eq!(split_broker_args("''"), vec![""]);
        assert_eq!(
            split_broker_args(r#"--mix pre"fix mid"post"#),
            vec!["--mix", "prefix midpost"]
        );
    }

    #[test]
    fn parses_broker_receipt_lines() {
        let receipt = parse_broker_receipt_line(
            "signal-plugin-sandbox state=attached sandbox_id=plugin-sandbox-broker instance_id=instance:sandbox:default epoch=1 lease_id=lease:plugin-sandbox-broker region_id=region:plugin-sandbox-broker detail=lease_attached\n",
        )
        .expect("receipt should parse");

        assert_eq!(receipt.state, SandboxBrokerReceiptState::Attached);
        assert_eq!(receipt.sandbox_id, "plugin-sandbox-broker");
        assert_eq!(
            receipt.instance_id.as_deref(),
            Some("instance:sandbox:default")
        );
        assert_eq!(receipt.processing_epoch, Some(1));
        assert_eq!(
            receipt.lease_id.as_deref(),
            Some("lease:plugin-sandbox-broker")
        );
        assert_eq!(
            receipt.region_id.as_deref(),
            Some("region:plugin-sandbox-broker")
        );
        assert_eq!(receipt.detail, "lease_attached");
    }
}
