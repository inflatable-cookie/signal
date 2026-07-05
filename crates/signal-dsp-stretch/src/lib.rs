//! Time-stretching backends for the Signal workspace (memo 013).
//!
//! The crate defines the abstract [`TimeStretcher`] contract — stretch audio
//! in time without shifting pitch — and ships ONE backend this round:
//! [`PhaseVocoderStretcher`], a dependency-light draft-quality phase vocoder.
//!
//! ## Quality tiers (memo 013)
//!
//! Memo 013 mandates dual quality tiers (real-time bounded-latency and
//! offline max-quality). This crate currently provides a single
//! [`StretchQuality::Draft`] tier: a plain Hann-windowed phase vocoder with
//! NO phase locking and NO transient preservation. Sustained/tonal material
//! stretches cleanly; percussive transients smear audibly at larger ratios.
//! That is the honest state of the tier — the Rubber Band (or elastique)
//! evaluation for the production tiers is recorded as open work (P-TS-001),
//! gated on an operator licensing call before distribution.
//!
//! ## Real-time posture
//!
//! This backend is OFFLINE-ONLY: it allocates its analysis/synthesis buffers
//! per call and processes whole buffers. It must never run on the audio
//! thread. Consumers that need stretched playback precompute the stretched
//! buffer control-side (anticipative posture) and hand the render plane an
//! ordinary sample buffer; a bounded-latency streaming tier is future work
//! behind the same trait.

#![warn(missing_docs)]

use rustfft::{num_complex::Complex32, FftPlanner};
use signal_primitives::Sample;

/// Quality tier of a stretch backend (memo 013 vocabulary). One tier exists
/// today; real-time and offline production tiers land with the library
/// evaluation (P-TS-001).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchQuality {
    /// Draft-quality phase vocoder: pitch-preserving, but transients smear
    /// and no formant handling. Offline use only.
    Draft,
}

/// Abstract time-stretcher contract (memo 013): stretch audio in time while
/// preserving pitch. `ratio` is the OUTPUT/INPUT duration factor — 2.0 makes
/// the audio twice as long (half speed), 0.5 twice as fast.
///
/// v1 scope is offline whole-buffer processing; the streaming/RT surface
/// (bounded latency, PDC reporting, variable ratio mid-stream) extends this
/// trait when a production backend lands.
pub trait TimeStretcher {
    /// Quality tier this backend provides — consumers must be able to make
    /// an honest offline/RT routing decision from this.
    fn quality(&self) -> StretchQuality;

    /// Current output/input duration ratio.
    fn ratio(&self) -> f64;

    /// Set the output/input duration ratio. Non-finite or non-positive
    /// values are clamped to 1.0 (identity).
    fn set_ratio(&mut self, ratio: f64);

    /// Stretch one mono buffer offline. Output length contract:
    /// `round(input.len() as f64 * ratio)` frames (identity ratio returns the
    /// input verbatim).
    fn stretch_mono(&mut self, input: &[Sample]) -> Vec<Sample>;
}

/// Draft-quality phase vocoder time-stretcher.
///
/// Classic STFT phase vocoder: fixed analysis hop, synthesis hop scaled by
/// the stretch ratio, per-bin phase propagation from the measured
/// instantaneous frequency, Hann analysis and synthesis windows with
/// window-power overlap-add normalization. Inputs shorter than one analysis
/// window fall back to linear time-domain interpolation (the honest cheap
/// path — a single window carries no phase-propagation benefit).
pub struct PhaseVocoderStretcher {
    ratio: f64,
    window_size: usize,
    analysis_hop: usize,
}

/// Default STFT window: 2048 samples (~43 ms at 48 kHz).
pub const DEFAULT_WINDOW_SIZE: usize = 2_048;
/// Default analysis hop: window / 4 (75% overlap).
pub const DEFAULT_ANALYSIS_HOP: usize = DEFAULT_WINDOW_SIZE / 4;

