//! Tonal analysis surfaces for Signal.
//!
//! The crate currently exposes offline whole-track key detection driven by
//! STFT-based chroma accumulation.
//!
//! ```no_run
//! use signal_analysis::AnalysisStage;
//! use signal_analysis_tonal::{KeyDetector, KeyDetectorConfig};
//! use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate};
//!
//! let audio = AudioBuffer::from_interleaved(
//!     SampleRate(48_000),
//!     ChannelLayout::Mono,
//!     vec![0.0; 48_000],
//! );
//! let mut detector = KeyDetector::new(KeyDetectorConfig::default());
//! let result = detector.analyze(&audio);
//!
//! assert_eq!(detector.mode(), signal_analysis::AnalysisMode::Offline);
//! assert_eq!(result.chroma.len(), 12);
//! ```

use signal_analysis::{
    prepare_audio_analysis, prepare_mono_analysis, AnalysisInputConfig, AnalysisMode,
    AnalysisStage, Confidence,
};
use signal_dsp_spectral::{bin_frequency, Spectrogram, Stft, StftConfig};
use signal_primitives::{AudioBuffer, FrameCount, Sample, SampleRate, Seconds};

const STANDARD_TUNING_HZ: f32 = 440.0;
const MAX_TUNING_DEVIATION_CENTS: f32 = 50.0;
const SEMITONE_WIDTH_RATIO: f32 = 0.057_762_265;

/// Tonal mode of a detected key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyMode {
    Major,
    Minor,
}

/// Pitch-class tonic for a detected key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tonic {
    C,
    Cs,
    D,
    Ds,
    E,
    F,
    Fs,
    G,
    Gs,
    A,
    As,
    B,
}

/// Whole-track key estimate.
///
/// This is only present when the best-scoring profile correlation is positive.
/// Callers should still pair it with [`TonalAnalysisResult::confidence`] before
/// presenting the key as definitive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key {
    pub tonic: Tonic,
    pub mode: KeyMode,
}

/// Correlation profile family used for key scoring.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyProfile {
    Krumhansl,
    Temperley,
}

/// Tuning-reference policy for chroma accumulation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TuningReferenceMode {
    StandardA440,
    Fixed(f32),
    Estimate,
}

/// Origin of the tuning reference used in the current analysis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TuningReferenceSource {
    StandardA440,
    FixedReference,
    Estimated,
}

/// One scored tuning-reference candidate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TuningCandidate {
    pub reference_hz: f32,
    pub cents_offset: f32,
    pub score: f32,
}

/// Tuning reference used for chroma accumulation and key scoring.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TuningEstimate {
    pub source: TuningReferenceSource,
    pub reference_hz: f32,
    pub cents_offset: f32,
    pub confidence: Confidence,
    pub score: f32,
    pub runner_up: Option<TuningCandidate>,
}

/// One ranked key-profile correlation candidate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TonalProfileCandidate {
    pub key: Key,
    pub correlation: f32,
}

/// Compact scoring diagnostics for the current global-key decision.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TonalScoringSummary {
    pub profile: KeyProfile,
    pub best: Option<TonalProfileCandidate>,
    pub runner_up: Option<TonalProfileCandidate>,
    pub ambiguity: Confidence,
}

/// Explicit ambiguity classes for local tonal analysis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TonalAmbiguityKind {
    WeakTonalCenter,
    CompetingKeyCenters,
    Modulation,
    MixedTonality,
}

/// Ambiguity evidence for one section-local tonal segment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TonalSegmentAmbiguitySummary {
    pub kind: TonalAmbiguityKind,
    pub confidence: Confidence,
    pub best_key: Option<Key>,
    pub alternate_key: Option<Key>,
    pub correlation_gap: f32,
}

/// Section-local tonal summary across one analysis window.
#[derive(Clone, Debug, PartialEq)]
pub struct TonalSegmentSummary {
    pub index: usize,
    pub start_seconds: f32,
    pub end_seconds: f32,
    pub key: Option<Key>,
    pub confidence: Confidence,
    pub chroma: [f32; 12],
    pub scoring: TonalScoringSummary,
    pub ambiguity: Option<TonalSegmentAmbiguitySummary>,
}

/// Coarse classification for a detected local harmonic change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HarmonicChangeKind {
    ConfirmedKeyChange,
    TonalDrift,
}

/// Harmonic change evidence between adjacent local tonal segments.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HarmonicChangeSummary {
    pub kind: HarmonicChangeKind,
    pub from_segment_index: usize,
    pub to_segment_index: usize,
    pub at_seconds: f32,
    pub from_key: Option<Key>,
    pub to_key: Option<Key>,
    pub confidence: Confidence,
    pub chroma_distance: Confidence,
}

