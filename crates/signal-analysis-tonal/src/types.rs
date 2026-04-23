use signal_analysis::Confidence;
use signal_dsp_spectral::StftConfig;
use signal_primitives::{FrameCount, SampleRate};

/// Tonal mode of a detected key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyMode {
    /// Ionian / major scale.
    Major,
    /// Aeolian / natural minor scale.
    Minor,
}

/// Pitch-class tonic for a detected key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tonic {
    /// C natural.
    C,
    /// C sharp / D flat.
    Cs,
    /// D natural.
    D,
    /// D sharp / E flat.
    Ds,
    /// E natural.
    E,
    /// F natural.
    F,
    /// F sharp / G flat.
    Fs,
    /// G natural.
    G,
    /// G sharp / A flat.
    Gs,
    /// A natural.
    A,
    /// A sharp / B flat.
    As,
    /// B natural.
    B,
}

/// Whole-track key estimate.
///
/// This is only present when the best-scoring profile correlation is positive.
/// Callers should still pair it with [`TonalAnalysisResult::confidence`] before
/// presenting the key as definitive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Key {
    /// Pitch-class root of the estimated key.
    pub tonic: Tonic,
    /// Major or minor mode of the estimated key.
    pub mode: KeyMode,
}

/// Correlation profile family used for key scoring.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyProfile {
    /// Krumhansl-Kessler (1982) key-profile weights.
    Krumhansl,
    /// Temperley (1999) revised key-profile weights.
    Temperley,
}

/// Tuning-reference policy for chroma accumulation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TuningReferenceMode {
    /// Always use A = 440 Hz.
    StandardA440,
    /// Use the given Hz value as the A4 reference.
    Fixed(f32),
    /// Search for the best-fit tuning reference within the configured window.
    Estimate,
}

/// Origin of the tuning reference used in the current analysis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TuningReferenceSource {
    /// Reference was the standard A = 440 Hz.
    StandardA440,
    /// Reference was a caller-supplied fixed value.
    FixedReference,
    /// Reference was found by the internal tuning search.
    Estimated,
}

/// One scored tuning-reference candidate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TuningCandidate {
    /// A4 reference frequency for this candidate, in Hz.
    pub reference_hz: f32,
    /// Offset from standard A440, in cents.
    pub cents_offset: f32,
    /// Alignment score against the spectrogram (higher is better).
    pub score: f32,
}

/// Tuning reference used for chroma accumulation and key scoring.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TuningEstimate {
    /// How the reference was determined.
    pub source: TuningReferenceSource,
    /// Resolved A4 reference frequency, in Hz.
    pub reference_hz: f32,
    /// Offset from standard A440, in cents. Zero for standard tuning.
    pub cents_offset: f32,
    /// Confidence in the tuning estimate; low when candidates are close in score.
    pub confidence: Confidence,
    /// Alignment score of the winning candidate (higher is better).
    pub score: f32,
    /// Second-best candidate from the tuning search, if one exists.
    pub runner_up: Option<TuningCandidate>,
}

/// One ranked key-profile correlation candidate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TonalProfileCandidate {
    /// Key hypothesis for this candidate.
    pub key: Key,
    /// Pearson correlation between the observed chroma and this key's profile.
    pub correlation: f32,
}

/// Compact scoring diagnostics for the current global-key decision.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TonalScoringSummary {
    /// Profile family used for correlation scoring.
    pub profile: KeyProfile,
    /// Highest-scoring key candidate, if any.
    pub best: Option<TonalProfileCandidate>,
    /// Second-highest-scoring key candidate, if any.
    pub runner_up: Option<TonalProfileCandidate>,
    /// Ambiguity between `best` and `runner_up`; high means the two are close.
    pub ambiguity: Confidence,
}

/// Explicit ambiguity classes for local tonal analysis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TonalAmbiguityKind {
    /// No dominant tonic; flat or noise-heavy chroma.
    WeakTonalCenter,
    /// Two or more keys score comparably within the same window.
    CompetingKeyCenters,
    /// Tonal center shifts between adjacent analysis windows.
    Modulation,
    /// Major and minor of the same tonic are nearly tied.
    MixedTonality,
}

/// Ambiguity evidence for one section-local tonal segment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TonalSegmentAmbiguitySummary {
    /// Ambiguity class for this segment.
    pub kind: TonalAmbiguityKind,
    /// Confidence in the ambiguity classification.
    pub confidence: Confidence,
    /// Highest-scoring key in the segment, if any.
    pub best_key: Option<Key>,
    /// Second-scoring key in the segment, if any.
    pub alternate_key: Option<Key>,
    /// Pearson correlation gap between `best_key` and `alternate_key`.
    pub correlation_gap: f32,
}

