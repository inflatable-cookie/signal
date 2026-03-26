use signal_ipc::PluginMessageEnvelope;
use signal_plugin::{BlockPayload, BlockProcessResult, CompletionState};

use crate::{translate_input_events, translate_output_events};

use super::failure::failure_event;
use super::state::ClapSandboxLifecycleHarness;

impl ClapSandboxLifecycleHarness {
    pub fn process_pending_block(&mut self) -> Result<BlockProcessResult, PluginMessageEnvelope> {
        let dispatch = self.read_pending_dispatch("processBlock")?;
        let completion = self.read_completion(&dispatch)?;
        if completion.slot.state != CompletionState::ReadyForProcessing {
            return Err(failure_event(
                self.sandbox_id.as_deref().unwrap_or("unknown"),
                self.current_instance_id(),
                "processBlock",
                "protocolViolation",
                "completion slot was not ready for processing",
                Some(dispatch.header.processing_epoch),
                self.active_lease
                    .as_ref()
                    .map(|lease| lease.lease_id.clone()),
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
            return Err(failure_event(
                self.sandbox_id.as_deref().unwrap_or("unknown"),
                self.current_instance_id(),
                "processBlock",
                "invalidState",
                "sandbox state machine rejected block processing transition",
                Some(dispatch.header.processing_epoch),
                self.active_lease
                    .as_ref()
                    .map(|lease| lease.lease_id.clone()),
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

    pub fn mark_deadline_miss(&mut self) -> Result<BlockProcessResult, PluginMessageEnvelope> {
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
