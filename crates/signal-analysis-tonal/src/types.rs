use signal_analysis::Confidence;
use signal_dsp_spectral::StftConfig;
use signal_primitives::{FrameCount, SampleRate};

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
    /// least one FFT bin mapping to it. This is critical when combined
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