/// Higher-level ambiguity surface across the local tonal timeline.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocalTonalAmbiguitySummary {
    pub kind: TonalAmbiguityKind,
    pub confidence: Confidence,
    pub primary_key: Option<Key>,
    pub alternate_key: Option<Key>,
    pub start_segment_index: usize,
    pub end_segment_index: usize,
    pub start_seconds: f32,
    pub end_seconds: f32,
}

/// Windowed local tonal tracking built on the same whole-track tuning/scoring
/// substrate.
#[derive(Clone, Debug, PartialEq)]
pub struct LocalTonalTrackingSummary {
    pub window_seconds: f32,
    pub hop_seconds: f32,
    pub segments: Vec<TonalSegmentSummary>,
    pub changes: Vec<HarmonicChangeSummary>,
    pub ambiguities: Vec<LocalTonalAmbiguitySummary>,
}

/// Analysis depth profile controlling the speed / accuracy trade-off.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnalysisProfile {
    /// 30-second centre segment, 2048-point FFT, no phase computation.
    /// Suitable for rapid library scanning.
    Low,
    /// 60-second centre segment, 4096-point FFT, no phase computation.
    /// Balanced accuracy and performance for interactive use.
    Medium,
    /// Full track, 4096-point FFT, full phase computation.
    /// Maximum accuracy for detailed analysis.
    High,
}

/// Configuration for the key detector.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KeyDetectorConfig {
    pub stft: StftConfig,
    pub profile: KeyProfile,
    pub tuning_reference: TuningReferenceMode,
    /// Search radius around A440 when `tuning_reference` is `Estimate`.
    pub tuning_search_cents: u16,
    /// Grid resolution for the tuning search when `tuning_reference` is
    /// `Estimate`.
    pub tuning_step_cents: u16,
    /// Window size for section-local tonal tracking.
    pub section_window_seconds: u16,
    /// Hop size for section-local tonal tracking.
    pub section_hop_seconds: u16,
    /// Sample rate used by the tonal analysis path after input prep.
    ///
    /// Freezing the analysis rate keeps chroma/profile behavior stable across
    /// source material that arrives at different native rates.
    pub analysis_sample_rate: SampleRate,
    /// Maximum duration to analyse, taken from the centre of the track.
    /// `None` means the entire track is processed.
    pub analysis_duration_seconds: Option<u32>,
}

impl KeyDetectorConfig {
    /// Rapid scanning profile — 30-second centre segment, 4096-point FFT.
    ///
    /// Uses a 4096-point window (rather than the 8192 of higher tiers) for
    /// speed; bass-register pitch-class mapping is less precise but
    /// acceptable for quick scanning.
    pub fn low() -> Self {
        Self {
            stft: StftConfig {
                window_size: FrameCount(4096),
                hop_size: FrameCount(2048),
                compute_phases: false,
            },
            profile: KeyProfile::Krumhansl,
            tuning_reference: TuningReferenceMode::Estimate,
            tuning_search_cents: 50,
            tuning_step_cents: 10,
            section_window_seconds: 4,
            section_hop_seconds: 2,
            analysis_sample_rate: SampleRate(48_000),
            analysis_duration_seconds: Some(30),
        }
    }

    /// Balanced profile — 60-second centre segment, 8192-point FFT.
    ///
    /// The 8192-point window gives ~5.4–5.9 Hz bin spacing (depending on
    /// sample rate), ensuring every semitone in octave 2 and above has at
    /// least one FFT bin mapping to it.  This is critical when combined
    /// with `1/f` chroma weighting, which amplifies bass-register bins
    /// that would otherwise be dwarfed by higher octaves.
    pub fn medium() -> Self {
        Self {
            stft: StftConfig {
                window_size: FrameCount(8192),
                hop_size: FrameCount(4096),
                compute_phases: false,
            },
            profile: KeyProfile::Krumhansl,
            tuning_reference: TuningReferenceMode::Estimate,
            tuning_search_cents: 50,
            tuning_step_cents: 5,
            section_window_seconds: 6,
            section_hop_seconds: 3,
            analysis_sample_rate: SampleRate(48_000),
            analysis_duration_seconds: Some(60),
        }
    }

    /// Full-accuracy profile — entire track, 8192-point FFT.
    pub fn high() -> Self {
        Self {
            stft: StftConfig {
                window_size: FrameCount(8192),
                hop_size: FrameCount(4096),
                compute_phases: false,
            },
            profile: KeyProfile::Krumhansl,
            tuning_reference: TuningReferenceMode::Estimate,
            tuning_search_cents: 50,
            tuning_step_cents: 5,
            section_window_seconds: 8,
            section_hop_seconds: 4,
            analysis_sample_rate: SampleRate(48_000),
            analysis_duration_seconds: None,
        }
    }
}

