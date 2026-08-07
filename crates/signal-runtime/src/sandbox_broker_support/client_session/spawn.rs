use std::{
    collections::VecDeque,
    io::{BufRead, BufReader},
    path::PathBuf,
    process::{Command, Stdio},
    sync::{mpsc, Arc, Mutex},
    time::Duration,
};

use crate::{RuntimeError, RuntimeErrorKind};

use super::super::types::*;

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
}
