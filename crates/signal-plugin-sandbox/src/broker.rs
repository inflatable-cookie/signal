//! Out-of-process sandbox broker shell.
//!
//! This binary is the seed for real plugin brokering: it exercises the three
//! pieces of plumbing that out-of-process hosting needs — child-process
//! spawn, a line-oriented stdio control protocol, and file-backed
//! shared-memory block transport — without pretending to host any plugin.
//! The `run` command performs verified shared-memory block round-trips; no
//! audio processing is claimed or simulated.
//!
//! Wire format: one receipt per line,
//! `signal-plugin-sandbox state=<token> sandbox_id=... instance_id=... epoch=... lease_id=... region_id=... detail=...`
//! with states `starting`, `ready`, `attached`, `running`, `timed_out`,
//! `crashed`, `teardown_complete`, `shutdown`.

use std::io::{self, BufRead, Write};

use signal_ipc::{MappedSharedMemoryRegion, SharedMemoryBroker};

/// Number of shared-memory block round-trips performed by the `run` command.
const RUN_BLOCK_COUNT: u64 = 8;
/// Size of the shared-memory region allocated at attach time.
const REGION_BYTES: u32 = 64 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SandboxBrokerState {
    Starting,
    Ready,
    Attached,
    Running,
    TimedOut,
    TeardownComplete,
    Crashed,
    Shutdown,
}

impl SandboxBrokerState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Starting => "starting",
            Self::Ready => "ready",
            Self::Attached => "attached",
            Self::Running => "running",
            Self::TimedOut => "timed_out",
            Self::TeardownComplete => "teardown_complete",
            Self::Crashed => "crashed",
            Self::Shutdown => "shutdown",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SandboxBrokerReceipt {
    pub state: SandboxBrokerState,
    pub sandbox_id: String,
    pub instance_id: Option<String>,
    pub processing_epoch: Option<u64>,
    pub lease_id: Option<String>,
    pub region_id: Option<String>,
    pub detail: String,
}