impl Default for KeyDetectorConfig {
    fn default() -> Self {
        Self::high()
    }
}

/// Whole-track tonal summary returned by [`KeyDetector`].
///
/// Practical integration order:
/// 1. Read `key` for the best whole-track tonic and mode hypothesis.
/// 2. Read `confidence` before treating that key as reliable; low values usually
///    mean the leading and runner-up profile correlations are close.
/// 3. Read `tuning` before assuming the chroma was accumulated at standard
///    concert pitch.
/// 4. Use `chroma`, `correlations`, and `scoring` when a UI or downstream tool
///    needs the supporting evidence rather than just the winning label.
/// 5. Use `local_tracking` when section-local key or harmonic-change evidence
///    is needed without rebuilding windowed chroma off the side.
#[derive(Clone, Debug, PartialEq)]
pub struct TonalAnalysisResult {
    pub key: Option<Key>,
    pub confidence: Confidence,
    pub tuning: TuningEstimate,
    pub chroma: [f32; 12],
    pub correlations: [f32; 24],
    pub scoring: TonalScoringSummary,
    pub local_tracking: LocalTonalTrackingSummary,
}

/// Offline detector for global key and chroma summaries.
#[derive(Debug, Default)]
pub struct KeyDetector {
    config: KeyDetectorConfig,
}

impl KeyDetector {
    /// Create a key detector with the provided config.
    pub fn new(config: KeyDetectorConfig) -> Self {
        Self { config }
    }

    /// Return the current detector config.
    pub fn config(&self) -> KeyDetectorConfig {
        self.config
    }

    /// Analyze a mono sample slice directly.
    pub fn analyze_mono(
        &mut self,
        sample_rate: SampleRate,
        mono_samples: &[Sample],
    ) -> TonalAnalysisResult {
        let prepared =
            prepare_mono_analysis(sample_rate, mono_samples, self.analysis_input_config());
        self.analyze_prepared(prepared.sample_rate, &prepared.samples)
    }

    fn analysis_input_config(&self) -> AnalysisInputConfig {
        AnalysisInputConfig {
            max_duration: self
                .config
                .analysis_duration_seconds
                .map(|seconds| Seconds(seconds as f32)),
            target_sample_rate: Some(self.config.analysis_sample_rate),
            ..AnalysisInputConfig::default()
        }
    }

    fn analyze_prepared(
        &self,
        sample_rate: SampleRate,
        mono_samples: &[Sample],
    ) -> TonalAnalysisResult {
        let stft = Stft::new(self.config.stft);
        let spectrogram = stft.analyze_mono(sample_rate, mono_samples);
        let tuning = estimate_tuning(&spectrogram, self.config);
        let chroma = spectrogram.chroma_with_reference(tuning.reference_hz);
        let local_tracking =
            analyze_local_tonal_tracking(&spectrogram, self.config, tuning.reference_hz);

        score_chroma(chroma, self.config.profile)
            .with_tuning(tuning)
            .with_local_tracking(local_tracking)
    }
}

impl AnalysisStage<TonalAnalysisResult> for KeyDetector {
    fn mode(&self) -> AnalysisMode {
        AnalysisMode::Offline
    }

    fn analyze(&mut self, audio: &AudioBuffer) -> TonalAnalysisResult {
        let prepared = prepare_audio_analysis(audio, self.analysis_input_config());
        self.analyze_prepared(prepared.sample_rate, &prepared.samples)
    }
}

const KRUMHANSL_MAJOR: [f32; 12] = [
    6.35, 2.23, 3.48, 2.33, 4.38, 4.09, 2.52, 5.19, 2.39, 3.66, 2.29, 2.88,
];
const KRUMHANSL_MINOR: [f32; 12] = [
    6.33, 2.68, 3.52, 5.38, 2.60, 3.53, 2.54, 4.75, 3.98, 2.69, 3.34, 3.17,
];
const TEMPERLEY_MAJOR: [f32; 12] = [5.0, 2.0, 3.5, 2.0, 4.5, 4.0, 2.0, 4.5, 2.0, 3.5, 1.5, 4.0];
const TEMPERLEY_MINOR: [f32; 12] = [5.0, 2.0, 3.5, 4.5, 2.0, 4.0, 2.0, 4.5, 3.5, 2.0, 1.5, 4.0];

