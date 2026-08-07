use signal_primitives::Sample;

use super::config::PhasePropagationMode;
use super::run::run_phase_vocoder;

/// Run the draft phase-vocoder backend.
pub(crate) fn phase_vocoder(
    input: &[Sample],
    target_len: usize,
    ratio: f64,
    window_size: usize,
    analysis_hop: usize,
) -> Vec<Sample> {
    run_phase_vocoder(
        input,
        target_len,
        ratio,
        window_size,
        analysis_hop,
        PhasePropagationMode::IndependentBins,
    )
}

/// Run the identity phase-locked phase-vocoder prototype.
#[cfg(any(test, feature = "evidence"))]
pub(crate) fn phase_locked_phase_vocoder(
    input: &[Sample],
    target_len: usize,
    ratio: f64,
    window_size: usize,
    analysis_hop: usize,
) -> Vec<Sample> {
    run_phase_vocoder(
        input,
        target_len,
        ratio,
        window_size,
        analysis_hop,
        PhasePropagationMode::IdentityLocked,
    )
}

/// `g10.041` Batch 41.3 candidate: transient reset above a crossover only.
///
/// `crossover_fraction` is a fraction of Nyquist. Bins below it propagate
/// continuously through a transient instead of being reset.
///
/// Evidence-only until listening admits it. Contract `084` Rule 2 keeps a
/// candidate isolated, and Rule 5 makes listening the promotion authority, so
/// an unadopted candidate having no production caller is the intended state
/// rather than an oversight. It is reachable under the `evidence` feature so a
/// listening pack can render it.
#[cfg(any(test, feature = "evidence"))]
pub(crate) fn high_band_transient_reset_phase_vocoder(
    input: &[Sample],
    target_len: usize,
    ratio: f64,
    window_size: usize,
    analysis_hop: usize,
    crossover_fraction: f64,
) -> Vec<Sample> {
    let bins = window_size / 2 + 1;
    let crossover_bin = (crossover_fraction * bins as f64).ceil().max(0.0) as usize;
    let mode = if ratio < 1.0 {
        PhasePropagationMode::IdentityLocked
    } else {
        PhasePropagationMode::IdentityLockedTransientResetHighBand { crossover_bin }
    };
    run_phase_vocoder(input, target_len, ratio, window_size, analysis_hop, mode)
}

/// Crossover for the transient phase reset, as a fraction of Nyquist.
///
/// `240 Hz` at `48 kHz`. Bins below it propagate continuously through a
/// transient; bins above it reset.
///
/// Admitted by listening on 2026-08-05 (`g10.041`). Low-frequency content is
/// sustained *through* a transient — a bass note rings on while the attack
/// happens — so resetting its phase destroys continuity in something that never
/// restarted, which is finding `A18`. High-frequency content *is* the transient,
/// and resetting it is what stops smearing.
///
/// Bounded on both sides by measurement: below about `120 Hz` the artifact
/// returns, and at `504 Hz` transient smear regresses because the reset stops
/// reaching content that should restart.
pub const TRANSIENT_RESET_CROSSOVER_FRACTION: f64 = 0.010;

/// Run the identity phase-locked prototype with transient phase resets.
pub(crate) fn transient_reset_phase_vocoder(
    input: &[Sample],
    target_len: usize,
    ratio: f64,
    window_size: usize,
    analysis_hop: usize,
) -> Vec<Sample> {
    let bins = window_size / 2 + 1;
    let crossover_bin = (TRANSIENT_RESET_CROSSOVER_FRACTION * bins as f64).ceil() as usize;
    let mode = if ratio < 1.0 {
        PhasePropagationMode::IdentityLocked
    } else {
        PhasePropagationMode::IdentityLockedTransientResetHighBand { crossover_bin }
    };
    run_phase_vocoder(input, target_len, ratio, window_size, analysis_hop, mode)
}

/// Run the OfflineHighQuality prototype over interleaved stereo with a linked
/// mid/side analysis surface instead of independent left/right stretching.
pub(crate) fn transient_reset_phase_vocoder_linked_stereo(
    input_interleaved: &[Sample],
    target_frames: usize,
    ratio: f64,
    window_size: usize,
    analysis_hop: usize,
) -> Vec<Sample> {
    let frame_count = input_interleaved.len() / 2;
    if frame_count == 0 || target_frames == 0 {
        return Vec::new();
    }

    let mut mid = Vec::with_capacity(frame_count);
    let mut side = Vec::with_capacity(frame_count);
    for frame in input_interleaved.chunks_exact(2) {
        let left = frame[0];
        let right = frame[1];
        mid.push((left + right) * 0.5);
        side.push((left - right) * 0.5);
    }

    let stretched_mid =
        transient_reset_phase_vocoder(&mid, target_frames, ratio, window_size, analysis_hop);
    let stretched_side =
        transient_reset_phase_vocoder(&side, target_frames, ratio, window_size, analysis_hop);

    let out_frames = stretched_mid
        .len()
        .min(stretched_side.len())
        .min(target_frames);
    let mut output = Vec::with_capacity(target_frames * 2);
    for index in 0..out_frames {
        let mid = stretched_mid[index];
        let side = stretched_side[index];
        output.push(mid + side);
        output.push(mid - side);
    }
    output.resize(target_frames * 2, 0.0);
    output
}