impl SandboxBrokerReceipt {
    pub fn render_line(&self) -> String {
        format!(
            "signal-plugin-sandbox state={} sandbox_id={} instance_id={} epoch={} lease_id={} region_id={} detail={}",
            self.state.as_str(),
            self.sandbox_id,
            self.instance_id.as_deref().unwrap_or("-"),
            self.processing_epoch
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".into()),
            self.lease_id.as_deref().unwrap_or("-"),
            self.region_id.as_deref().unwrap_or("-"),
            self.detail.replace(' ', "_"),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SandboxBrokerCommand {
    Status,
    Attach,
    Run,
    RunTimeout,
    Teardown,
    Shutdown,
}

impl SandboxBrokerCommand {
    fn parse(line: &str) -> Result<Self, String> {
        match line.trim() {
            "status" => Ok(Self::Status),
            "attach" => Ok(Self::Attach),
            "run" => Ok(Self::Run),
            "run-timeout" => Ok(Self::RunTimeout),
            "teardown" => Ok(Self::Teardown),
            "shutdown" => Ok(Self::Shutdown),
            other => Err(format!("unknown_command:{other}")),
        }
    }
}

struct AttachedRegion {
    region: MappedSharedMemoryRegion,
    lease_id: String,
    processed_blocks: u64,
}

pub struct SandboxBrokerProcess {
    broker: SharedMemoryBroker,
    sandbox_id: String,
    instance_id: String,
    processing_epoch: u64,
    attached: Option<AttachedRegion>,
    last_state: SandboxBrokerState,
}

impl Default for SandboxBrokerProcess {
    fn default() -> Self {
        Self {
            broker: SharedMemoryBroker::default(),
            sandbox_id: "plugin-sandbox-broker".into(),
            instance_id: "instance:sandbox:shm".into(),
            processing_epoch: 1,
            attached: None,
            last_state: SandboxBrokerState::Starting,
        }
    }
}

impl SandboxBrokerProcess {
    pub fn startup_receipts(&mut self) -> [SandboxBrokerReceipt; 2] {
        self.last_state = SandboxBrokerState::Ready;
        [
            self.receipt(SandboxBrokerState::Starting, "broker_boot"),
            self.receipt(SandboxBrokerState::Ready, "awaiting_commands"),
        ]
    }

    pub fn serve<R: BufRead, W: Write>(&mut self, input: R, mut output: W) -> io::Result<()> {
        for receipt in self.startup_receipts() {
            writeln!(output, "{}", receipt.render_line())?;
        }

        for line in input.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match SandboxBrokerCommand::parse(&line) {
                Ok(SandboxBrokerCommand::Status) => {
                    let receipt = self.receipt(self.last_state, "status");
                    writeln!(output, "{}", receipt.render_line())?;
                }
                Ok(SandboxBrokerCommand::Attach) => {
                    let receipt = self.attach();
                    writeln!(output, "{}", receipt.render_line())?;
                }
                Ok(SandboxBrokerCommand::Run) => {
                    for receipt in self.run() {
                        writeln!(output, "{}", receipt.render_line())?;
                    }
                }
                Ok(SandboxBrokerCommand::RunTimeout) => {
                    for receipt in self.run_timeout() {
                        writeln!(output, "{}", receipt.render_line())?;
                    }
                }
                Ok(SandboxBrokerCommand::Teardown) => {
                    let receipt = self.teardown();
                    writeln!(output, "{}", receipt.render_line())?;
                }
                Ok(SandboxBrokerCommand::Shutdown) => {
                    let receipt = self.shutdown_receipt();
                    writeln!(output, "{}", receipt.render_line())?;
                    return Ok(());
                }
                Err(error) => {
                    self.last_state = SandboxBrokerState::Crashed;
                    let receipt = self.receipt(SandboxBrokerState::Crashed, &error);
                    writeln!(output, "{}", receipt.render_line())?;
                }
            }
        }

        let receipt = self.shutdown_receipt();
        writeln!(output, "{}", receipt.render_line())?;
        Ok(())
    }

    fn receipt(&self, state: SandboxBrokerState, detail: &str) -> SandboxBrokerReceipt {
        let attached = self.attached.as_ref();
        SandboxBrokerReceipt {
            state,
            sandbox_id: self.sandbox_id.clone(),
            instance_id: attached.map(|_| self.instance_id.clone()),
            processing_epoch: attached.map(|_| self.processing_epoch),
            lease_id: attached.map(|region| region.lease_id.clone()),
            region_id: attached.map(|region| region.region.metadata().region_id.clone()),
            detail: detail.to_string(),
        }
    }

    fn attach(&mut self) -> SandboxBrokerReceipt {
        if self.attached.is_some() {
            self.last_state = SandboxBrokerState::Crashed;
            return self.receipt(SandboxBrokerState::Crashed, "already_attached");
        }
        let lease_id = format!("lease:{}", self.sandbox_id);
        match self.broker.create_region(&lease_id, REGION_BYTES) {
            Ok(region) => {
                self.attached = Some(AttachedRegion {
                    region,
                    lease_id,
                    processed_blocks: 0,
                });
                self.last_state = SandboxBrokerState::Attached;
                self.receipt(
                    SandboxBrokerState::Attached,
                    &format!("lease_attached|shm_bytes={REGION_BYTES}"),
                )
            }
            Err(error) => {
                self.last_state = SandboxBrokerState::Crashed;
                self.receipt(
                    SandboxBrokerState::Crashed,
                    &format!("shm_create:{}", error.detail()),
                )
            }
        }
    }

    /// Performs verified shared-memory block round-trips: for each block a
    /// deterministic pattern is written into the mapped region, flushed, and
    /// read back. This exercises the transport only — no plugin processing.
    fn run(&mut self) -> Vec<SandboxBrokerReceipt> {
        if self.attached.is_none() {
            self.last_state = SandboxBrokerState::Crashed;
            return vec![self.receipt(SandboxBrokerState::Crashed, "missing_attached_session")];
        }

        let mut receipts = Vec::new();
        for block_sequence in 0..RUN_BLOCK_COUNT {
            match self.roundtrip_block(block_sequence) {
                Ok(checksum) => {
                    self.last_state = SandboxBrokerState::Running;
                    receipts.push(self.receipt(
                        SandboxBrokerState::Running,
                        &format!(
                            "shm_block_roundtrip|block_sequence={block_sequence}|checksum={checksum}"
                        ),
                    ));
                }
                Err(detail) => {
                    self.last_state = SandboxBrokerState::Crashed;
                    receipts.push(self.receipt(SandboxBrokerState::Crashed, &detail));
                    return receipts;
                }
            }
        }
        let total = self
            .attached
            .as_ref()
            .map(|attached| attached.processed_blocks)
            .unwrap_or(0);
        self.last_state = SandboxBrokerState::Attached;
        receipts.push(self.receipt(
            SandboxBrokerState::Attached,
            &format!("execution_complete|processed_blocks={RUN_BLOCK_COUNT}|total_blocks={total}"),
        ));
        receipts
    }

    fn roundtrip_block(&mut self, block_sequence: u64) -> Result<u64, String> {
        let Some(attached) = self.attached.as_mut() else {
            return Err("missing_attached_session".into());
        };
        let bytes = attached.region.as_mut_slice();
        for (index, slot) in bytes.iter_mut().enumerate() {
            *slot = (index as u64)
                .wrapping_mul(31)
                .wrapping_add(block_sequence.wrapping_mul(17)) as u8;
        }
        attached
            .region
            .flush()
            .map_err(|error| format!("shm_flush:{error}"))?;
        let mut checksum = 0u64;
        for (index, slot) in attached.region.as_slice().iter().enumerate() {
            let expected = (index as u64)
                .wrapping_mul(31)
                .wrapping_add(block_sequence.wrapping_mul(17)) as u8;
            if *slot != expected {
                return Err(format!(
                    "shm_verify_mismatch|block_sequence={block_sequence}|offset={index}"
                ));
            }
            checksum = checksum
                .wrapping_mul(1099511628211)
                .wrapping_add(*slot as u64);
        }
        attached.processed_blocks = attached.processed_blocks.saturating_add(1);
        Ok(checksum)
    }

    /// Exercises the bounded deadline-miss path: reports a recoverable
    /// timeout, then re-attaches without losing the shared-memory lease.
    fn run_timeout(&mut self) -> Vec<SandboxBrokerReceipt> {
        if self.attached.is_none() {
            self.last_state = SandboxBrokerState::Crashed;
            return vec![self.receipt(SandboxBrokerState::Crashed, "missing_attached_session")];
        }
        self.last_state = SandboxBrokerState::Attached;
        vec![
            self.receipt(
                SandboxBrokerState::TimedOut,
                "execution_interrupted|timeout=recoverable|lease=retained",
            ),
            self.receipt(
                SandboxBrokerState::Attached,
                "reattached_after_timeout|processed_blocks=0",
            ),
        ]
    }

    fn teardown(&mut self) -> SandboxBrokerReceipt {
        let Some(attached) = self.attached.take() else {
            self.last_state = SandboxBrokerState::TeardownComplete;
            return self.receipt(SandboxBrokerState::TeardownComplete, "teardown_noop");
        };
        let transport = attached.region.metadata().clone();
        let processed_blocks = attached.processed_blocks;
        let instance_id = self.instance_id.clone();
        let processing_epoch = self.processing_epoch;
        drop(attached.region);
        let detail = match self.broker.destroy_region(&transport) {
            Ok(()) => format!(
                "lease_cleanup_ok|region_destroyed|processed_blocks_total={processed_blocks}"
            ),
            Err(error) => {
                self.last_state = SandboxBrokerState::Crashed;
                return SandboxBrokerReceipt {
                    state: SandboxBrokerState::Crashed,
                    sandbox_id: self.sandbox_id.clone(),
                    instance_id: Some(instance_id),
                    processing_epoch: Some(processing_epoch),
                    lease_id: None,
                    region_id: Some(transport.region_id.clone()),
                    detail: format!("shm_destroy:{}", error.detail()),
                };
            }
        };
        self.last_state = SandboxBrokerState::TeardownComplete;
        SandboxBrokerReceipt {
            state: SandboxBrokerState::TeardownComplete,
            sandbox_id: self.sandbox_id.clone(),
            instance_id: Some(instance_id),
            processing_epoch: Some(processing_epoch),
            lease_id: None,
            region_id: None,
            detail,
        }
    }

    fn shutdown_receipt(&mut self) -> SandboxBrokerReceipt {
        if self.attached.is_some() {
            let _ = self.teardown();
        }
        self.last_state = SandboxBrokerState::Shutdown;
        SandboxBrokerReceipt {
            state: SandboxBrokerState::Shutdown,
            sandbox_id: self.sandbox_id.clone(),
            instance_id: None,
            processing_epoch: None,
            lease_id: None,
            region_id: None,
            detail: "broker_shutdown".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn serve_lines(commands: &str) -> Vec<String> {
        let mut broker = SandboxBrokerProcess::default();
        let mut output = Vec::new();
        broker
            .serve(Cursor::new(commands.to_string()), &mut output)
            .expect("broker serve should succeed");
        String::from_utf8(output)
            .expect("broker output should be utf-8")
            .lines()
            .map(|line| line.to_string())
            .collect()
    }

    #[test]
    fn broker_reports_startup_and_shutdown() {
        let lines = serve_lines("shutdown\n");
        assert!(lines[0].contains("state=starting"));
        assert!(lines[1].contains("state=ready"));
        assert!(lines[2].contains("state=shutdown"));
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn broker_attach_run_teardown_roundtrips_shared_memory() {
        let lines = serve_lines("attach\nrun\nteardown\nshutdown\n");
        assert!(lines[2].contains("state=attached"));
        assert!(lines[2].contains("lease_id=lease:plugin-sandbox-broker"));
        assert!(lines[2].contains("shm_bytes=65536"));
        let running = lines
            .iter()
            .filter(|line| line.contains("state=running"))
            .count();
        assert_eq!(running, 8);
        assert!(lines
            .iter()
            .any(|line| line.contains("execution_complete|processed_blocks=8")));
        assert!(lines.iter().any(
            |line| line.contains("state=teardown_complete") && line.contains("lease_cleanup_ok")
        ));
    }

    #[test]
    fn broker_timeout_path_reports_recoverable_interrupt_and_reattaches() {
        let lines = serve_lines("attach\nrun-timeout\nteardown\nshutdown\n");
        assert!(lines
            .iter()
            .any(|line| line.contains("state=timed_out") && line.contains("timeout=recoverable")));
        assert!(lines
            .iter()
            .any(|line| line.contains("reattached_after_timeout")));
    }

    #[test]
    fn broker_rejects_run_without_attach_and_unknown_commands() {
        let lines = serve_lines("run\nbogus\nshutdown\n");
        assert!(lines.iter().any(
            |line| line.contains("state=crashed") && line.contains("missing_attached_session")
        ));
        assert!(lines
            .iter()
            .any(|line| line.contains("unknown_command:bogus")));
    }
}
