use signal_primitives::Sample;

pub(crate) struct PhaseVocoderConfig {
    pub(crate) target_len: usize,
    pub(crate) window_size: usize,
    pub(crate) analysis_hop: usize,
    pub(crate) synthesis_hop: f64,
    pub(crate) bins: usize,
    pub(crate) frame_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SpectralPeak {
    pub(crate) bin: usize,
    pub(crate) magnitude: f32,
}

// `IdentityLockedTransientReset` resets every bin. It is no longer the shipped
// path — `g10.041` replaced it with the high-band variant after listening — but
// it is retained as the control the `A18` evidence compares against.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PhasePropagationMode {
    IndependentBins,
    IdentityLocked,
    IdentityLockedTransientReset,
    /// `g10.041` Batch 41.3 candidate for `A18`.
    ///
    /// Resets transient phase only above a crossover, leaving low bins to
    /// propagate continuously. Low-frequency content is *sustained through* a
    /// transient — a bass note rings on while the attack happens — so resetting
    /// its phase destroys continuity in something that never restarted. High
    /// bins are the transient, and resetting them is what stops smearing.
    ///
    /// The crossover is a fraction of Nyquist rather than a frequency, because
    /// the stretch API is sample-rate agnostic all the way down.
    IdentityLockedTransientResetHighBand {
        crossover_bin: usize,
    },
}

impl PhaseVocoderConfig {
    pub(crate) fn new(
        input: &[Sample],
        target_len: usize,
        ratio: f64,
        window_size: usize,
        analysis_hop: usize,
    ) -> Self {
        Self {
            target_len,
            window_size,
            analysis_hop,
            synthesis_hop: analysis_hop as f64 * ratio,
            bins: window_size / 2 + 1,
            frame_count: (input.len().saturating_sub(window_size)) / analysis_hop + 1,
        }
    }
}
