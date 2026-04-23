use signal_plugin::{BlockPayload, BlockProcessResult, CompletionState};

use crate::{translate_input_events, translate_output_events, ClapHarnessResult};

use super::state::ClapSandboxLifecycleHarness;

impl ClapSandboxLifecycleHarness {
    /// Reads the pending block dispatch, processes it through the CLAP instance, and writes the result back.
    pub fn process_pending_block(&mut self) -> ClapHarnessResult<BlockProcessResult> {
        let dispatch = self.read_pending_dispatch("processBlock")?;
        let completion = self.read_completion(&dispatch)?;
        if completion.slot.state != CompletionState::ReadyForProcessing {
            return Err(self.failure_error_for_instance(
                self.current_instance_id(),
                self.active_lease
                    .as_ref()
                    .map(|lease| lease.lease_id.clone()),
                "processBlock",
                "protocolViolation",
                "completion slot was not ready for processing",
                None,
            ));
        }

        let input = self.read_input_payload(&dispatch)?;
        let clap_input_events = translate_input_events(&input.events);
        self.block_machine.begin_block(&dispatch);
        if !self.block_machine.mark_processing()
            || !self.block_machine.mark_completed(
                dispatch.header.processing_epoch,
                dispatch.header.block_sequence,
            )
        {
            return Err(self.failure_error_for_instance(
                self.current_instance_id(),
                self.active_lease
                    .as_ref()
                    .map(|lease| lease.lease_id.clone()),
                "processBlock",
                "invalidState",
                "sandbox state machine rejected block processing transition",
                None,
            ));
        }
        let output = BlockPayload::new(
            input.audio.clone(),
            translate_output_events(&clap_input_events),
        );
        let result = BlockProcessResult {
            slot: self.block_machine.slot(),
            generated_event_bytes: output.events.encoded_bytes(),
            fallback_applied: false,
        };
        self.commit_processed_block(&dispatch, &output, result)
    }

    /// Records a deadline miss for the current block, applying the timeout fallback.
    pub fn mark_deadline_miss(&mut self) -> ClapHarnessResult<BlockProcessResult> {
        let dispatch = self.read_pending_dispatch("processBlock")?;
        self.block_machine.begin_block(&dispatch);
        self.block_machine.mark_timed_out();
        let result = BlockProcessResult {
            slot: self.block_machine.slot(),
            generated_event_bytes: 0,
            fallback_applied: true,
        };
        self.write_completion(result)
    }
}
