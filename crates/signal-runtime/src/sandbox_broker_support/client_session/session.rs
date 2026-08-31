use super::super::types::*;

impl SandboxBrokerClientSession {
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

    /// OS process id of the spawned broker child.
    ///
    /// Same value as [`std::process::Child::id`] on the owned child. Hosts
    /// use this for crash evidence and `sandbox_pid` reporting without an
    /// external process probe.
    pub fn child_pid(&self) -> u32 {
        self.child.id()
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
}