/// Section-local tonal summary across one analysis window.
#[derive(Clone, Debug, PartialEq)]
pub struct TonalSegmentSummary {
    /// Zero-based segment index in the local tracking timeline.
    pub index: usize,
    /// Segment start time relative to the analysed buffer start, in seconds.
    pub start_seconds: f32,
    /// Segment end time relative to the analysed buffer start, in seconds.
    pub end_seconds: f32,
    /// Best-matching key for this segment, if the correlation is positive.
    pub key: Option<Key>,
    /// Confidence in the segment key estimate.
    pub confidence: Confidence,
    /// Chroma vector accumulated over this segment.
    pub chroma: [f32; 12],
    /// Full scoring diagnostics for this segment's key decision.
    pub scoring: TonalScoringSummary,
    /// Ambiguity evidence for this segment, present when ambiguity is detected.
    pub ambiguity: Option<TonalSegmentAmbiguitySummary>,
}

/// Coarse classification for a detected local harmonic change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HarmonicChangeKind {
    /// Both adjacent segments have confident, differing keys.
    ConfirmedKeyChange,
    /// Key changed but at least one segment has low tonal confidence.
    TonalDrift,
}

/// Harmonic change evidence between adjacent local tonal segments.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HarmonicChangeSummary {
    /// Coarse classification of the change.
    pub kind: HarmonicChangeKind,
    /// Index of the outgoing segment.
    pub from_segment_index: usize,
    /// Index of the incoming segment.
    pub to_segment_index: usize,
    /// Time of the boundary between segments, in seconds.
    pub at_seconds: f32,
    /// Key of the outgoing segment.
    pub from_key: Option<Key>,
    /// Key of the incoming segment.
    pub to_key: Option<Key>,
    /// Overall confidence in the reported change.
    pub confidence: Confidence,
    /// Normalized chroma distance between the two adjacent segments.
    pub chroma_distance: Confidence,
}

/// Higher-level ambiguity surface across the local tonal timeline.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LocalTonalAmbiguitySummary {
    /// Ambiguity class for this region.
    pub kind: TonalAmbiguityKind,
    /// Confidence in the ambiguity classification.
    pub confidence: Confidence,
    /// Dominant key hypothesis across the ambiguous region.
    pub primary_key: Option<Key>,
    /// Competing key hypothesis across the ambiguous region.
    pub alternate_key: Option<Key>,
    /// Index of the first segment in the ambiguous region.
    pub start_segment_index: usize,
    /// Index of the last segment in the ambiguous region (inclusive).
    pub end_segment_index: usize,
    /// Start time of the ambiguous region, in seconds.
    pub start_seconds: f32,
    /// End time of the ambiguous region, in seconds.
    pub end_seconds: f32,
}

/// Windowed local tonal tracking built on the same whole-track tuning/scoring
/// substrate.
#[derive(Clone, Debug, PartialEq)]
pub struct LocalTonalTrackingSummary {
    /// Window size used for local segment analysis, in seconds.
    pub window_seconds: f32,
    /// Hop size between adjacent local segments, in seconds.
    pub hop_seconds: f32,
    /// Ordered list of per-segment tonal summaries.
    pub segments: Vec<TonalSegmentSummary>,
    /// Detected harmonic changes between adjacent segments.
    pub changes: Vec<HarmonicChangeSummary>,
    /// Detected higher-level ambiguity regions spanning multiple segments.
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
    /// STFT parameters for chroma accumulation.
    pub stft: StftConfig,
    /// Key-profile family used for correlation scoring.
    pub profile: KeyProfile,
    /// Tuning-reference policy applied before chroma accumulation.
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
    /// Best whole-track key estimate. `None` when no positive correlation exists.
    pub key: Option<Key>,
    /// Confidence in `key`; low when the best and runner-up correlations are close.
    pub confidence: Confidence,
    /// Tuning reference resolved and used for chroma accumulation.
    pub tuning: TuningEstimate,
    /// Accumulated chroma vector; 12 pitch-class bins in C-to-B order.
    pub chroma: [f32; 12],
    /// Pearson correlations for all 24 key profiles (12 major then 12 minor, C-rooted).
    pub correlations: [f32; 24],
    /// Scoring diagnostics for the global key decision.
    pub scoring: TonalScoringSummary,
    /// Windowed local tonal tracking over the analysed buffer.
    pub local_tracking: LocalTonalTrackingSummary,
}
