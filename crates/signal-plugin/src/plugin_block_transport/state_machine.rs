use crate::{BlockDispatch, CompletionSlot, CompletionState};

/// Sandbox-side state machine that tracks the lifecycle of the current block.
///
/// The state machine enforces that block-processing transitions happen in the correct order
/// (dispatch → ready → processing → completed/timed-out) and guards against out-of-order
/// or duplicate completions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SandboxStateMachine {
    slot: CompletionSlot,
}

impl SandboxStateMachine {
    /// Creates a new state machine in the idle state.
    pub fn new() -> Self {
        Self {
            slot: CompletionSlot::idle(),
        }
    }

    /// Returns the current completion slot.
    pub fn slot(&self) -> CompletionSlot {
        self.slot
    }

    /// Transitions the slot to `ReadyForProcessing` for the given dispatch.
    pub fn begin_block(&mut self, dispatch: &BlockDispatch) {
        self.slot = CompletionSlot {
            state: CompletionState::ReadyForProcessing,
            processing_epoch: dispatch.header.processing_epoch,
            block_sequence: dispatch.header.block_sequence,
        };
    }

    /// Transitions from `ReadyForProcessing` to `Processing`.
    ///
    /// Returns `true` on success, `false` if the slot was not in the expected state.
    pub fn mark_processing(&mut self) -> bool {
        if matches!(self.slot.state, CompletionState::ReadyForProcessing) {
            self.slot.state = CompletionState::Processing;
            return true;
        }
        false
    }

    /// Transitions from `Processing` to `Completed` if epoch and sequence match.
    ///
    /// Returns `true` on success, `false` if the slot was not in the expected state or the
    /// identifiers do not match the current block.
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

    /// Forces the slot into the `TimedOut` state.
    pub fn mark_timed_out(&mut self) {
        self.slot.state = CompletionState::TimedOut;
    }

    /// Sets the slot to `Invalidated` for the given epoch, clearing the block sequence.
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