/// Score a chroma vector against key profiles, returning the best-matching
/// key with its confidence and full correlation set.
fn score_chroma(chroma: [f32; 12], profile: KeyProfile) -> TonalAnalysisResult {
    let correlations = correlate_profiles(chroma, profile);

    let (best_index, best_score) = correlations
        .iter()
        .copied()
        .enumerate()
        .max_by(|(_, lhs), (_, rhs)| lhs.partial_cmp(rhs).unwrap_or(core::cmp::Ordering::Equal))
        .unwrap_or((0, 0.0));

    let second_best = correlations
        .iter()
        .copied()
        .enumerate()
        .filter(|(index, _)| *index != best_index)
        .map(|(_, score)| score)
        .fold(f32::NEG_INFINITY, |best, score| best.max(score));

    let key = if best_score.is_finite() && best_score > second_best && best_score > 0.0 {
        Some(key_from_index(best_index))
    } else {
        None
    };

    let confidence = if best_score > second_best && best_score != 0.0 {
        Confidence::new(((best_score - second_best) / best_score.abs()).max(0.0))
    } else {
        Confidence::new(0.0)
    };

    let best = best_score.is_finite().then_some(TonalProfileCandidate {
        key: key_from_index(best_index),
        correlation: best_score,
    });
    let runner_up = correlations
        .iter()
        .copied()
        .enumerate()
        .filter(|(index, _)| *index != best_index)
        .max_by(|(_, lhs), (_, rhs)| lhs.partial_cmp(rhs).unwrap_or(core::cmp::Ordering::Equal))
        .map(|(index, correlation)| TonalProfileCandidate {
            key: key_from_index(index),
            correlation,
        });

    TonalAnalysisResult {
        key,
        confidence,
        tuning: TuningEstimate {
            source: TuningReferenceSource::StandardA440,
            reference_hz: STANDARD_TUNING_HZ,
            cents_offset: 0.0,
            confidence: Confidence::new(1.0),
            score: 0.0,
            runner_up: None,
        },
        chroma,
        correlations,
        scoring: TonalScoringSummary {
            profile,
            best,
            runner_up,
            ambiguity: confidence,
        },
        local_tracking: LocalTonalTrackingSummary {
            window_seconds: 0.0,
            hop_seconds: 0.0,
            segments: Vec::new(),
            changes: Vec::new(),
            ambiguities: Vec::new(),
        },
    }
}

impl TonalAnalysisResult {
    fn with_tuning(mut self, tuning: TuningEstimate) -> Self {
        self.tuning = tuning;
        self
    }

    fn with_local_tracking(mut self, local_tracking: LocalTonalTrackingSummary) -> Self {
        self.local_tracking = local_tracking;
        self
    }
}

fn analyze_local_tonal_tracking(
    spectrogram: &Spectrogram,
    config: KeyDetectorConfig,
    reference_hz: f32,
) -> LocalTonalTrackingSummary {
    let frame_chromas = spectrogram_frame_chromas(spectrogram, reference_hz);
    let frame_count = frame_chromas.len();
    if frame_count == 0 || spectrogram.sample_rate.0 == 0 {
        return LocalTonalTrackingSummary {
            window_seconds: config.section_window_seconds as f32,
            hop_seconds: config.section_hop_seconds as f32,
            segments: Vec::new(),
            changes: Vec::new(),
            ambiguities: Vec::new(),
        };
    }

    let frame_hop_seconds = spectrogram.config.hop_size.0 as f32 / spectrogram.sample_rate.0 as f32;
    let window_frames =
        ((config.section_window_seconds as f32 / frame_hop_seconds).round() as usize).max(1);
    let hop_frames =
        ((config.section_hop_seconds as f32 / frame_hop_seconds).round() as usize).max(1);
    let mut segments = Vec::new();
    let mut start = 0usize;

    loop {
        let end = (start + window_frames).min(frame_count);
        let chroma = aggregate_chroma(&frame_chromas[start..end]);
        let segment_result = score_chroma(chroma, config.profile);
        let start_seconds = frame_index_to_seconds(start, spectrogram);
        let end_seconds = frame_end_to_seconds(end.saturating_sub(1), spectrogram);

        segments.push(TonalSegmentSummary {
            index: segments.len(),
            start_seconds,
            end_seconds,
            key: segment_result.key,
            confidence: segment_result.confidence,
            chroma,
            scoring: segment_result.scoring,
            ambiguity: segment_ambiguity(&segment_result),
        });

        if end >= frame_count {
            break;
        }
        start = start.saturating_add(hop_frames);
    }

    let changes = harmonic_changes(&segments);
    let ambiguities = local_tonal_ambiguities(&segments, &changes);
    LocalTonalTrackingSummary {
        window_seconds: config.section_window_seconds as f32,
        hop_seconds: config.section_hop_seconds as f32,
        segments,
        changes,
        ambiguities,
    }
}

