//! Sandbox broker client session (attach/activate/editor/teardown wire).

use std::{
    collections::VecDeque,
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process::{Command, Stdio},
    sync::{
        mpsc::{self, RecvTimeoutError},
        Arc, Mutex,
    },
    time::Duration,
};

use crate::{RuntimeError, RuntimeErrorKind};

use super::types::*;

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
        Self::spawn_command(&command, &args, config)
    }

    /// Spawns a sandbox broker child process from an explicit command line
    /// (hosts that know their broker binary path use this instead of the
    /// environment-variable configuration; the same env fallbacks apply for
    /// workdir and read timeout).
    pub fn spawn_command(
        command: &str,
        args: &[String],
        config: &SandboxBrokerSpawnConfig,
    ) -> Result<Self, RuntimeError> {
        let read_timeout = config
            .read_timeout_ms
            .or_else(|| {
                std::env::var("SIGNAL_PLUGIN_SANDBOX_BROKER_READ_TIMEOUT_MS")
                    .ok()
                    .and_then(|value| value.parse::<u64>().ok())
            })
            .map(Duration::from_millis)
            .unwrap_or(DEFAULT_BROKER_READ_TIMEOUT);

        let mut process = Command::new(command);
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
            pushback: VecDeque::new(),
            editor_closed_notifications: VecDeque::new(),
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

    /// Sends the attach command and reads the attached session receipt.
    pub fn attach(
        &mut self,
        fallback_sandbox_id: &str,
        fallback_instance_id: &str,
    ) -> std::io::Result<SandboxBrokerAttachedSession> {
        self.write_command("attach")?;
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

    /// Sends the teardown command and returns the teardown receipt.
    pub fn request_teardown(&mut self) -> std::io::Result<SandboxBrokerTeardownReceipt> {
        self.write_command("teardown")?;
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

    /// Sends the `run` command and collects the shared-memory block exercise
    /// stream until the broker re-attaches.
    pub fn request_execution_stream(&mut self) -> std::io::Result<SandboxBrokerExecutionSummary> {
        self.write_command("run")?;
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

    /// Sends the `run-timeout` command, which exercises the broker's bounded
    /// deadline-miss path: a `timed_out` receipt followed by re-attachment.
    pub fn request_timeout_probe(&mut self) -> std::io::Result<SandboxBrokerExecutionSummary> {
        self.write_command("run-timeout")?;
        let timed_out = self.read_receipt()?;
        if timed_out.state != SandboxBrokerReceiptState::TimedOut {
            return Err(std::io::Error::other(format!(
                "unexpected broker timeout state: {} ({})",
                timed_out.state, timed_out.detail
            )));
        }
        let reattached = self.read_receipt()?;
        match reattached.state {
            SandboxBrokerReceiptState::Attached => Ok(SandboxBrokerExecutionSummary {
                processed_blocks: 0,
                detail: format!("{} | {}", timed_out.detail, reattached.detail),
            }),
            SandboxBrokerReceiptState::Crashed => Err(std::io::Error::other(format!(
                "sandbox broker timeout path crashed: {}",
                reattached.detail
            ))),
            other => Err(std::io::Error::other(format!(
                "unexpected broker timeout state: {} ({})",
                other, reattached.detail
            ))),
        }
    }

    /// Kill the child broker process and mark the session failed. The
    /// crash-isolation escape hatch for a wedged or misbehaving child;
    /// subsequent commands fail fast.
    pub fn kill(&mut self) {
        self.mark_failed();
    }

    /// Whether the child broker process is still alive. `false` after the
    /// session failed (timeout/torn pipe kills the child) or the child
    /// exited on its own — the crash-isolation signal callers key bypass on.
    pub fn is_alive(&mut self) -> bool {
        if self.failed {
            return false;
        }
        match self.child.try_wait() {
            Ok(None) => true,
            Ok(Some(_)) | Err(_) => false,
        }
    }

    /// Sends `load-plugin` and returns the child's parameter inventory.
    ///
    /// The v1 wire format is whitespace-separated: library paths containing
    /// whitespace are rejected here rather than corrupting the command line.
    pub fn load_plugin(
        &mut self,
        library_path: &str,
        plugin_id: &str,
    ) -> std::io::Result<SandboxPluginInventory> {
        if library_path.chars().any(char::is_whitespace)
            || plugin_id.chars().any(char::is_whitespace)
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "plugin library paths and ids with whitespace are unsupported by the v1 broker wire format",
            ));
        }
        self.write_command(&format!("load-plugin {library_path} {plugin_id}"))?;
        let receipt = self.read_receipt()?;
        if receipt.state != SandboxBrokerReceiptState::PluginLoaded {
            return Err(std::io::Error::other(format!(
                "unexpected broker load-plugin state: {} ({})",
                receipt.state, receipt.detail
            )));
        }
        let parameters = receipt
            .extra_value("params")
            .map(parse_parameter_inventory)
            .unwrap_or_default();
        Ok(SandboxPluginInventory {
            parameters,
            detail: receipt.detail,
        })
    }

    /// Sends `activate` and returns either the audio block lease or the
    /// typed layout rejection.
    pub fn activate_plugin(
        &mut self,
        sample_rate_hz: u32,
        min_frames: u32,
        max_frames: u32,
    ) -> std::io::Result<SandboxPluginActivateOutcome> {
        self.write_command(&format!(
            "activate {sample_rate_hz} {min_frames} {max_frames}"
        ))?;
        let receipt = self.read_receipt()?;
        match receipt.state {
            SandboxBrokerReceiptState::PluginActivated => {
                let shm_path = receipt
                    .extra_value("shm_path")
                    .map(decode_wire_token)
                    .ok_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "plugin_activated receipt missing shm_path",
                        )
                    })?;
                let shm_bytes = receipt
                    .extra_value("shm_bytes")
                    .and_then(|value| value.parse::<u32>().ok())
                    .ok_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "plugin_activated receipt missing shm_bytes",
                        )
                    })?;
                let lease_max_frames = receipt
                    .extra_value("max_frames")
                    .and_then(|value| value.parse::<u32>().ok())
                    .unwrap_or(max_frames);
                let channels = receipt
                    .extra_value("channels")
                    .and_then(|value| value.parse::<u32>().ok())
                    .unwrap_or(2);
                Ok(SandboxPluginActivateOutcome::Activated(
                    SandboxPluginAudioLease {
                        region_id: receipt.region_id.unwrap_or_default(),
                        lease_id: receipt.lease_id.unwrap_or_default(),
                        shm_path,
                        shm_bytes,
                        max_frames: lease_max_frames,
                        channels,
                        detail: receipt.detail,
                    },
                ))
            }
            SandboxBrokerReceiptState::LayoutUnsupported => {
                Ok(SandboxPluginActivateOutcome::LayoutUnsupported {
                    detail: receipt.detail,
                })
            }
            other => Err(std::io::Error::other(format!(
                "unexpected broker activate state: {} ({})",
                other, receipt.detail
            ))),
        }
    }

    fn simple_plugin_command(
        &mut self,
        command: &str,
        expected: SandboxBrokerReceiptState,
    ) -> std::io::Result<String> {
        self.write_command(command)?;
        let receipt = self.read_receipt()?;
        if receipt.state != expected {
            return Err(std::io::Error::other(format!(
                "unexpected broker {command} state: {} ({})",
                receipt.state, receipt.detail
            )));
        }
        Ok(receipt.detail)
    }

    /// Sends one normalized 0..1 parameter write (g12.023). The child
    /// queues it on the format's audio-thread-correct set path; delivery
    /// is block-boundary.
    pub fn set_parameter(&mut self, parameter_id: u32, normalized: f32) -> std::io::Result<String> {
        self.set_parameters(&[(parameter_id, normalized)])
    }

    /// Sends a batched `(parameter_id, normalized 0..1)` write (g12.023):
    /// one `set-params` command, one `param_set` receipt for the whole
    /// batch.
    pub fn set_parameters(&mut self, changes: &[(u32, f32)]) -> std::io::Result<String> {
        if changes.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "set_parameters requires at least one change",
            ));
        }
        let blob = changes
            .iter()
            .map(|(parameter_id, normalized)| format!("{parameter_id}:{normalized}"))
            .collect::<Vec<_>>()
            .join(";");
        self.simple_plugin_command(
            &format!("set-params {blob}"),
            SandboxBrokerReceiptState::ParamSet,
        )
    }

    /// Sends `open-editor <instance>` (g13.027): the child opens a
    /// child-owned floating editor window titled by `instance` on its main
    /// thread, hosting the plugin's editor via the format's gui adapter.
    /// The RT audio path is untouched. `instance` is an opaque parent
    /// token; the v1 wire format forbids whitespace in it.
    pub fn open_editor(&mut self, instance: &str) -> std::io::Result<SandboxEditorOpened> {
        if instance.is_empty() || instance.chars().any(char::is_whitespace) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "editor instance tokens must be non-empty and whitespace-free on the v1 wire",
            ));
        }
        self.write_command(&format!("open-editor {instance}"))?;
        let receipt = self.read_receipt()?;
        if receipt.state != SandboxBrokerReceiptState::EditorOpened {
            return Err(std::io::Error::other(format!(
                "unexpected broker open-editor state: {} ({})",
                receipt.state, receipt.detail
            )));
        }
        let width = receipt
            .extra_value("width")
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(0);
        let height = receipt
            .extra_value("height")
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(0);
        Ok(SandboxEditorOpened {
            width,
            height,
            detail: receipt.detail,
        })
    }

    /// Sends `close-editor <instance>` (g13.027): the child destroys the
    /// editor window. Tolerant of an already-closed editor — see
    /// [`SandboxEditorClosed::closed`].
    pub fn close_editor(&mut self, instance: &str) -> std::io::Result<SandboxEditorClosed> {
        if instance.is_empty() || instance.chars().any(char::is_whitespace) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "editor instance tokens must be non-empty and whitespace-free on the v1 wire",
            ));
        }
        self.write_command(&format!("close-editor {instance}"))?;
        let receipt = self.read_receipt()?;
        if receipt.state != SandboxBrokerReceiptState::EditorClosed {
            return Err(std::io::Error::other(format!(
                "unexpected broker close-editor state: {} ({})",
                receipt.state, receipt.detail
            )));
        }
        Ok(SandboxEditorClosed {
            closed: receipt.extra_value("reason") == Some("host_requested"),
            detail: receipt.detail,
        })
    }

    /// Drain the editor instances the child reported closed on its own
    /// (the user clicked the window's close button — `reason=user_closed`
    /// notifications, g13.027). Polls pending receipt lines without
    /// blocking; lines that are not user-close notifications are kept for
    /// the next command read.
    pub fn take_editor_closed_notifications(&mut self) -> Vec<String> {
        while let Ok(Ok(line)) = self.receipts.try_recv() {
            match parse_broker_receipt_line(&line) {
                Ok(receipt) => match user_closed_editor_instance(&receipt) {
                    Some(instance) => self.editor_closed_notifications.push_back(instance),
                    None => self.pushback.push_back(line),
                },
                Err(_) => self.pushback.push_back(line),
            }
        }
        self.editor_closed_notifications.drain(..).collect()
    }

    /// Sends `start-processing`: the child spawns its audio thread.
    pub fn start_processing(&mut self) -> std::io::Result<String> {
        self.simple_plugin_command(
            "start-processing",
            SandboxBrokerReceiptState::ProcessingStarted,
        )
    }

    /// Sends `stop-processing`: the child stops and joins its audio thread.
    pub fn stop_processing(&mut self) -> std::io::Result<String> {
        self.simple_plugin_command(
            "stop-processing",
            SandboxBrokerReceiptState::ProcessingStopped,
        )
    }

    /// Sends `deactivate`: the child deactivates the plugin and destroys the
    /// audio block region (any parent mapping goes stale first — detach
    /// before calling this).
    pub fn deactivate_plugin(&mut self) -> std::io::Result<String> {
        self.simple_plugin_command("deactivate", SandboxBrokerReceiptState::PluginDeactivated)
    }

    /// Sends `unload-plugin`: full plugin teardown in the child.
    pub fn unload_plugin(&mut self) -> std::io::Result<String> {
        self.simple_plugin_command("unload-plugin", SandboxBrokerReceiptState::PluginUnloaded)
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

    /// Read the next COMMAND receipt. Spontaneous child→parent
    /// notifications (`editor_closed` with `reason=user_closed`, g13.027)
    /// may arrive interleaved with command receipts; they are recorded for
    /// [`Self::take_editor_closed_notifications`] and never satisfy a
    /// command wait.
    fn read_receipt(&mut self) -> std::io::Result<SandboxBrokerReceiptLine> {
        loop {
            let receipt = self.read_receipt_line()?;
            match user_closed_editor_instance(&receipt) {
                Some(instance) => self.editor_closed_notifications.push_back(instance),
                None => return Ok(receipt),
            }
        }
    }

    fn read_receipt_line(&mut self) -> std::io::Result<SandboxBrokerReceiptLine> {
        if self.failed {
            return Err(std::io::Error::other(
                "sandbox broker session already failed",
            ));
        }
        if let Some(line) = self.pushback.pop_front() {
            return parse_broker_receipt_line(&line);
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
