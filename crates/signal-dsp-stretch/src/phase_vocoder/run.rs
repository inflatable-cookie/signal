use signal_primitives::Sample;

use super::config::{PhasePropagationMode, PhaseVocoderConfig};
use super::engine::DraftPhaseVocoder;

/// Largest synthesis hop, as a fraction of the window, that keeps overlap-add
/// coverage intact. Above this the window-power normalization gate zeroes
/// output samples. Frozen by Contract `046`, 2026-07-27 addendum.
pub(crate) const MAX_SYNTHESIS_HOP_WINDOW_FRACTION: f64 = 0.75;

/// Analysis hop that keeps `analysis_hop * ratio` inside the overlap coverage
/// bound. Returns `analysis_hop` unchanged whenever the configured geometry
/// already satisfies it, so ratios inside the bound stay byte-exact.
pub(crate) fn overlap_safe_analysis_hop(
    analysis_hop: usize,
    ratio: f64,
    window_size: usize,
) -> usize {
    if !ratio.is_finite() || ratio <= 0.0 {
        return analysis_hop;
    }
    let max_synthesis_hop = MAX_SYNTHESIS_HOP_WINDOW_FRACTION * window_size as f64;
    if analysis_hop as f64 * ratio <= max_synthesis_hop {
        return analysis_hop;
    }
    ((max_synthesis_hop / ratio).floor() as usize).clamp(1, analysis_hop)
}

pub(crate) fn run_phase_vocoder(
    input: &[Sample],
    target_len: usize,
    ratio: f64,
    window_size: usize,
    analysis_hop: usize,
    mode: PhasePropagationMode,
) -> Vec<Sample> {
    let analysis_hop = overlap_safe_analysis_hop(analysis_hop, ratio, window_size);
    // Give the first and last source samples complete overlapping analysis
    // windows. The extra post-roll hop guarantees that the cropped target
    // remains inside synthesized coverage for both compression and expansion.
    let prefix_frames = window_size / 2;
    let suffix_frames = window_size + analysis_hop;
    let mut padded_input = vec![0.0; prefix_frames + input.len() + suffix_frames];
    padded_input[prefix_frames..prefix_frames + input.len()].copy_from_slice(input);

    // Synthesis hops scale while the samples inside each synthesis window do
    // not, so the crop starts at the analysis window centre. The prefix is
    // exactly half a window, so the ratio-scaled centre offset is always zero
    // and the start is simply that half window; the earlier expression spelled
    // this as `((prefix - half_window) * ratio + half_window)`, which reads as
    // ratio-dependent but never was.
    let output_start = prefix_frames;
    let output_end = output_start + target_len;
    let config =
        PhaseVocoderConfig::new(&padded_input, output_end, ratio, window_size, analysis_hop);
    let mut engine = DraftPhaseVocoder::new(config, mode);
    engine.process(&padded_input);
    engine.finish()[output_start..output_end].to_vec()
}