fn spectrogram_frame_chromas(spectrogram: &Spectrogram, reference_hz: f32) -> Vec<[f32; 12]> {
    let window_size = spectrogram.config.window_size.0;
    if spectrogram.frames.is_empty() || window_size == 0 || spectrogram.sample_rate.0 == 0 {
        return Vec::new();
    }

    let bin_spacing = spectrogram.sample_rate.0 as f32 / window_size as f32;
    let min_frequency = bin_spacing / SEMITONE_WIDTH_RATIO;
    let mut chromas = Vec::with_capacity(spectrogram.frames.len());

    for frame in &spectrogram.frames {
        let mut chroma = [0.0; 12];
        for (bin_index, magnitude) in frame.magnitudes.iter().enumerate().skip(1) {
            let frequency = bin_frequency(bin_index, spectrogram.sample_rate, window_size);
            if frequency < min_frequency || frequency > 5_000.0 {
                continue;
            }
            let midi = 69.0 + 12.0 * (frequency / reference_hz.max(1.0)).log2();
            let pitch_class = (midi.round() as i32).rem_euclid(12) as usize;
            chroma[pitch_class] += *magnitude / frequency;
        }
        normalize_array(&mut chroma);
        chromas.push(chroma);
    }

    chromas
}

fn aggregate_chroma(frame_chromas: &[[f32; 12]]) -> [f32; 12] {
    let mut chroma = [0.0; 12];
    for frame in frame_chromas {
        for (slot, value) in chroma.iter_mut().zip(frame.iter().copied()) {
            *slot += value;
        }
    }
    normalize_array(&mut chroma);
    chroma
}

fn normalize_array(values: &mut [f32; 12]) {
    let max_value = values.iter().copied().fold(0.0f32, f32::max);
    if max_value > 0.0 {
        for value in values.iter_mut() {
            *value /= max_value;
        }
    }
}

fn frame_index_to_seconds(frame_index: usize, spectrogram: &Spectrogram) -> f32 {
    frame_index as f32 * spectrogram.config.hop_size.0 as f32 / spectrogram.sample_rate.0 as f32
}

fn frame_end_to_seconds(frame_index: usize, spectrogram: &Spectrogram) -> f32 {
    let end_samples = frame_index
        .saturating_mul(spectrogram.config.hop_size.0)
        .saturating_add(spectrogram.config.window_size.0);
    end_samples as f32 / spectrogram.sample_rate.0 as f32
}

fn harmonic_changes(segments: &[TonalSegmentSummary]) -> Vec<HarmonicChangeSummary> {
    let mut changes = Vec::new();

    for pair in segments.windows(2) {
        let left = &pair[0];
        let right = &pair[1];
        let distance = chroma_distance(left.chroma, right.chroma);
        let distance_confidence = Confidence::new(distance);
        let key_changed = left.key != right.key && left.key.is_some() && right.key.is_some();
        let confidence = if key_changed {
            Confidence::new(
                ((left.confidence.0 + right.confidence.0) * 0.5 * distance.max(0.35))
                    .clamp(0.0, 1.0),
            )
        } else {
            Confidence::new(
                ((left.confidence.0 + right.confidence.0) * 0.25 * distance).clamp(0.0, 1.0),
            )
        };

        let kind = if key_changed {
            Some(HarmonicChangeKind::ConfirmedKeyChange)
        } else if distance >= 0.30 && (left.confidence.0 >= 0.15 || right.confidence.0 >= 0.15) {
            Some(HarmonicChangeKind::TonalDrift)
        } else {
            None
        };

        if let Some(kind) = kind {
            changes.push(HarmonicChangeSummary {
                kind,
                from_segment_index: left.index,
                to_segment_index: right.index,
                at_seconds: right.start_seconds,
                from_key: left.key,
                to_key: right.key,
                confidence,
                chroma_distance: distance_confidence,
            });
        }
    }

    changes
}