impl PhaseVocoderStretcher {
    /// Stretcher with the default window/hop configuration.
    pub fn new(ratio: f64) -> Self {
        Self::with_window(ratio, DEFAULT_WINDOW_SIZE, DEFAULT_ANALYSIS_HOP)
    }

    /// Stretcher with an explicit window size and analysis hop. The window
    /// is clamped to a power of two ≥ 64; the hop to `1..=window/2`.
    pub fn with_window(ratio: f64, window_size: usize, analysis_hop: usize) -> Self {
        let window_size = window_size.next_power_of_two().max(64);
        let analysis_hop = analysis_hop.clamp(1, window_size / 2);
        let mut stretcher = Self {
            ratio: 1.0,
            window_size,
            analysis_hop,
        };
        stretcher.set_ratio(ratio);
        stretcher
    }
}

impl TimeStretcher for PhaseVocoderStretcher {
    fn quality(&self) -> StretchQuality {
        StretchQuality::Draft
    }

    fn ratio(&self) -> f64 {
        self.ratio
    }

    fn set_ratio(&mut self, ratio: f64) {
        self.ratio = if ratio.is_finite() && ratio > 0.0 {
            ratio
        } else {
            1.0
        };
    }

    fn stretch_mono(&mut self, input: &[Sample]) -> Vec<Sample> {
        let target_len = (input.len() as f64 * self.ratio).round() as usize;
        if input.is_empty() || target_len == 0 {
            return Vec::new();
        }
        if (self.ratio - 1.0).abs() < 1.0e-9 {
            return input.to_vec();
        }
        if input.len() < self.window_size {
            return linear_time_scale(input, target_len);
        }
        phase_vocoder(
            input,
            target_len,
            self.ratio,
            self.window_size,
            self.analysis_hop,
        )
    }
}

/// Stretch an interleaved stereo buffer through `stretcher`, channel by
/// channel. Output frame count follows the mono length contract; both
/// channels are stretched with identical parameters so they stay
/// sample-aligned.
pub fn stretch_interleaved_stereo(
    stretcher: &mut dyn TimeStretcher,
    frames: &[Sample],
) -> Vec<Sample> {
    let frame_count = frames.len() / 2;
    let mut left = Vec::with_capacity(frame_count);
    let mut right = Vec::with_capacity(frame_count);
    for frame in frames.chunks_exact(2) {
        left.push(frame[0]);
        right.push(frame[1]);
    }
    let left = stretcher.stretch_mono(&left);
    let right = stretcher.stretch_mono(&right);
    let out_frames = left.len().min(right.len());
    let mut output = Vec::with_capacity(out_frames * 2);
    for index in 0..out_frames {
        output.push(left[index]);
        output.push(right[index]);
    }
    output
}

/// Cheap fallback for sub-window inputs: linear interpolation over time
/// (this pitch-shifts, but a sub-window buffer is too short for the phase
/// vocoder to do better; documented, deterministic).
fn linear_time_scale(input: &[Sample], target_len: usize) -> Vec<Sample> {
    if input.len() == 1 {
        return vec![input[0]; target_len];
    }
    let step = (input.len() - 1) as f64 / (target_len.max(2) - 1) as f64;
    (0..target_len)
        .map(|index| {
            let position = index as f64 * step;
            let left = position.floor() as usize;
            let right = (left + 1).min(input.len() - 1);
            let fraction = (position - left as f64) as f32;
            input[left] + (input[right] - input[left]) * fraction
        })
        .collect()
}

fn wrap_phase(phase: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    let wrapped = phase - tau * (phase / tau).round();
    wrapped
}

