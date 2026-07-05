//! Time-stretching backends for the Signal workspace.
//!
//! The crate defines the abstract [`TimeStretcher`] contract — stretch audio
//! in time without shifting pitch — and ships ONE backend this round:
//! [`PhaseVocoderStretcher`], a dependency-light draft-quality phase vocoder.
//!
//! ## Signal-owned backend tiers
//!
//! Signal owns three execution tiers:
//!
//! - [`StretchBackendTier::Repitch`]: render-plane rate conversion, pitch
//!   changes with tempo, realtime-safe today.
//! - [`StretchBackendTier::RealtimePreview`]: bounded-latency pitch-preserving
//!   preview stretch, planned.
//! - [`StretchBackendTier::OfflineHighQuality`]: deterministic
//!   export/cache/freeze stretch, planned as the quality reference tier.
//!
//! The current [`PhaseVocoderStretcher`] remains [`StretchQuality::Draft`]: a
//! plain Hann-windowed phase vocoder with NO phase locking and NO transient
//! preservation. Sustained/tonal material stretches cleanly; percussive
//! transients smear audibly at larger ratios. Rubber Band-class quality is the
//! target for the planned Signal-native tiers, but Rubber Band source is not
//! an implementation input.
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

mod benchmark;
mod phase_vocoder;

pub use benchmark::{
    assess_stretch_metrics, format_stretch_acceptance_report, generate_synthetic_stretch_audio,
    output_length_drift_samples, synthetic_stretch_corpus_cases, StretchAcceptanceReport,
    StretchAcceptanceSeverity, StretchAcceptanceStatus, StretchCorpusCase, StretchCorpusFamily,
    StretchCorpusSource, StretchMetric, StretchMetricAssessment, StretchMetricLimit,
    StretchMetricValue, StretchSyntheticAudio, STRETCH_BENCHMARK_CORPUS,
};

use phase_vocoder::phase_vocoder;
use signal_primitives::Sample;

/// Quality tier of a stretch backend (memo 013 vocabulary). One tier exists
/// today; real-time and offline production tiers land with the library
/// evaluation (P-TS-001).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchQuality {
    /// Draft-quality phase vocoder: pitch-preserving, but transients smear
    /// and no formant handling. Offline use only.
    Draft,
    /// Bounded-latency preview quality. Planned; not implemented by the
    /// current backend.
    RealtimePreview,
    /// Highest-quality deterministic offline/export quality. Planned; not
    /// implemented by the current backend.
    OfflineHighQuality,
}

/// Signal-owned stretch execution tier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchBackendTier {
    /// Existing render-plane varispeed path. Tempo changes also shift pitch.
    Repitch,
    /// Planned bounded-latency preview tier for live audition and playback.
    RealtimePreview,
    /// Planned deterministic high-quality tier for exports, freeze, and
    /// cached post-warp artifacts.
    OfflineHighQuality,
}

/// Implementation status for one tier in the Signal-native stretch program.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchBackendStatus {
    /// The tier is implemented in Signal today.
    Implemented,
    /// The tier is designed but not implemented.
    Planned,
}

/// Clean-room architecture contract for one Signal-owned tier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StretchBackendPlan {
    /// Signal-owned execution tier.
    pub tier: StretchBackendTier,
    /// Current implementation status.
    pub status: StretchBackendStatus,
    /// Whether tempo and pitch can be controlled independently.
    pub independent_tempo_and_pitch: bool,
    /// Whether stretch ratio may change within one render.
    pub dynamic_ratio: bool,
    /// Whether transient preservation is part of the tier contract.
    pub transient_preservation: bool,
    /// Whether stereo or multichannel vertical coherence is part of the tier
    /// contract.
    pub vertical_phase_coherence: bool,
    /// Whether the tier promises sample-accurate or near-sample-accurate
    /// timeline alignment.
    pub alignment_promised: bool,
    /// Whether processing may run on the realtime audio thread.
    pub audio_thread_safe: bool,
    /// Whether rendered output is deterministic enough for cache identity,
    /// export reuse, and regression comparison.
    pub deterministic_output: bool,
}

/// Signal-owned tier plan. This is a code-level mirror of the roadmap
/// contract so callers can gate behavior without vendor-specific names.
pub const SIGNAL_STRETCH_BACKEND_PLAN: [StretchBackendPlan; 3] = [
    StretchBackendPlan {
        tier: StretchBackendTier::Repitch,
        status: StretchBackendStatus::Implemented,
        independent_tempo_and_pitch: false,
        dynamic_ratio: true,
        transient_preservation: true,
        vertical_phase_coherence: true,
        alignment_promised: true,
        audio_thread_safe: true,
        deterministic_output: true,
    },
    StretchBackendPlan {
        tier: StretchBackendTier::RealtimePreview,
        status: StretchBackendStatus::Planned,
        independent_tempo_and_pitch: true,
        dynamic_ratio: true,
        transient_preservation: true,
        vertical_phase_coherence: true,
        alignment_promised: true,
        audio_thread_safe: false,
        deterministic_output: true,
    },
    StretchBackendPlan {
        tier: StretchBackendTier::OfflineHighQuality,
        status: StretchBackendStatus::Planned,
        independent_tempo_and_pitch: true,
        dynamic_ratio: true,
        transient_preservation: true,
        vertical_phase_coherence: true,
        alignment_promised: true,
        audio_thread_safe: false,
        deterministic_output: true,
    },
];

