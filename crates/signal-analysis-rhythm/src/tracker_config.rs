use signal_dsp_spectral::StftConfig;
use signal_primitives::{FrameCount, SampleRate};

/// Controls the trade-off between speed and accuracy in rhythm analysis.
///
/// Each tier configures the FFT size, onset-feature set, segment duration,
/// and meter inference to match a different use case.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnalysisProfile {
    /// 30-second centre segment, 1024-point FFT, no phase computation,
    /// three onset features, no meter inference.  Suitable for rapid
    /// library scanning.  ~20× faster than [`High`](AnalysisProfile::High)
    /// on a 4-minute track.
    Low,
    /// 60-second centre segment, 1024-point FFT, no phase computation,
    /// three onset features, with meter inference.  Balanced accuracy and
    /// performance for interactive use.  ~5× faster than
    /// [`High`](AnalysisProfile::High).
    Medium,
    /// Full track, 2048-point FFT with phases, all five onset features,
    /// full meter inference and diagnostics.  Maximum accuracy.
    High,
}

/// Configuration for the offline beat tracker.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BeatTrackerConfig {
    /// STFT parameters: window size, hop size, and phase computation flag.
    pub stft: StftConfig,
    /// Minimum tempo the tracker will consider, in BPM.
    pub min_bpm: f32,
    /// Maximum tempo the tracker will consider, in BPM.
    pub max_bpm: f32,
    /// Fractional beat-period tolerance for beat tracking (0–1).
    pub beat_tolerance: f32,
    /// Sample rate used by the rhythm analysis path after input prep.
    ///
    /// Freezing the analysis rate keeps onset framing and tempo heuristics on
    /// one stable domain across source material with different native rates.
    pub analysis_sample_rate: SampleRate,
    /// When set, only analyze this many seconds from the centre of the track.
    /// Dramatically reduces processing time for long audio files.
    pub analysis_duration_seconds: Option<f32>,
    /// Controls the speed/accuracy trade-off.  See [`AnalysisProfile`].
    pub profile: AnalysisProfile,
}

impl Default for BeatTrackerConfig {
    fn default() -> Self {
        Self::high()
    }
}

impl BeatTrackerConfig {
    /// Fastest preset — 30-second centre segment, small FFT, reduced
    /// onset features, no meter.  ~20× faster than [`high`](Self::high).
    pub fn low() -> Self {
        Self {
            stft: StftConfig {
                window_size: FrameCount(1024),
                hop_size: FrameCount(512),
                compute_phases: false,
            },
            min_bpm: 70.0,
            max_bpm: 180.0,
            beat_tolerance: 0.2,
            analysis_sample_rate: SampleRate(48_000),
            analysis_duration_seconds: Some(30.0),
            profile: AnalysisProfile::Low,
        }
    }

    /// Balanced preset — 60-second centre segment, small FFT, reduced
    /// onset features, with meter.  ~5× faster than [`high`](Self::high).
    pub fn medium() -> Self {
        Self {
            stft: StftConfig {
                window_size: FrameCount(1024),
                hop_size: FrameCount(512),
                compute_phases: false,
            },
            min_bpm: 70.0,
            max_bpm: 180.0,
            beat_tolerance: 0.2,
            analysis_sample_rate: SampleRate(48_000),
            analysis_duration_seconds: Some(60.0),
            profile: AnalysisProfile::Medium,
        }
    }

    /// Full-accuracy preset — entire track, large FFT with phases, all
    /// five onset features, full meter and diagnostics.
    pub fn high() -> Self {
        Self {
            stft: StftConfig::new(2048, 512),
            min_bpm: 70.0,
            max_bpm: 180.0,
            beat_tolerance: 0.2,
            analysis_sample_rate: SampleRate(48_000),
            analysis_duration_seconds: None,
            profile: AnalysisProfile::High,
        }
    }
}
