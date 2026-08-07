use std::sync::Arc;

use signal_dsp::{BinauralConvolver, OnePoleLowPass};

/// One preloaded mono sound.
pub type BankSound = Arc<Vec<f32>>;

/// One HRIR ear pair (left taps, right taps).
pub type BankHrir = (Vec<f32>, Vec<f32>);

/// Cutoffs at/above this disable the occlusion filter entirely.
pub(crate) const OCCLUSION_OPEN_HZ: f32 = 20_000.0;

pub(crate) struct VoiceSlot {
    pub(crate) convolver: BinauralConvolver,
    pub(crate) occlusion: OnePoleLowPass,
    pub(crate) occluded: bool,
    pub(crate) sound: Option<BankSound>,
    pub(crate) playhead: usize,
    pub(crate) gain: f32,
    /// Which HRIR index is loaded (dedup: re-selecting it is a no-op).
    pub(crate) hrir_index: Option<usize>,
}

impl VoiceSlot {
    pub(crate) fn active(&self) -> bool {
        self.sound.is_some()
    }

    pub(crate) fn stop(&mut self) {
        self.sound = None;
        self.playhead = 0;
    }
}

pub(crate) struct BankState {
    pub(crate) slots: Vec<VoiceSlot>,
}
