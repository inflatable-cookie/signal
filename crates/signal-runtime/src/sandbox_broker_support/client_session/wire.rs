use std::io::Write;
use std::sync::mpsc::RecvTimeoutError;

use super::super::types::*;

impl SandboxBrokerClientSession {
    /// Returns the retained tail of the broker's stderr output for diagnostics.
    pub fn stderr_tail(&self) -> Vec<String> {
        self.stderr_tail
            .lock()
            .map(|tail| tail.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub(crate) fn stderr_tail_suffix(&self) -> String {
        let tail = self.stderr_tail();
        if tail.is_empty() {
            String::new()
        } else {
            format!("; broker stderr tail: {}", tail.join(" | "))
        }
    }

    /// Marks the session failed and kills the child process so it cannot
    /// linger after a timeout or torn pipe.
    pub(crate) fn mark_failed(&mut self) {
        self.failed = true;
        let _ = self.child.kill();
        let _ = self.child.wait();
    }

    /// Read the next COMMAND receipt. Spontaneous child→parent
    /// notifications (`editor_closed` with `reason=user_closed`, g13.027)
    /// may arrive interleaved with command receipts; they are recorded for
    /// [`Self::take_editor_closed_notifications`] and never satisfy a
    /// command wait.
    pub(crate) fn read_receipt(&mut self) -> std::io::Result<SandboxBrokerReceiptLine> {
        loop {
            let receipt = self.read_receipt_line()?;
            match user_closed_editor_instance(&receipt) {
                Some(instance) => self.editor_closed_notifications.push_back(instance),
                None => return Ok(receipt),
            }
        }
    }

    pub(crate) fn read_receipt_line(&mut self) -> std::io::Result<SandboxBrokerReceiptLine> {
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

    pub(crate) fn write_command(&mut self, command: &str) -> std::io::Result<()> {
        if self.failed {
            return Err(std::io::Error::other(
                "sandbox broker session already failed",
            ));
        }
        writeln!(self.stdin, "{command}")?;
        self.stdin.flush()
    }
}
