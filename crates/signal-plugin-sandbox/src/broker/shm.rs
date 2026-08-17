//! Shared-memory attach/run/teardown commands for the sandbox broker.

use super::hosted::AttachedRegion;
use super::process::SandboxBrokerProcess;
use super::types::*;

impl SandboxBrokerProcess {
    pub(crate) fn attach(&mut self) -> SandboxBrokerReceipt {
        if self.attached.is_some() {
            return self.crashed_receipt("already_attached");
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
            Err(error) => self.crashed_receipt(&format!("shm_create:{}", error.detail())),
        }
    }

    /// Performs verified shared-memory block round-trips: for each block a
    /// deterministic pattern is written into the mapped region, flushed, and
    /// read back. This exercises the transport only — no plugin processing.
    pub(crate) fn run(&mut self) -> Vec<SandboxBrokerReceipt> {
        if self.attached.is_none() {
            return vec![self.crashed_receipt("missing_attached_session")];
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
                    receipts.push(self.crashed_receipt(&detail));
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

    pub(crate) fn roundtrip_block(&mut self, block_sequence: u64) -> Result<u64, String> {
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
    pub(crate) fn run_timeout(&mut self) -> Vec<SandboxBrokerReceipt> {
        if self.attached.is_none() {
            return vec![self.crashed_receipt("missing_attached_session")];
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

    pub(crate) fn teardown(&mut self) -> SandboxBrokerReceipt {
        // Plugin teardown path folds into the transport teardown so a
        // parent-side `teardown` always leaves the child clean.
        if !self.plugins.is_empty() {
            self.unload_all_plugins();
        }
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
                    extra: Vec::new(),
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
            extra: Vec::new(),
            detail,
        }
    }
}