/// Returns the Signal-owned plan for `tier`.
pub fn stretch_backend_plan(tier: StretchBackendTier) -> StretchBackendPlan {
    SIGNAL_STRETCH_BACKEND_PLAN
        .iter()
        .copied()
        .find(|plan| plan.tier == tier)
        .expect("all StretchBackendTier variants are represented")
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

    #[test]
    fn backend_plan_tracks_signal_owned_tiers() {
        assert_eq!(SIGNAL_STRETCH_BACKEND_PLAN.len(), 3);
        assert_eq!(
            stretch_backend_plan(StretchBackendTier::Repitch).status,
            StretchBackendStatus::Implemented
        );
        let preview = stretch_backend_plan(StretchBackendTier::RealtimePreview);
        assert_eq!(preview.status, StretchBackendStatus::Planned);
        assert!(preview.independent_tempo_and_pitch);
        assert!(preview.dynamic_ratio);
        assert!(!preview.audio_thread_safe);

        let offline = stretch_backend_plan(StretchBackendTier::OfflineHighQuality);
        assert_eq!(offline.status, StretchBackendStatus::Planned);
        assert!(offline.transient_preservation);
        assert!(offline.vertical_phase_coherence);
        assert!(offline.deterministic_output);
    }

    #[test]
    fn benchmark_corpus_covers_required_material_families() {
        let required = [
            StretchCorpusFamily::DrumsPercussion,
            StretchCorpusFamily::Bass,
            StretchCorpusFamily::Vocals,
            StretchCorpusFamily::PadsSustains,
            StretchCorpusFamily::FullMix,
            StretchCorpusFamily::TempoRamp,
            StretchCorpusFamily::LoopSeam,
            StretchCorpusFamily::ExtremeRatio,
        ];

        for family in required {
            assert!(
                STRETCH_BENCHMARK_CORPUS
                    .iter()
                    .any(|case| case.family == family),
                "missing corpus family {family:?}"
            );
        }
        assert!(STRETCH_BENCHMARK_CORPUS.iter().all(|case| case
            .ratios
            .iter()
            .all(|ratio| ratio.is_finite() && *ratio > 0.0)));
    }

    #[test]
    fn output_length_drift_tracks_fixed_ratio_contract() {
        assert_eq!(output_length_drift_samples(1_000, 1_500, 1.5), 0.0);
        assert_eq!(output_length_drift_samples(1_001, 1_502, 1.5), 0.0);
        assert_eq!(output_length_drift_samples(1_001, 1_503, 1.5), 1.0);
        assert!(output_length_drift_samples(1_000, 1_000, f64::NAN).is_nan());
    }

    #[test]
    fn metric_assessment_aggregates_warnings_and_failures() {
        let measurements = [
            StretchMetricValue::new(StretchMetric::TimingDriftSamples, 0.0),
            StretchMetricValue::new(StretchMetric::StereoImageDelta, 0.2),
            StretchMetricValue::new(StretchMetric::LoopBoundaryClickDbfs, -24.0),
        ];
        let limits = [
            StretchMetricLimit::max(
                StretchMetric::TimingDriftSamples,
                1.0,
                StretchAcceptanceSeverity::Fail,
            ),
            StretchMetricLimit::max(
                StretchMetric::StereoImageDelta,
                0.1,
                StretchAcceptanceSeverity::Warn,
            ),
            StretchMetricLimit::max(
                StretchMetric::LoopBoundaryClickDbfs,
                -60.0,
                StretchAcceptanceSeverity::Fail,
            ),
        ];

        let report = assess_stretch_metrics(&measurements, &limits);

        assert_eq!(report.status, StretchAcceptanceStatus::Fail);
        assert_eq!(report.metrics[0].status, StretchAcceptanceStatus::Pass);
        assert_eq!(report.metrics[1].status, StretchAcceptanceStatus::Warn);
        assert_eq!(report.metrics[2].status, StretchAcceptanceStatus::Fail);
    }

    #[test]
    fn synthetic_corpus_cases_run_without_file_io() {
        let cases = synthetic_stretch_corpus_cases();
        assert_eq!(cases.len(), 3);
        for (case, audio) in cases {
            assert_eq!(case.source, StretchCorpusSource::Synthetic);
            assert!(audio.sample_rate_hz > 0);
            assert!(audio.channels > 0);
            assert_eq!(audio.samples.len() % audio.channels as usize, 0);
            assert!(audio.samples.iter().any(|sample| sample.abs() > 0.01));
        }
    }

    #[test]
    fn acceptance_report_format_is_deterministic() {
        let report = assess_stretch_metrics(
            &[StretchMetricValue::new(
                StretchMetric::TimingDriftSamples,
                0.0,
            )],
            &[StretchMetricLimit::max(
                StretchMetric::TimingDriftSamples,
                1.0,
                StretchAcceptanceSeverity::Fail,
            )],
        );

        assert_eq!(
            format_stretch_acceptance_report("stretch:tempo_ramp", &report),
            "case=stretch:tempo_ramp status=Pass\nmetric=TimingDriftSamples value=0.000000 max=1.000000 status=Pass"
        );
    }
}
