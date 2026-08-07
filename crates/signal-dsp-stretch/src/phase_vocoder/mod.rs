mod config;
mod engine;
mod entry;
mod run;
mod wrap_phase;

#[cfg(any(test, feature = "evidence"))]
pub(crate) use entry::{high_band_transient_reset_phase_vocoder, phase_locked_phase_vocoder};
pub(crate) use entry::{
    phase_vocoder, transient_reset_phase_vocoder, transient_reset_phase_vocoder_linked_stereo,
};

#[cfg(test)]
pub(crate) use config::{PhasePropagationMode, PhaseVocoderConfig, SpectralPeak};
#[cfg(test)]
pub(crate) use engine::DraftPhaseVocoder;
#[cfg(test)]
pub(crate) use run::{
    overlap_safe_analysis_hop, run_phase_vocoder, MAX_SYNTHESIS_HOP_WINDOW_FRACTION,
};
#[cfg(test)]
pub(crate) use signal_primitives::Sample;
#[cfg(test)]
pub(crate) use wrap_phase::wrap_phase;

#[cfg(test)]
mod tests;