fn phase_vocoder(
    input: &[Sample],
    target_len: usize,
    ratio: f64,
    window_size: usize,
    analysis_hop: usize,
) -> Vec<Sample> {
    let synthesis_hop = analysis_hop as f64 * ratio;
    let bins = window_size / 2 + 1;
    let window: Vec<f32> = (0..window_size)
        .map(|index| 0.5 - 0.5 * (std::f32::consts::TAU * index as f32 / window_size as f32).cos())
        .collect();

    let mut planner = FftPlanner::<f32>::new();
    let forward = planner.plan_fft_forward(window_size);
    let inverse = planner.plan_fft_inverse(window_size);

    // Expected per-hop phase advance of each bin's center frequency.
    let omega: Vec<f32> = (0..bins)
        .map(|bin| std::f32::consts::TAU * bin as f32 * analysis_hop as f32 / window_size as f32)
        .collect();

    let frame_count = (input.len().saturating_sub(window_size)) / analysis_hop + 1;
    let output_len = target_len;
    let ola_len =
        ((frame_count.saturating_sub(1)) as f64 * synthesis_hop).ceil() as usize + window_size + 1;
    let mut output = vec![0.0f32; ola_len.max(output_len)];
    let mut norm = vec![0.0f32; output.len()];

    let mut prev_phase = vec![0.0f32; bins];
    let mut synth_phase = vec![0.0f32; bins];
    let mut buffer = vec![Complex32::new(0.0, 0.0); window_size];

    for frame_index in 0..frame_count {
        let analysis_start = frame_index * analysis_hop;
        for (slot, (sample, weight)) in buffer.iter_mut().zip(
            input[analysis_start..analysis_start + window_size]
                .iter()
                .zip(window.iter()),
        ) {
            *slot = Complex32::new(sample * weight, 0.0);
        }
        forward.process(&mut buffer);

        // Phase propagation on the positive-frequency half; the negative
        // half is rebuilt by conjugate symmetry before the inverse FFT.
        let mut spectrum = vec![Complex32::new(0.0, 0.0); window_size];
        for bin in 0..bins {
            let magnitude = buffer[bin].norm();
            let phase = buffer[bin].arg();
            if frame_index == 0 {
                synth_phase[bin] = phase;
            } else {
                let deviation = wrap_phase(phase - prev_phase[bin] - omega[bin]);
                let advance =
                    (omega[bin] + deviation) * (synthesis_hop / analysis_hop as f64) as f32;
                synth_phase[bin] = wrap_phase(synth_phase[bin] + advance);
            }
            prev_phase[bin] = phase;
            spectrum[bin] = Complex32::from_polar(magnitude, synth_phase[bin]);
        }
        for bin in 1..window_size.div_ceil(2) {
            spectrum[window_size - bin] = spectrum[bin].conj();
        }

        inverse.process(&mut spectrum);
        let synthesis_start = (frame_index as f64 * synthesis_hop).round() as usize;
        let scale = 1.0 / window_size as f32;
        for (index, weight) in window.iter().enumerate() {
            let out_index = synthesis_start + index;
            if out_index >= output.len() {
                break;
            }
            output[out_index] += spectrum[index].re * scale * weight;
            norm[out_index] += weight * weight;
        }
    }

    // Window-power OLA normalization with a floor so sparse edges never blow
    // up; then trim/pad to the length contract.
    for (sample, weight) in output.iter_mut().zip(norm.iter()) {
        if *weight > 1.0e-3 {
            *sample /= *weight;
        }
    }
    output.resize(output_len, 0.0);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(frequency_hz: f32, sample_rate_hz: f32, len: usize) -> Vec<Sample> {
        (0..len)
            .map(|index| {
                (std::f32::consts::TAU * frequency_hz * index as f32 / sample_rate_hz).sin()
            })
            .collect()
    }

    /// Dominant frequency estimate by zero-crossing count over a trimmed
    /// interior span (skips windup/tail edges).
    fn dominant_frequency_hz(samples: &[Sample], sample_rate_hz: f32) -> f32 {
        let margin = samples.len() / 8;
        let interior = &samples[margin..samples.len() - margin];
        let crossings = interior
            .windows(2)
            .filter(|pair| (pair[0] >= 0.0) != (pair[1] >= 0.0))
            .count();
        crossings as f32 * sample_rate_hz / (2.0 * interior.len() as f32)
    }

    fn rms(samples: &[Sample]) -> f32 {
        (samples.iter().map(|s| s * s).sum::<f32>() / samples.len().max(1) as f32).sqrt()
    }

    #[test]
    fn identity_ratio_is_passthrough() {
        let input = sine(440.0, 48_000.0, 10_000);
        let mut stretcher = PhaseVocoderStretcher::new(1.0);
        assert_eq!(stretcher.stretch_mono(&input), input);
    }

    #[test]
    fn ratio_clamps_invalid_values_to_identity() {
        let mut stretcher = PhaseVocoderStretcher::new(f64::NAN);
        assert_eq!(stretcher.ratio(), 1.0);
        stretcher.set_ratio(-2.0);
        assert_eq!(stretcher.ratio(), 1.0);
        stretcher.set_ratio(1.5);
        assert_eq!(stretcher.ratio(), 1.5);
    }

    #[test]
    fn stretch_honors_output_length_contract() {
        let input = sine(440.0, 48_000.0, 48_000);
        for ratio in [0.5, 0.75, 1.25, 1.5, 2.0] {
            let mut stretcher = PhaseVocoderStretcher::new(ratio);
            let output = stretcher.stretch_mono(&input);
            assert_eq!(
                output.len(),
                (input.len() as f64 * ratio).round() as usize,
                "ratio {ratio}"
            );
        }
    }

    #[test]
    fn stretch_preserves_pitch_within_tolerance() {
        let sample_rate = 48_000.0;
        let input = sine(440.0, sample_rate, 48_000);
        for ratio in [0.75, 1.5, 2.0] {
            let mut stretcher = PhaseVocoderStretcher::new(ratio);
            let output = stretcher.stretch_mono(&input);
            let frequency = dominant_frequency_hz(&output, sample_rate);
            assert!(
                (frequency - 440.0).abs() < 15.0,
                "ratio {ratio}: dominant frequency {frequency} Hz, expected ~440 Hz"
            );
            assert!(
                rms(&output) > 0.3,
                "ratio {ratio}: stretched output lost energy (rms {})",
                rms(&output)
            );
        }
    }

    #[test]
    fn sub_window_input_scales_by_linear_fallback() {
        let input: Vec<f32> = (0..100).map(|index| index as f32 / 100.0).collect();
        let mut stretcher = PhaseVocoderStretcher::new(2.0);
        let output = stretcher.stretch_mono(&input);
        assert_eq!(output.len(), 200);
        // Monotone ramp stays monotone under linear scaling.
        assert!(output.windows(2).all(|pair| pair[1] >= pair[0] - 1.0e-6));
    }

    #[test]
    fn stereo_helper_keeps_channels_aligned_and_interleaved() {
        let sample_rate = 48_000.0;
        let left = sine(440.0, sample_rate, 24_000);
        let right = sine(220.0, sample_rate, 24_000);
        let mut frames = Vec::with_capacity(left.len() * 2);
        for (l, r) in left.iter().zip(right.iter()) {
            frames.push(*l);
            frames.push(*r);
        }
        let mut stretcher = PhaseVocoderStretcher::new(1.5);
        let output = stretch_interleaved_stereo(&mut stretcher, &frames);
        assert_eq!(output.len() % 2, 0);
        assert_eq!(output.len() / 2, (24_000f64 * 1.5).round() as usize);
        let out_left: Vec<f32> = output.iter().step_by(2).copied().collect();
        let out_right: Vec<f32> = output.iter().skip(1).step_by(2).copied().collect();
        assert!((dominant_frequency_hz(&out_left, sample_rate) - 440.0).abs() < 15.0);
        assert!((dominant_frequency_hz(&out_right, sample_rate) - 220.0).abs() < 10.0);
    }
}
