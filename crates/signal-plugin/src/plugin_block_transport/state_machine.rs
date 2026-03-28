use crate::{BlockDispatch, CompletionSlot, CompletionState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SandboxStateMachine {
    slot: CompletionSlot,
}

impl SandboxStateMachine {
    pub fn new() -> Self {
        Self {
            slot: CompletionSlot::idle(),
        }
    }

    pub fn slot(&self) -> CompletionSlot {
        self.slot
    }

    pub fn begin_block(&mut self, dispatch: &BlockDispatch) {
        self.slot = CompletionSlot {
            state: CompletionState::ReadyForProcessing,
            processing_epoch: dispatch.header.processing_epoch,
            block_sequence: dispatch.header.block_sequence,
        };
    }

    pub fn mark_processing(&mut self) -> bool {
        if matches!(self.slot.state, CompletionState::ReadyForProcessing) {
            self.slot.state = CompletionState::Processing;
            return true;
        }
        false
    }

    pub fn mark_completed(&mut self, processing_epoch: u64, block_sequence: u64) -> bool {
        if matches!(self.slot.state, CompletionState::Processing)
            && self.slot.processing_epoch == processing_epoch
            && self.slot.block_sequence == block_sequence
        {
            self.slot.state = CompletionState::Completed;
            return true;
        }
        false
    }

    pub fn mark_timed_out(&mut self) {
        self.slot.state = CompletionState::TimedOut;
    }

    pub fn invalidate_epoch(&mut self, processing_epoch: u64) {
        self.slot = CompletionSlot {
            state: CompletionState::Invalidated,
            processing_epoch,
            block_sequence: 0,
        };
    }
}

impl Default for SandboxStateMachine {
    fn default() -> Self {
        Self::new()
    }
}