fn segment_ambiguity(result: &TonalAnalysisResult) -> Option<TonalSegmentAmbiguitySummary> {
    let best = result.scoring.best?;
    let runner_up = result.scoring.runner_up;
    let correlation_gap = runner_up
        .map(|candidate| (best.correlation - candidate.correlation).abs())
        .unwrap_or(best.correlation.abs());
    let ambiguity_confidence = Confidence::new((1.0 - result.confidence.0).clamp(0.0, 1.0));

    if result.key.is_none() || result.confidence.0 < 0.10 || best.correlation < 0.45 {
        return Some(TonalSegmentAmbiguitySummary {
            kind: TonalAmbiguityKind::WeakTonalCenter,
            confidence: ambiguity_confidence,
            best_key: Some(best.key),
            alternate_key: runner_up.map(|candidate| candidate.key),
            correlation_gap,
        });
    }

    if let Some(runner_up) = runner_up {
        if runner_up.key != best.key && correlation_gap <= 0.08 {
            return Some(TonalSegmentAmbiguitySummary {
                kind: TonalAmbiguityKind::CompetingKeyCenters,
                confidence: ambiguity_confidence,
                best_key: Some(best.key),
                alternate_key: Some(runner_up.key),
                correlation_gap,
            });
        }
    }

    None
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct StableKeyRun {
    key: Key,
    start_segment_index: usize,
    end_segment_index: usize,
    start_seconds: f32,
    end_seconds: f32,
    average_confidence: f32,
}

fn local_tonal_ambiguities(
    segments: &[TonalSegmentSummary],
    changes: &[HarmonicChangeSummary],
) -> Vec<LocalTonalAmbiguitySummary> {
    if segments.is_empty() {
        return Vec::new();
    }

    let mut ambiguities = Vec::new();
    let first_segment = segments.first().expect("non-empty segments");
    let last_segment = segments.last().expect("non-empty segments");
    let average_segment_confidence = segments
        .iter()
        .map(|segment| segment.confidence.0)
        .sum::<f32>()
        / segments.len() as f32;
    let weak_segments: Vec<&TonalSegmentSummary> = segments
        .iter()
        .filter(|segment| {
            matches!(
                segment.ambiguity,
                Some(TonalSegmentAmbiguitySummary {
                    kind: TonalAmbiguityKind::WeakTonalCenter,
                    ..
                })
            )
        })
        .collect();
    let stable_runs = stable_key_runs(segments);
    let confirmed_changes: Vec<&HarmonicChangeSummary> = changes
        .iter()
        .filter(|change| change.kind == HarmonicChangeKind::ConfirmedKeyChange)
        .collect();
    let competing_segments: Vec<&TonalSegmentSummary> = segments
        .iter()
        .filter(|segment| {
            matches!(
                segment.ambiguity,
                Some(TonalSegmentAmbiguitySummary {
                    kind: TonalAmbiguityKind::CompetingKeyCenters,
                    ..
                })
            )
        })
        .collect();

    if (weak_segments.len() * 2 >= segments.len() || average_segment_confidence < 0.12)
        && confirmed_changes.is_empty()
        && stable_runs.len() <= 1
    {
        ambiguities.push(LocalTonalAmbiguitySummary {
            kind: TonalAmbiguityKind::WeakTonalCenter,
            confidence: Confidence::new(weak_segments.len() as f32 / segments.len() as f32),
            primary_key: segments.iter().find_map(|segment| segment.key),
            alternate_key: weak_segments.iter().find_map(|segment| {
                segment
                    .ambiguity
                    .and_then(|ambiguity| ambiguity.alternate_key)
            }),
            start_segment_index: first_segment.index,
            end_segment_index: last_segment.index,
            start_seconds: first_segment.start_seconds,
            end_seconds: last_segment.end_seconds,
        });
    }

    if confirmed_changes.len() == 1 {
        let change = confirmed_changes[0];
        ambiguities.push(LocalTonalAmbiguitySummary {
            kind: TonalAmbiguityKind::Modulation,
            confidence: change.confidence,
            primary_key: change.from_key,
            alternate_key: change.to_key,
            start_segment_index: change.from_segment_index,
            end_segment_index: change.to_segment_index,
            start_seconds: segments[change.from_segment_index].start_seconds,
            end_seconds: segments[change.to_segment_index].end_seconds,
        });
    } else if confirmed_changes.len() > 1 || stable_runs.len() > 2 || competing_segments.len() >= 2
    {
        let primary_key = stable_runs
            .first()
            .map(|run| run.key)
            .or_else(|| competing_segments.first().and_then(|segment| segment.key));
        let alternate_key = stable_runs.get(1).map(|run| run.key).or_else(|| {
            competing_segments.iter().find_map(|segment| {
                segment
                    .ambiguity
                    .and_then(|ambiguity| ambiguity.alternate_key)
            })
        });
        let ambiguity_strength = if !competing_segments.is_empty() {
            competing_segments
                .iter()
                .filter_map(|segment| segment.ambiguity.map(|ambiguity| ambiguity.confidence.0))
                .sum::<f32>()
                / competing_segments.len() as f32
        } else {
            (stable_runs.len() as f32 / segments.len() as f32).clamp(0.0, 1.0)
        };
        ambiguities.push(LocalTonalAmbiguitySummary {
            kind: TonalAmbiguityKind::MixedTonality,
            confidence: Confidence::new(ambiguity_strength.clamp(0.0, 1.0)),
            primary_key,
            alternate_key,
            start_segment_index: first_segment.index,
            end_segment_index: last_segment.index,
            start_seconds: first_segment.start_seconds,
            end_seconds: last_segment.end_seconds,
        });
    }

    ambiguities
}

fn stable_key_runs(segments: &[TonalSegmentSummary]) -> Vec<StableKeyRun> {
    let mut runs: Vec<StableKeyRun> = Vec::new();

    for segment in segments.iter().filter(|segment| segment.key.is_some()) {
        if segment.confidence.0 < 0.10 {
            continue;
        }
        if matches!(
            segment.ambiguity,
            Some(TonalSegmentAmbiguitySummary {
                kind: TonalAmbiguityKind::WeakTonalCenter,
                ..
            })
        ) {
            continue;
        }

        let key = segment.key.expect("filtered to some key");
        match runs.last_mut() {
            Some(run) if run.key == key => {
                let run_length = (run
                    .end_segment_index
                    .saturating_sub(run.start_segment_index)
                    + 1) as f32;
                run.end_segment_index = segment.index;
                run.end_seconds = segment.end_seconds;
                run.average_confidence = ((run.average_confidence * run_length)
                    + segment.confidence.0)
                    / (run_length + 1.0);
            }
            _ => {
                runs.push(StableKeyRun {
                    key,
                    start_segment_index: segment.index,
                    end_segment_index: segment.index,
                    start_seconds: segment.start_seconds,
                    end_seconds: segment.end_seconds,
                    average_confidence: segment.confidence.0,
                });
            }
        }
    }

    runs
}

fn chroma_distance(lhs: [f32; 12], rhs: [f32; 12]) -> f32 {
    let total = lhs
        .iter()
        .zip(rhs.iter())
        .map(|(left, right)| (left - right).abs())
        .sum::<f32>();
    (0.5 * total).clamp(0.0, 1.0)
}

fn estimate_tuning(spectrogram: &Spectrogram, config: KeyDetectorConfig) -> TuningEstimate {
    match config.tuning_reference {
        TuningReferenceMode::StandardA440 => TuningEstimate {
            source: TuningReferenceSource::StandardA440,
            reference_hz: STANDARD_TUNING_HZ,
            cents_offset: 0.0,
            confidence: Confidence::new(1.0),
            score: 0.0,
            runner_up: None,
        },
        TuningReferenceMode::Fixed(reference_hz) => TuningEstimate {
            source: TuningReferenceSource::FixedReference,
            reference_hz,
            cents_offset: cents_offset_from_standard(reference_hz),
            confidence: Confidence::new(1.0),
            score: 0.0,
            runner_up: None,
        },
        TuningReferenceMode::Estimate => estimate_tuning_reference(spectrogram, config),
    }
}

fn estimate_tuning_reference(
    spectrogram: &Spectrogram,
    config: KeyDetectorConfig,
) -> TuningEstimate {
    let search_cents = config.tuning_search_cents.max(1) as i32;
    let step_cents = config.tuning_step_cents.max(1) as i32;
    let mut best: Option<TuningCandidate> = None;
    let mut runner_up: Option<TuningCandidate> = None;

    for cents in (-search_cents..=search_cents).step_by(step_cents as usize) {
        let cents_offset = cents as f32;
        let reference_hz = reference_hz_from_cents(cents_offset);
        let score = tuning_alignment_score(spectrogram, reference_hz);
        let candidate = TuningCandidate {
            reference_hz,
            cents_offset,
            score,
        };

        let replace_best = match best {
            None => true,
            Some(current) => {
                candidate.score > current.score
                    || ((candidate.score - current.score).abs() < 1.0e-6
                        && candidate.cents_offset.abs() < current.cents_offset.abs())
            }
        };

        if replace_best {
            runner_up = best;
            best = Some(candidate);
        } else if runner_up.is_none_or(|current| candidate.score > current.score) {
            runner_up = Some(candidate);
        }
    }

    let best = best.unwrap_or(TuningCandidate {
        reference_hz: STANDARD_TUNING_HZ,
        cents_offset: 0.0,
        score: 0.0,
    });
    let confidence = if let Some(runner_up) = runner_up {
        if best.score > runner_up.score && best.score > 0.0 {
            Confidence::new(((best.score - runner_up.score) / best.score.abs()).max(0.0))
        } else {
            Confidence::new(0.0)
        }
    } else {
        Confidence::new(0.0)
    };

    TuningEstimate {
        source: TuningReferenceSource::Estimated,
        reference_hz: best.reference_hz,
        cents_offset: best.cents_offset,
        confidence,
        score: best.score,
        runner_up,
    }
}

fn tuning_alignment_score(spectrogram: &Spectrogram, reference_hz: f32) -> f32 {
    let window_size = spectrogram.config.window_size.0;
    if spectrogram.frames.is_empty() || window_size == 0 || spectrogram.sample_rate.0 == 0 {
        return 0.0;
    }

    let bin_spacing = spectrogram.sample_rate.0 as f32 / window_size as f32;
    let min_frequency = bin_spacing / SEMITONE_WIDTH_RATIO;
    let mut score = 0.0f32;

    for frame in &spectrogram.frames {
        for (bin_index, magnitude) in frame.magnitudes.iter().enumerate().skip(1) {
            let frequency = bin_frequency(bin_index, spectrogram.sample_rate, window_size);
            if frequency < min_frequency || frequency > 5_000.0 {
                continue;
            }

            let midi = 69.0 + 12.0 * (frequency / reference_hz.max(1.0)).log2();
            let cents = 100.0 * (midi - midi.round()).abs();
            let closeness = (1.0 - cents / MAX_TUNING_DEVIATION_CENTS).max(0.0);
            score += (magnitude / frequency) * closeness * closeness;
        }
    }

    score
}

fn reference_hz_from_cents(cents_offset: f32) -> f32 {
    STANDARD_TUNING_HZ * 2.0_f32.powf(cents_offset / 1200.0)
}

fn cents_offset_from_standard(reference_hz: f32) -> f32 {
    1200.0 * (reference_hz / STANDARD_TUNING_HZ).log2()
}

/// Compute Pearson correlation coefficients between the observed chroma and
/// all 24 rotated key profiles.
///
/// Pearson correlation centres both vectors around their means before
/// computing the cosine of the angle between them.  This is the standard
/// approach in the Krumhansl-Schmuckler key-finding algorithm and is
/// substantially better at distinguishing relative major/minor keys (e.g.
/// A minor vs C major) than a raw dot product, because it measures the
/// *shape* of the distribution rather than being dominated by which pitch
/// classes carry the most absolute energy.
fn correlate_profiles(chroma: [f32; 12], profile: KeyProfile) -> [f32; 24] {
    let (major_profile, minor_profile) = match profile {
        KeyProfile::Krumhansl => (KRUMHANSL_MAJOR, KRUMHANSL_MINOR),
        KeyProfile::Temperley => (TEMPERLEY_MAJOR, TEMPERLEY_MINOR),
    };

    let mut correlations = [0.0; 24];
    for tonic in 0..12 {
        correlations[tonic] = pearson(chroma, rotate_profile(&major_profile, tonic));
        correlations[12 + tonic] = pearson(chroma, rotate_profile(&minor_profile, tonic));
    }
    correlations
}

fn rotate_profile(profile: &[f32; 12], tonic: usize) -> [f32; 12] {
    let mut rotated = [0.0; 12];
    for (index, value) in profile.iter().copied().enumerate() {
        rotated[(index + tonic) % 12] = value;
    }
    rotated
}

/// Pearson product-moment correlation coefficient between two 12-element
/// vectors.  Returns 0.0 when either vector has zero variance (e.g. all
/// values identical or all zeros after mean-centring).
fn pearson(x: [f32; 12], y: [f32; 12]) -> f32 {
    let n = 12.0f32;
    let x_mean = x.iter().copied().sum::<f32>() / n;
    let y_mean = y.iter().copied().sum::<f32>() / n;

    let mut numerator = 0.0f32;
    let mut x_var = 0.0f32;
    let mut y_var = 0.0f32;

    for i in 0..12 {
        let dx = x[i] - x_mean;
        let dy = y[i] - y_mean;
        numerator += dx * dy;
        x_var += dx * dx;
        y_var += dy * dy;
    }

    let denominator = (x_var * y_var).sqrt();
    if denominator == 0.0 {
        0.0
    } else {
        numerator / denominator
    }
}

fn key_from_index(index: usize) -> Key {
    let tonic = tonic_from_index(index % 12);
    let mode = if index < 12 {
        KeyMode::Major
    } else {
        KeyMode::Minor
    };
    Key { tonic, mode }
}

fn tonic_from_index(index: usize) -> Tonic {
    match index % 12 {
        0 => Tonic::C,
        1 => Tonic::Cs,
        2 => Tonic::D,
        3 => Tonic::Ds,
        4 => Tonic::E,
        5 => Tonic::F,
        6 => Tonic::Fs,
        7 => Tonic::G,
        8 => Tonic::Gs,
        9 => Tonic::A,
        10 => Tonic::As,
        _ => Tonic::B,
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
