use crate::{BlockDispatch, SharedMemoryLayout};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompletionState {
    Idle,
    ReadyForProcessing,
    Processing,
    Completed,
    TimedOut,
    Invalidated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompletionSlot {
    pub state: CompletionState,
    pub processing_epoch: u64,
    pub block_sequence: u64,
}

impl CompletionSlot {
    pub fn idle() -> Self {
        Self {
            state: CompletionState::Idle,
            processing_epoch: 0,
            block_sequence: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockProcessResult {
    pub slot: CompletionSlot,
    pub generated_event_bytes: u32,
    pub fallback_applied: bool,
}

impl BlockProcessResult {
    pub const ENCODED_BYTES: usize = 32;

    pub fn ready_for(dispatch: &BlockDispatch) -> Self {
        Self {
            slot: CompletionSlot {
                state: CompletionState::ReadyForProcessing,
                processing_epoch: dispatch.header.processing_epoch,
                block_sequence: dispatch.header.block_sequence,
            },
            generated_event_bytes: 0,
            fallback_applied: false,
        }
    }

    pub fn write_to_shared_memory(
        &self,
        layout: SharedMemoryLayout,
        bytes: &mut [u8],
    ) -> Result<(), &'static str> {
        let completion_region = layout.region_slice_mut(bytes, layout.completion)?;
        if completion_region.len() < Self::ENCODED_BYTES {
            return Err("completion region is too small for block result");
        }

        completion_region[0..4]
            .copy_from_slice(&completion_state_code(self.slot.state).to_le_bytes());
        completion_region[4..8].fill(0);
        completion_region[8..16].copy_from_slice(&self.slot.processing_epoch.to_le_bytes());
        completion_region[16..24].copy_from_slice(&self.slot.block_sequence.to_le_bytes());
        completion_region[24..28].copy_from_slice(&self.generated_event_bytes.to_le_bytes());
        completion_region[28] = u8::from(self.fallback_applied);
        completion_region[29..32].fill(0);
        Ok(())
    }

    pub fn read_from_shared_memory(
        layout: SharedMemoryLayout,
        bytes: &[u8],
    ) -> Result<Self, &'static str> {
        let completion_region = layout.region_slice(bytes, layout.completion)?;
        if completion_region.len() < Self::ENCODED_BYTES {
            return Err("completion region is too small for block result");
        }

        Ok(Self {
            slot: CompletionSlot {
                state: completion_state_from_code(u32::from_le_bytes(
                    completion_region[0..4]
                        .try_into()
                        .map_err(|_| "completion state decode")?,
                ))?,
                processing_epoch: u64::from_le_bytes(
                    completion_region[8..16]
                        .try_into()
                        .map_err(|_| "completion epoch decode")?,
                ),
                block_sequence: u64::from_le_bytes(
                    completion_region[16..24]
                        .try_into()
                        .map_err(|_| "completion sequence decode")?,
                ),
            },
            generated_event_bytes: u32::from_le_bytes(
                completion_region[24..28]
                    .try_into()
                    .map_err(|_| "generated events decode")?,
            ),
            fallback_applied: completion_region[28] != 0,
        })
    }
}

fn completion_state_code(state: CompletionState) -> u32 {
    match state {
        CompletionState::Idle => 0,
        CompletionState::ReadyForProcessing => 1,
        CompletionState::Processing => 2,
        CompletionState::Completed => 3,
        CompletionState::TimedOut => 4,
        CompletionState::Invalidated => 5,
    }
}

fn completion_state_from_code(code: u32) -> Result<CompletionState, &'static str> {
    match code {
        0 => Ok(CompletionState::Idle),
        1 => Ok(CompletionState::ReadyForProcessing),
        2 => Ok(CompletionState::Processing),
        3 => Ok(CompletionState::Completed),
        4 => Ok(CompletionState::TimedOut),
        5 => Ok(CompletionState::Invalidated),
        _ => Err("unknown completion state code"),
    }
}
